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

/*
*   convert a 8 bit color depth to 15 bit 
*/
static void hScale8To15(int16_t *dst, int dstW,
                          const uint8_t *src, const int16_t *filter,
                          const int32_t *filterPos, int filterSize) {
    int i;
    for (i = 0; i < dstW; i++) {
        int j;
        int srcPos = filterPos[i];
        int val    = 0;
        for (j = 0; j < filterSize; j++) {
            val += ((int)src[srcPos + j]) * filter[filterSize * i + j];
        }
        dst[i] = FFMIN(val >> 7, (1 << 15) - 1); // the cubic equation does overflow ...
    }
}

// perform bilinear interpolation
float bilinear_interpolate(float q11, float q12, float q21, float q22, float x, float y) {
    float r1 = (1 - x) * q11 + x * q12;
    float r2 = (1 - x) * q21 + x * q22;
    return (1 - y) * r1 + y * r2;
}

void resize_image(const uint8_t* input, uint8_t* output, 
                  int input_width, int input_height, 
                  int output_width, int output_height) {
    int32_t x_ratio = (input_width << 16) / output_width;
    int32_t y_ratio = (input_height << 16) / output_height;

    for (int i = 0; i < output_height; i++) {
        int32_t y = (i * y_ratio) >> 16;
        int32_t y_diff = ((i * y_ratio) & 0xFFFF) >> 1;  // Divide by 2 for 15-bit precision
        for (int j = 0; j < output_width; j++) {
            int32_t x = (j * x_ratio) >> 16;
            int32_t x_diff = ((j * x_ratio) & 0xFFFF) >> 1;  // Divide by 2 for 15-bit precision

            uint8_t q11 = input[FFMIN(y, input_height-1) * input_width + FFMIN(x, input_width-1)];
            uint8_t q12 = input[FFMIN(y, input_height-1) * input_width + FFMIN(x+1, input_width-1)];
            uint8_t q21 = input[FFMIN(y+1, input_height-1) * input_width + FFMIN(x, input_width-1)];
            uint8_t q22 = input[FFMIN(y+1, input_height-1) * input_width + FFMIN(x+1, input_width-1)];

            int32_t result = (q11 * (0x7FFF - x_diff) * (0x7FFF - y_diff) +
                              q12 * x_diff * (0x7FFF - y_diff) +
                              q21 * (0x7FFF - x_diff) * y_diff +
                              q22 * x_diff * y_diff) >> 22;  // Shift by 22 instead of 32

            output[i * output_width + j] = (uint8_t)FFMIN(result, 255);  // Clamp to 8-bit range
        }
    }
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

int main(int argc, char **argv) 
{
    uint8_t *src_data[4], *dst_data[4];
    int src_linesize[4], dst_linesize[4];
    int src_w = 320, src_h = 240, dst_w, dst_h;
    const char *dst_size = NULL;
    const char *dst_filename = NULL;
    FILE *dst_file;
    int dst_bufsize;
    struct SwsContext *sws_ctx;
}
// int main(int argc, char *argv[]) {
//     if (argc != 7) {
//         printf("Usage: %s <input_file> <input_width> <input_height> <output_file> <output_width> <output_height>\n", argv[0]);
//         return 1;
//     }

//     const char *input_file = argv[1];
//     int input_width = atoi(argv[2]);
//     int input_height = atoi(argv[3]);
//     const char *output_file = argv[4];
//     int output_width = atoi(argv[5]);
//     int output_height = atoi(argv[6]);

//     // Allocate memory for input and output
//     unsigned char* input = (unsigned char*)malloc(input_width * input_height * sizeof(unsigned char));
//     unsigned char* output = (unsigned char*)malloc(output_width * output_height * sizeof(unsigned char));

//     if (!input || !output) {
//         printf("Memory allocation failed\n");
//         return 1;
//     }

//     FILE *fp = fopen(input_file, "r");
//     if (!fp) {
//         printf("Failed to open input file: %s\n", input_file);
//         free(input);
//         free(output);
//         return 1;
//     }

//     int value;
//     size_t read_count = 0;
//     while (read_count < input_width * input_height && fscanf(fp, "%d", &value) == 1) {
//         input[read_count] = (unsigned char)value;
//         read_count++;
//     }
//     fclose(fp);

//     if (read_count != input_width * input_height) {
//         printf("Failed to read the entire input. Expected %d values, read %zu values.\n", 
//             input_width * input_height, read_count);
//         free(input);
//         free(output);
//         return 1;
//     }

//     printf("Original image:\n");
//     print_image_summary(input_width, input_height, input);
//     print_data_sample(input, input_width, input_height, 64);

//     resize_image(input, output, input_width, input_height, output_width, output_height);

//     printf("Resized image:\n");
//     print_image_summary(output_width, output_height, output);
//     print_data_sample(output, output_width, output_height, 64);

//     // Write output file as plain text
//     fp = fopen(output_file, "w");
//     if (!fp) {
//         printf("Failed to open output file: %s\n", output_file);
//         free(input);
//         free(output);
//         return 1;
//     }

//     for (int i = 0; i < output_height; i++) {
//         for (int j = 0; j < output_width; j++) {
//             fprintf(fp, "%d ", output[i * output_width + j]);
//         }
//         fprintf(fp, "\n");
//     }
//     fclose(fp);

//     printf("Resized image written to %s\n", output_file);

//     // Free allocated memory
//     free(input);
//     free(output);

//     return 0;
// }