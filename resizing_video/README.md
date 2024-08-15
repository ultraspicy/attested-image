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

To compile from source, cd into ffmpeg fold and
`brew install autoconf automake cmake git libtool pkg-config texi2html yasm nasm` 
`./configure --enable-debug=3 --disable-optimizations --disable-stripping`

`make -j$(nproc)`
`sudo make install`