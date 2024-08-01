#include <stdio.h>
#include <stdlib.h>

#define INPUT_SIZE 4
#define OUTPUT_SIZE 3

// Function to perform bilinear interpolation
float bilinear_interpolate(float q11, float q12, float q21, float q22, float x, float y) {
    float r1 = (1 - x) * q11 + x * q12;
    float r2 = (1 - x) * q21 + x * q22;
    return (1 - y) * r1 + y * r2;
}

// Function to resize the image
void resize_image(int input[INPUT_SIZE][INPUT_SIZE], int output[OUTPUT_SIZE][OUTPUT_SIZE]) {
    float x_ratio = (float)(INPUT_SIZE - 1) / (OUTPUT_SIZE - 1);
    float y_ratio = (float)(INPUT_SIZE - 1) / (OUTPUT_SIZE - 1);

    for (int i = 0; i < OUTPUT_SIZE; i++) {
        for (int j = 0; j < OUTPUT_SIZE; j++) {
            float x = j * x_ratio;
            float y = i * y_ratio;
            int x_floor = (int)x;
            int y_floor = (int)y;
            float x_diff = x - x_floor;
            float y_diff = y - y_floor;

            if (x_floor == INPUT_SIZE - 1 || y_floor == INPUT_SIZE - 1) {
                output[i][j] = input[y_floor][x_floor];
            } else {
                float q11 = input[y_floor][x_floor];
                float q12 = input[y_floor][x_floor + 1];
                float q21 = input[y_floor + 1][x_floor];
                float q22 = input[y_floor + 1][x_floor + 1];

                output[i][j] = (int)(bilinear_interpolate(q11, q12, q21, q22, x_diff, y_diff) + 0.5);
            }
        }
    }
}

// Function to print a matrix
void print_matrix(int size, int matrix[size][size]) {
    for (int i = 0; i < size; i++) {
        for (int j = 0; j < size; j++) {
            printf("%3d ", matrix[i][j]);
        }
        printf("\n");
    }
    printf("\n");
}

int main() {
    int input[INPUT_SIZE][INPUT_SIZE] = {
        {10,  20,  30,  40},
        {50,  60,  70,  80},
        {90, 100, 110, 120},
        {130, 140, 150, 160}
    };

    int output[OUTPUT_SIZE][OUTPUT_SIZE];

    printf("Original 4x4 image:\n");
    print_matrix(INPUT_SIZE, input);

    resize_image(input, output);

    printf("Resized 3x3 image:\n");
    print_matrix(OUTPUT_SIZE, output);

    return 0;
}