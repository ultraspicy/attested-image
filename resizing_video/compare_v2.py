import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns

def read_yuv_from_file(filename, width, height):
    with open(filename, 'rb') as f:
        y = np.frombuffer(f.read(width*height), dtype=np.uint8).reshape((height, width))
        cb = np.frombuffer(f.read(width*height//4), dtype=np.uint8).reshape((height//2, width//2))
        cr = np.frombuffer(f.read(width*height//4), dtype=np.uint8).reshape((height//2, width//2))
    return y, cb, cr

def calculate_psnr(original, processed):
    mse = np.mean((original - processed) ** 2)
    if mse == 0:
        return float('inf')
    max_pixel = 255.0
    psnr = 20 * np.log10(max_pixel / np.sqrt(mse))
    return psnr

def print_yuv_summary(y, cb, cr, name, original_y, original_cb, original_cr):
    print(f"Summary for {name}:")
    print(f"Y shape: {y.shape}, Cb shape: {cb.shape}, Cr shape: {cr.shape}")
    print(f"Y - Min: {np.min(y)}, Max: {np.max(y)}, Mean: {np.mean(y):.2f}, Std: {np.std(y):.2f}")
    print(f"Cb - Min: {np.min(cb)}, Max: {np.max(cb)}, Mean: {np.mean(cb):.2f}, Std: {np.std(cb):.2f}")
    print(f"Cr - Min: {np.min(cr)}, Max: {np.max(cr)}, Mean: {np.mean(cr):.2f}, Std: {np.std(cr):.2f}")
    print(f"Y PSNR: {calculate_psnr(original_y, y):.2f} dB")
    print(f"Cb PSNR: {calculate_psnr(original_cb, cb):.2f} dB")
    print(f"Cr PSNR: {calculate_psnr(original_cr, cr):.2f} dB")
    print()

def compare_yuv(y1, cb1, cr1, y2, cb2, cr2, name1, name2):
    def compare_component(comp1, comp2, comp_name):
        diff = comp1.astype(np.float32) - comp2.astype(np.float32)
        abs_diff = np.abs(diff)
        
        print(f"{comp_name} - Mean abs diff: {np.mean(abs_diff):.4f}, Max abs diff: {np.max(abs_diff):.4f}")
        print(f"{comp_name} - % pixels with diff > 0: {np.mean(abs_diff > 0) * 100:.2f}%")
        print(f"{comp_name} - % pixels with diff > 1: {np.mean(abs_diff > 1) * 100:.2f}%")
        
        plt.figure(figsize=(10, 6))
        plt.hist(diff.flatten(), bins=100, range=(-10, 10))
        plt.title(f"{comp_name} Difference Histogram ({name1} - {name2})")
        plt.xlabel("Difference")
        plt.ylabel("Frequency")
        plt.savefig(f"difference_histogram_{comp_name.lower()}_{name1.lower()}_{name2.lower()}.png")
        plt.close()
        
        plt.figure(figsize=(12, 8))
        sns.heatmap(abs_diff, cmap='coolwarm', cbar_kws={'label': 'Absolute Difference'})
        plt.title(f"{comp_name} Difference Heatmap ({name1} - {name2})")
        plt.savefig(f"difference_heatmap_{comp_name.lower()}_{name1.lower()}_{name2.lower()}.png")
        plt.close()

    print(f"Comparison between {name1} and {name2}:")
    compare_component(y1, y2, "Y")
    compare_component(cb1, cb2, "Cb")
    compare_component(cr1, cr2, "Cr")
    print()

def main():
    original_file = "./sand_19201080.yuv"
    ffmpeg_file = "./sand_480270.yuv"
    bilinear_file = "./custom_bilinear_480x270.yuv"
    
    original_width, original_height = 1920, 1080
    processed_width, processed_height = 480, 270

    try:
        original_y, original_cb, original_cr = read_yuv_from_file(original_file, original_width, original_height)
        ffmpeg_y, ffmpeg_cb, ffmpeg_cr = read_yuv_from_file(ffmpeg_file, processed_width, processed_height)
        bilinear_y, bilinear_cb, bilinear_cr = read_yuv_from_file(bilinear_file, processed_width, processed_height)

        # Resize original to match processed images
        original_y_resized = original_y[::original_height//processed_height, ::original_width//processed_width]
        original_cb_resized = original_cb[::original_height//processed_height//2, ::original_width//processed_width//2]
        original_cr_resized = original_cr[::original_height//processed_height//2, ::original_width//processed_width//2]

        print_yuv_summary(original_y_resized, original_cb_resized, original_cr_resized, "Original (Resized)", 
                          original_y_resized, original_cb_resized, original_cr_resized)
        print_yuv_summary(ffmpeg_y, ffmpeg_cb, ffmpeg_cr, "FFmpeg Output", 
                          original_y_resized, original_cb_resized, original_cr_resized)
        print_yuv_summary(bilinear_y, bilinear_cb, bilinear_cr, "Custom Bilinear Output", 
                          original_y_resized, original_cb_resized, original_cr_resized)

        compare_yuv(ffmpeg_y, ffmpeg_cb, ffmpeg_cr, bilinear_y, bilinear_cb, bilinear_cr, "FFmpeg", "Custom Bilinear")

    except Exception as e:
        print(f"An error occurred: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()