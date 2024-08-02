import numpy as np
from PIL import Image
import sys

def save_rgb_channels(image_path):
    # Open the image
    img = Image.open(image_path)
    
    # Convert the image to RGB mode if it's not already
    img = img.convert('RGB')
    
    # Convert the image to a numpy array
    img_array = np.array(img)
    
    # Separate the channels
    r_channel = img_array[:,:,0]
    g_channel = img_array[:,:,1]
    b_channel = img_array[:,:,2]
    
    # Generate output file names based on input file name
    base_name = image_path.rsplit('.', 1)[0]
    r_file = f'{base_name}_R_channel.txt'
    g_file = f'{base_name}_G_channel.txt'
    b_file = f'{base_name}_B_channel.txt'
    
    # Save each channel as a separate file
    np.savetxt(r_file, r_channel, fmt='%d')
    np.savetxt(g_file, g_channel, fmt='%d')
    np.savetxt(b_file, b_channel, fmt='%d')
    
    print(f"Channel files have been saved for {image_path}.")

# Check if a file name was provided as a command-line argument
if len(sys.argv) > 1:
    image_path = sys.argv[1]
    save_rgb_channels(image_path)
else:
    print("Please provide an image file name as a command-line argument.")
    print("Usage: python script_name.py <image_file_name>")