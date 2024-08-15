import numpy as np
from PIL import Image
import sys

def save_yuv_channels(image_path):
    # Open the image
    img = Image.open(image_path)
    
    # Convert the image to YCbCr mode
    img_yuv = img.convert('YCbCr')
    
    # Convert the image to a numpy array
    img_array = np.array(img_yuv)
    
    # Separate the channels
    y_channel = img_array[:,:,0]
    u_channel = img_array[:,:,1]
    v_channel = img_array[:,:,2]
    
    # Generate output file names based on input file name
    base_name = image_path.rsplit('.', 1)[0]
    y_file = f'{base_name}_Y_channel.txt'
    u_file = f'{base_name}_U_channel.txt'
    v_file = f'{base_name}_V_channel.txt'
    
    # Save each channel as a separate file
    np.savetxt(y_file, y_channel, fmt='%d')
    np.savetxt(u_file, u_channel, fmt='%d')
    np.savetxt(v_file, v_channel, fmt='%d')
    
    print(f"YUV channel files have been saved for {image_path}.")

    # Print some statistics about each channel
    print(f"Y channel - Min: {y_channel.min()}, Max: {y_channel.max()}, Mean: {y_channel.mean():.2f}")
    print(f"U channel - Min: {u_channel.min()}, Max: {u_channel.max()}, Mean: {u_channel.mean():.2f}")
    print(f"V channel - Min: {v_channel.min()}, Max: {v_channel.max()}, Mean: {v_channel.mean():.2f}")

# Check if a file name was provided as a command-line argument
if len(sys.argv) > 1:
    image_path = sys.argv[1]
    save_yuv_channels(image_path)
else:
    print("Please provide an image file name as a command-line argument.")
    print("Usage: python script_name.py <image_file_name>")