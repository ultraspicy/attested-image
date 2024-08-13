#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <libavutil/imgutils.h>
#include <libavutil/parseutils.h>
#include <libswscale/swscale.h>
#include <math.h>

#define SWS_FAST_BILINEAR     1
#define SWS_BILINEAR          2
#define SWS_BICUBIC           4

#define FILTER_BITS 14
#define FILTER_SCALE (1 << 14)

typedef struct {
    int *filterPos;
    int16_t *filter;
    int filterSize;
    int *xInc;
} SwsContext;

typedef struct {
    int *filterPos;
    int16_t *filter;
    int filterSize;
    int *xInc;
} Context;

static Context* alloc_context() {
    Context *c = (Context*)calloc(1, sizeof(Context));
    return c;
}

static void free_context(Context *c) {
    if (c) {
        free(c->filterPos);
        free(c->filter);
        free(c->xInc);
        free(c);
    }
}

static int init_filter(Context *c, int srcW, int dstW, int filterSize) {
    int i, j;
    c->filterPos = (int*)malloc(dstW * sizeof(int));
    c->filter = (int16_t*)malloc(dstW * filterSize * sizeof(int16_t));
    c->filterSize = filterSize;

    if (!c->filterPos || !c->filter)
        return -1;

    int64_t xInc = (((int64_t)srcW << 16) / dstW + 1) >> 1; // scaling factor in 16.16 fix point

    for (i = 0; i < dstW; i++) {
        int64_t srcPos = ((int64_t)i * xInc) >> 16;
        int64_t xxInc = xInc & 0xffff;
        int xx = xxInc * (1 << FILTER_BITS) / xInc;

        c->filterPos[i] = srcPos;

        for (j = 0; j < filterSize; j++) {
            int64_t coeff;
            if (j == 0) {
                coeff = (1 << FILTER_BITS) - xx ;
            } else {
                coeff = xx;
            }
            c->filter[i * filterSize + j] = (int16_t)coeff;
        }

        // Normalize filter coefficients
        int64_t sum = 0;
        for (j = 0; j < filterSize; j++)
            sum += c->filter[i * filterSize + j];
        
        if (sum != FILTER_SCALE) {
            for (j = 0; j < filterSize; j++) {
                int64_t coeff = (int64_t)c->filter[i * filterSize + j] * FILTER_SCALE / sum;
                c->filter[i * filterSize + j] = (int32_t)coeff;
            }
        }
    }

    return 0;
}

void resize_image(const uint8_t* input, uint8_t* output, 
                  int input_width, int input_height, 
                  int output_width, int output_height) {
    Context *c = alloc_context();
    int filterSize = SWS_BILINEAR;  // for bilinear
    
    if (init_filter(c, input_width, output_width, filterSize) < 0) {
        free_context(c);
        return;
    }

    double scale_y = (double)input_height / output_height;

    for (int i = 0; i < output_height; i++) {
        int srcY = (int)(i * scale_y);
        int nextY = FFMIN(srcY + 1, input_height - 1);
        int wy = (int)((i * scale_y - srcY) * FILTER_SCALE + 0.5);

        for (int j = 0; j < output_width; j++) {
            int srcX = c->filterPos[j];
            int16_t *filter = &c->filter[j * filterSize];

            int32_t val = 0;
            for (int k = 0; k < filterSize; k++) {
                int x = FFMIN(srcX + k, input_width - 1);
                int coeff_x = filter[k];
                
                int pixel1 = input[srcY * input_width + x];
                int pixel2 = input[nextY * input_width + x];
                
                int interpolated = (pixel1 * (FILTER_SCALE - wy) + pixel2 * wy) >> FILTER_BITS;
                val += interpolated * coeff_x;
            }

            val = (val + (1 << (FILTER_BITS - 1))) >> FILTER_BITS;
            output[i * output_width + j] = FFMIN(FFMAX(val, 0), 255);
        }
    }

    free_context(c);
}

// print image summary
void print_image_summary(int width, int height, const unsigned char* data) {
    unsigned long sum = 0;
    unsigned char min = 255, max = 0;

    for (int i = 0; i < width * height; i++) {
        sum += data[i];
        if (data[i] < min) min = data[i];
        if (data[i] > max) max = data[i];
    }

    float avg = (float)sum / (width * height);

    printf("Image Summary (%dx%d):\n", width, height);
    printf("Min value: %d\n", min);
    printf("Max value: %d\n", max);
    printf("Average value: %.2f\n\n", avg);
}

// print a small sample of the data
void print_data_sample(const unsigned char* data, int width, int height, int sample_size) {
    printf("Data sample (first %d values):\n", sample_size);
    for (int i = 0; i < sample_size && i < width * height; i++) {
        printf("%3d ", data[i]);
        if ((i + 1) % 16 == 0) printf("\n");
    }
    printf("\n");
}

// int main(int argc, char **argv) 
// {
//     uint8_t *src_data[4], *dst_data[4];
//     int src_linesize[4], dst_linesize[4];
//     int src_w = 320, src_h = 240, dst_w, dst_h;
//     const char *dst_size = NULL;
//     const char *dst_filename = NULL;
//     FILE *dst_file;
//     int dst_bufsize;
//     struct SwsContext *sws_ctx;
// }
int main(int argc, char *argv[]) {
    if (argc != 7) {
        printf("Usage: %s <input_file> <input_width> <input_height> <output_file> <output_width> <output_height>\n", argv[0]);
        return 1;
    }

    const char *input_file = argv[1];
    int input_width = atoi(argv[2]);
    int input_height = atoi(argv[3]);
    const char *output_file = argv[4];
    int output_width = atoi(argv[5]);
    int output_height = atoi(argv[6]);

    // Allocate memory for input and output
    unsigned char* input = (unsigned char*)malloc(input_width * input_height * sizeof(unsigned char));
    unsigned char* output = (unsigned char*)malloc(output_width * output_height * sizeof(unsigned char));

    if (!input || !output) {
        printf("Memory allocation failed\n");
        return 1;
    }

    FILE *fp = fopen(input_file, "r");
    if (!fp) {
        printf("Failed to open input file: %s\n", input_file);
        free(input);
        free(output);
        return 1;
    }

    int value;
    size_t read_count = 0;
    while (read_count < input_width * input_height && fscanf(fp, "%d", &value) == 1) {
        input[read_count] = (unsigned char)value;
        read_count++;
    }
    fclose(fp);

    if (read_count != input_width * input_height) {
        printf("Failed to read the entire input. Expected %d values, read %zu values.\n", 
            input_width * input_height, read_count);
        free(input);
        free(output);
        return 1;
    }

    printf("Original image:\n");
    print_image_summary(input_width, input_height, input);
    print_data_sample(input, input_width, input_height, 64);

    resize_image(input, output, input_width, input_height, output_width, output_height);

    printf("Resized image:\n");
    print_image_summary(output_width, output_height, output);
    print_data_sample(output, output_width, output_height, 64);

    // Write output file as plain text
    fp = fopen(output_file, "w");
    if (!fp) {
        printf("Failed to open output file: %s\n", output_file);
        free(input);
        free(output);
        return 1;
    }

    for (int i = 0; i < output_height; i++) {
        for (int j = 0; j < output_width; j++) {
            fprintf(fp, "%d ", output[i * output_width + j]);
        }
        fprintf(fp, "\n");
    }
    fclose(fp);

    printf("Resized image written to %s\n", output_file);

    // Free allocated memory
    free(input);
    free(output);

    return 0;
}