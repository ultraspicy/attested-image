sand_19201080.jpg, this is a sample frame that we will experiment on. Convert it to RGB
```
python3 convert_image_to_rgb.py sand_19201080.jpg
```

Resize the image using ffmpeg, which outputs pic1
```
ffmpeg -i sand_19201080.jpg -vf scale=480:270:flags=bilinear sand_480270.jpg 
```
and then convert pic1 (jpg file) into 3 plain text file, storing the RGB channel data
```
python3 convert_image_to_rgb.py sand_480270.jpg
```


Resize the imagine using our own code
```
gcc -o resize_image resize_image.c
./resize_image sand_19201080_R_channel.txt 1920 1080 custom_bilinear_r.txt 480 270
```
this outuputs `custom_bilinear_r.txt`, do the same for G and B channel



To compare how close our own bilinear filter is with pic1


A experience result is as-follows
```
Summary for Original Image (Resized):
Shape: (270, 480)
Min value: 10
Max value: 255
Mean value: 156.85
Median value: 172.0
Standard deviation: 71.02
PSNR (wrt original): inf dB

Summary for FFmpeg Output:
Shape: (270, 480)
Min value: 14
Max value: 255
Mean value: 156.96
Median value: 173.0
Standard deviation: 70.84
PSNR (wrt original): 40.23 dB

Summary for Custom Bilinear Output:
Shape: (270, 480)
Min value: 10
Max value: 255
Mean value: 156.99
Median value: 173.0
Standard deviation: 71.01
PSNR (wrt original): 42.98 dB

Comparison between FFmpeg and Custom Bilinear:
Mean absolute difference: 1.5314
Median absolute difference: 1.0000
Max absolute difference: 56.0000
Standard deviation of difference: 2.3709
Percentage of pixels with difference > 0: 73.63%
Percentage of pixels with difference > 1: 38.39%
Percentage of pixels with difference > 5: 2.40%
```