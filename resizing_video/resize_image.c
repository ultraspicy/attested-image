#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <libavutil/imgutils.h>
#include <libavutil/parseutils.h>
#include <libswscale/swscale.h>
#include <math.h>

#define FFMIN(a,b) ((a) < (b) ? (a) : (b))
#define FFMAX(a,b) ((a) > (b) ? (a) : (b))
#define SWS_FAST_BILINEAR     1
#define SWS_BILINEAR          2
#define SWS_BICUBIC           4

typedef struct {
    int *filterPos;
    int16_t *filter;
    int filterSize;
    int *xInc;
} SwsContext;

/*
*   convert a 8 bit color depth to 15 bit 
*/
// static void hScale8To15(int16_t *dst, int dstW,
//                           const uint8_t *src, const int16_t *filter,
//                           const int32_t *filterPos, int filterSize) {
//     int i;
//     for (i = 0; i < dstW; i++) {
//         int j;
//         int srcPos = filterPos[i];
//         int val    = 0;
//         for (j = 0; j < filterSize; j++) {
//             val += ((int)src[srcPos + j]) * filter[filterSize * i + j];
//         }
//         dst[i] = FFMIN(val >> 7, (1 << 15) - 1); // the cubic equation does overflow ...
//     }
// }


typedef struct {
    int *filterPos;
    int16_t *filter;
    int filterSize;
    int *xInc;
} CustomSwsContext;

static CustomSwsContext* custom_sws_alloc_context() {
    CustomSwsContext *c = (CustomSwsContext*)calloc(1, sizeof(CustomSwsContext));
    return c;
}

static void custom_sws_freeContext(CustomSwsContext *c) {
    if (c) {
        free(c->filterPos);
        free(c->filter);
        free(c->xInc);
        free(c);
    }
}

static int custom_initFilter(CustomSwsContext *c, int srcW, int dstW, int filterSize) {
    int i, j;
    int fone = 1 << 14;
    c->filterPos = (int*)malloc(dstW * sizeof(int));
    c->filter = (int16_t*)malloc(dstW * filterSize * sizeof(int16_t));
    c->filterSize = filterSize;

    if (!c->filterPos || !c->filter)
        return -1;

    double scale = (double)srcW / dstW;

    for (i = 0; i < dstW; i++) {
        double center = (i + 0.5) * scale - 0.5;
        int left = (int)center;
        c->filterPos[i] = left;

        for (j = 0; j < filterSize; j++) {
            double d = fabs(center - (left + j));
            int coeff = (int)((1.0 - d) * fone + 0.5);
            if (coeff < 0) coeff = 0;
            c->filter[i * filterSize + j] = coeff;
        }

        int sum = 0;
        for (j = 0; j < filterSize; j++)
            sum += c->filter[i * filterSize + j];
        
        if (sum != fone) {
            for (j = 0; j < filterSize; j++) {
                int corr = ((int64_t)c->filter[i * filterSize + j] * fone) / sum;
                c->filter[i * filterSize + j] = (int16_t)corr;
            }
        }
    }

    return 0;
}

void resize_image(const uint8_t* input, uint8_t* output, 
                  int input_width, int input_height, 
                  int output_width, int output_height) {
    CustomSwsContext *c = custom_sws_alloc_context();
    int filterSize = 2;  // for bilinear
    
    if (custom_initFilter(c, input_width, output_width, filterSize) < 0) {
        custom_sws_freeContext(c);
        return;
    }

    double scale_y = (double)input_height / output_height;

    for (int i = 0; i < output_height; i++) {
        double sy = i * scale_y;
        int sy_floor = (int)sy;
        int vy = (int)((sy - sy_floor) * (1 << 14));

        for (int j = 0; j < output_width; j++) {
            int sx = c->filterPos[j];
            int16_t *filter = &c->filter[j * filterSize];

            int val = 0;
            for (int y = 0; y < 2; y++) {
                int src_y = FFMIN(sy_floor + y, input_height - 1);
                int vy_weight = y == 0 ? (1 << 14) - vy : vy;
                for (int x = 0; x < 2; x++) {
                    int src_x = FFMIN(sx + x, input_width - 1);
                    int pos = src_y * input_width + src_x;
                    val += input[pos] * filter[x] * vy_weight;
                }
            }

            val = (val + (1 << 27)) >> 28;
            output[i * output_width + j] = FFMIN(FFMAX(val, 0), 255);
        }
    }

    custom_sws_freeContext(c);
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