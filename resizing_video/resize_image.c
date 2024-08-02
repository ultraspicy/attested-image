#include <stdio.h>
#include <stdlib.h>

// perform bilinear interpolation
float bilinear_interpolate(float q11, float q12, float q21, float q22, float x, float y) {
    float r1 = (1 - x) * q11 + x * q12;
    float r2 = (1 - x) * q21 + x * q22;
    return (1 - y) * r1 + y * r2;
}

// resize the image
void resize_image(const unsigned char* input, unsigned char* output, 
                  int input_width, int input_height, 
                  int output_width, int output_height) {
    float x_ratio = (float)(input_width - 1) / (output_width - 1);
    float y_ratio = (float)(input_height - 1) / (output_height - 1);

    for (int i = 0; i < output_height; i++) {
        for (int j = 0; j < output_width; j++) {
            float x = j * x_ratio;
            float y = i * y_ratio;
            int x_floor = (int)x;
            int y_floor = (int)y;
            float x_diff = x - x_floor;
            float y_diff = y - y_floor;

            if (x_floor == input_width - 1 || y_floor == input_height - 1) {
                output[i * output_width + j] = input[y_floor * input_width + x_floor];
            } else {
                float q11 = input[y_floor * input_width + x_floor];
                float q12 = input[y_floor * input_width + (x_floor + 1)];
                float q21 = input[(y_floor + 1) * input_width + x_floor];
                float q22 = input[(y_floor + 1) * input_width + (x_floor + 1)];

                float result = bilinear_interpolate(q11, q12, q21, q22, x_diff, y_diff);
                output[i * output_width + j] = (unsigned char)(result + 0.5);
            }
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