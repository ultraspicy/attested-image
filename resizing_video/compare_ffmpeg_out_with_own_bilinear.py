import numpy as np
import matplotlib.pyplot as plt
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns

def read_array_from_file(filename, width, height):
    with open(filename, 'r') as f:
        data = []
        for line in f:
            data.extend(map(int, line.split()))
    
    data = np.array(data, dtype=np.uint8)
    
    if data.size != width * height:
        raise ValueError(f"Expected {width * height} values, but got {data.size}")
    
    return data.reshape((height, width))

def calculate_psnr(original, processed):
    mse = np.mean((original - processed) ** 2)
    if mse == 0:
        return float('inf')
    max_pixel = 255.0
    psnr = 20 * np.log10(max_pixel / np.sqrt(mse))
    return psnr

def print_array_summary(arr, name, original):
    print(f"Summary for {name}:")
    print(f"Shape: {arr.shape}")
    print(f"Min value: {np.min(arr)}")
    print(f"Max value: {np.max(arr)}")
    print(f"Mean value: {np.mean(arr):.2f}")
    print(f"Median value: {np.median(arr)}")
    print(f"Standard deviation: {np.std(arr):.2f}")
    psnr_value = calculate_psnr(original, arr)
    print(f"PSNR (wrt original): {psnr_value:.2f} dB")
    print()

def compare_arrays(arr1, arr2, name1, name2):
    diff = arr1.astype(np.float32) - arr2.astype(np.float32)
    abs_diff = np.abs(diff)
    
    print(f"Comparison between {name1} and {name2}:")
    print(f"Mean absolute difference: {np.mean(abs_diff):.4f}")
    print(f"Median absolute difference: {np.median(abs_diff):.4f}")
    print(f"Max absolute difference: {np.max(abs_diff):.4f}")
    print(f"Standard deviation of difference: {np.std(diff):.4f}")
    print(f"Percentage of pixels with difference > 0: {np.mean(abs_diff > 0) * 100:.2f}%")
    print(f"Percentage of pixels with difference > 1: {np.mean(abs_diff > 1) * 100:.2f}%")
    print(f"Percentage of pixels with difference > 5: {np.mean(abs_diff > 5) * 100:.2f}%")
    
    # Plot histogram of differences
    plt.figure(figsize=(10, 6))
    plt.hist(diff.flatten(), bins=100, range=(-10, 10))
    plt.title(f"Histogram of Differences ({name1} - {name2})")
    plt.xlabel("Difference")
    plt.ylabel("Frequency")
    plt.savefig(f"difference_histogram_{name1.lower()}_{name2.lower()}.png")
    plt.close()
    
    # Create heatmap of differences
    plt.figure(figsize=(12, 8))
    sns.heatmap(abs_diff, cmap='coolwarm', cbar_kws={'label': 'Absolute Difference'})
    plt.title(f"Heatmap of Differences ({name1} - {name2})")
    plt.xlabel("X-axis")
    plt.ylabel("Y-axis")
    plt.savefig(f"difference_heatmap_{name1.lower()}_{name2.lower()}.png")
    plt.close()
    
    print(f"Histogram of differences saved as 'difference_histogram_{name1.lower()}_{name2.lower()}.png'")
    print(f"Heatmap of differences saved as 'difference_heatmap_{name1.lower()}_{name2.lower()}.png'")
    print()

def main():
    # File paths and dimensions
    original_file = "./sand_19201080_Y_channel.txt"
    original_width, original_height = 1920, 1080
    
    ffmpeg_file = "./sand_480270_Y_channel.txt"
    bilinear_file = "./custom_bilinear_Y.txt"
    processed_width, processed_height = 480, 270 
    
    try:
        # Read arrays
        original_arr = read_array_from_file(original_file, original_width, original_height)
        ffmpeg_arr = read_array_from_file(ffmpeg_file, processed_width, processed_height)
        bilinear_arr = read_array_from_file(bilinear_file, processed_width, processed_height)

        # Resize original array to match processed images
        original_resized = original_arr[::original_height//processed_height, ::original_width//processed_width]

        # Print summaries
        print_array_summary(original_resized, "Original Image (Resized)", original_resized)
        print_array_summary(ffmpeg_arr, "FFmpeg Output", original_resized)
        print_array_summary(bilinear_arr, "Custom Bilinear Output", original_resized)

        # Compare arrays
        compare_arrays(ffmpeg_arr, bilinear_arr, "FFmpeg", "Custom Bilinear")

    except Exception as e:
        print(f"An error occurred: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()