'''
Python script that reads a JPG file, extracts its pixel data, 
and serializes the RGB values of each pixel into a JSON file. 

This script utilizes the Pillow library for image processing and json for serialization.
Before you run the script, make sure to install the Pillow library if you haven’t already:

pip install Pillow
'''
import json
from PIL import Image
import os

def serialize_image_to_json(image_path, output_json_path):

    output_dir = os.path.dirname(output_json_path)
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    # Open the image file
    with Image.open(image_path) as img:
        img = img.convert('RGB')  # Ensure image is in RGB format
        pixels = list(img.getdata())  # Get all pixels from the image
        width, height = img.size
        print(f"the size of this image, width = {width}, height = {height}")
        
        # Organize pixels into a 2D array (list of lists)
        # pixel_data = []
        # for i in range(height):
        #     row = pixels[i * width:(i + 1) * width]
        #     pixel_data.append([{'r': pixel[0], 'g': pixel[1], 'b': pixel[2]} for pixel in row])

        # Create a flat list of pixel RGB values as dictionaries
        pixel_data = [{'r': pixel[0], 'g': pixel[1], 'b': pixel[2]} for pixel in pixels]
    
    # Serialize pixel data to JSON
    with open(output_json_path, 'w') as json_file:
        json.dump(pixel_data, json_file, indent=4)

    print(f"JSON file created at {output_json_path}")

def main():
    import sys
    if len(sys.argv) != 3:
        print("Usage: python serialize_image_to_json.py <path_to_image>.jpg <output>.json")
        sys.exit(1)
    
    image_path = sys.argv[1]
    output_json_path = sys.argv[2]
    serialize_image_to_json(image_path, output_json_path)

# only serialize the first frame of original/edited video for unit test
def unit_test():
    serialize_image_to_json('./extracted_frames_original/frame_0000.jpg', './serialized_frames_original/frame_0000.json' )
    serialize_image_to_json('./extracted_frames_edited/frame_0000.jpg', './serialized_frames_edited/frame_0000.json' )

if __name__ == "__main__":
    # main()

    # test for a single output
    unit_test()