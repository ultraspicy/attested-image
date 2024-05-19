Pipeline

Expected Input: 2 videos
Expected Output: true/flase on whether the second video

Step1:
From original video, edit video. Decompose the video into a list of frame

Step2:
Compare per frame, the diff_square_sum should be less than the threshold

Step3:
use all proof to compute the final output