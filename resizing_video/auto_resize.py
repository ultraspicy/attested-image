import time
import random
from PIL import Image
import numpy as np
import math
import cv2
def bilinear_resize(image, edited_image, height, width):
  """
  `image` is a 2-D numpy array
  `height` and `width` are the desired spatial dimension of the new 2-D array.
  """
  img_height, img_width = image.shape[:2]

  resized = np.empty([height, width])

  x_ratio = float(img_width - 1) / (width - 1) if width > 1 else 0
  y_ratio = float(img_height - 1) / (height - 1) if height > 1 else 0

  for i in range(height):
    for j in range(width):

      x_l, y_l = math.floor(x_ratio * j), math.floor(y_ratio * i)
      x_h, y_h = math.ceil(x_ratio * j), math.ceil(y_ratio * i)
      x_h = min(x_h, img_width - 1)
      y_h = min(y_h, img_height - 1)
      x_l = min(x_l, img_width - 1)
      y_l = min(y_l, img_height - 1)
      x_weight = (x_ratio * j) - x_l
      y_weight = (y_ratio * i) - y_l
      
      a = image[y_l, x_l]
      b = image[y_l, x_h]
      c = image[y_h, x_l]
      d = image[y_h, x_h]

      pixel = a * (1 - x_weight) * (1 - y_weight) + b * x_weight * (1 - y_weight) + c * y_weight * (1 - x_weight) + d * x_weight * y_weight

      resized[i][j] = pixel
      if (abs(resized[i][j] - edited_image[i][j])>30):
        print(i,j, resized[i][j], edited_image[i][j], resized[i][j] -edited_image[i][j])
  cv2.imwrite('tmp.png', resized)
  cv2.imwrite('tmp2.png', edited_image)
  return resized
    
if __name__ == "__main__":
    image_path = "../python_scripts/extracted_frames_original/frame_0005.jpg"
    changed_path = "../python_scripts/extracted_frames_changed/frame_0005.jpg"
    with Image.open(image_path) as img:
        img = img.convert('RGB')  # Ensure image is in RGB format
        pixels = list(img.getdata())  # Get all pixels from the image
        H_ORIG, W_ORIG = img.size
        print(f"the size of this image, width = {W_ORIG}, height = {H_ORIG}")
        w_r_vals = [[0 for _ in range(W_ORIG)] for _ in range(H_ORIG)]
        w_g_vals = [[0 for _ in range(W_ORIG)] for _ in range(H_ORIG)]
        w_b_vals = [[0 for _ in range(W_ORIG)] for _ in range(H_ORIG)]
        for i in range(H_ORIG):
            for j in range(W_ORIG):
                w_r_vals[i][j] = pixels[i * W_ORIG + j][0]
                w_g_vals[i][j] = pixels[i * W_ORIG + j][1]
                w_b_vals[i][j] = pixels[i * W_ORIG + j][2]
    with Image.open(changed_path) as img:
        img = img.convert('RGB')  # Ensure image is in RGB format
        pixels = list(img.getdata())  # Get all pixels from the image
        H_ORIG, W_ORIG = img.size
        print(f"the size of this image, width = {W_ORIG}, height = {H_ORIG}")
        r_r_vals = [[0 for _ in range(W_ORIG)] for _ in range(H_ORIG)]
        r_g_vals = [[0 for _ in range(W_ORIG)] for _ in range(H_ORIG)]
        r_b_vals = [[0 for _ in range(W_ORIG)] for _ in range(H_ORIG)]
        for i in range(H_ORIG):
            for j in range(W_ORIG):
                r_r_vals[i][j] = pixels[i * W_ORIG + j][0]
                r_g_vals[i][j] = pixels[i * W_ORIG + j][1]
                r_b_vals[i][j] = pixels[i * W_ORIG + j][2]
    edited_image = bilinear_resize(np.array(w_r_vals), np.array(r_r_vals), 540, 960)
    print(edited_image)