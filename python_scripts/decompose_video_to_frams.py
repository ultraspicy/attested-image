'''
uses the cv2 module from OpenCV to decompose a video 
into its constituent frames and save them as images

Before run, ensure the OpenCV is installed
pip install opencv-python
'''
import cv2
import os

def decompose_video_to_frames(video_path, output_folder):
    # Ensure the output directory exists
    if not os.path.exists(output_folder):
        os.makedirs(output_folder)
    
    # Load the video
    video = cv2.VideoCapture(video_path)
    
    if not video.isOpened():
        print("Error: Could not open video.")
        return
    
    frame_count = 0
    while True:
        # Read next frame from the video
        ret, frame = video.read()
        
        # Check if frame is read correctly
        if not ret:
            break
        
        # Save the frame as an image file
        frame_path = os.path.join(output_folder, f"frame_{frame_count:04d}.jpg")
        cv2.imwrite(frame_path, frame)
        frame_count += 1
        # print(f"Saved {frame_path}")
    
    # Release the video capture object
    video.release()
    print("Done. Extracted {} frames.".format(frame_count))

# decompose original and edited video into series of pictures
original_video_path = './../resources/original.mp4'  
output_folder = './extracted_frames_original'    
decompose_video_to_frames(original_video_path, output_folder)

edited_video_path = './../resources/edited.mp4'  
output_folder = './extracted_frames_edited'    
decompose_video_to_frames(edited_video_path, output_folder)