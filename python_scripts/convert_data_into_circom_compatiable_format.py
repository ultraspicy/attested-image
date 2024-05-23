'''
the output from serialize_image_to_json.py will output data in the following format
{
    [
        {
            r: xxx,
            g: xxx,
            b: xxx
        },
        {
            r: yyy,
            g: yyy,
            b: yyy
        }
    ]
}

circom circuit expect input is as follows
{
    "in1": [
        "1",
        "2",
        "3"
        ],
    "in2": [
        "1",
        "2",
        "3",
        ]
}   

this file will take two file/snippet from deserilized image, and use three
thread to process R, G, B channel respectively to generate a series of proof

The first step is convert the RGB json into the format that will be used in ZKP later
The second step is to run the ZKP and collect all proof
'''
import json 
import os

def convert_rgb_jsons_to_circom_input(rgb_json_path1, rgb_json_path2, output_json_dir, file_descriptor_prefix):
    # Read the first RGB JSON file
    with open(rgb_json_path1, 'r') as file:
        rgb_data1 = json.load(file)
    
    # Read the second RGB JSON file
    with open(rgb_json_path2, 'r') as file:
        rgb_data2 = json.load(file)
    
    # Prepare the new structure for Circom input
    circom_input_r = {"in1": [], "in2": []}
    circom_input_g = {"in1": [], "in2": []}
    circom_input_b = {"in1": [], "in2": []}
    
    # Convert the first RGB data to Circom's expected format and add to in1
    for pixel in rgb_data1:
        circom_input_r["in1"].append(str(pixel['r']))
        circom_input_g["in1"].append(str(pixel['g']))
        circom_input_b["in1"].append(str(pixel['b']))
    
    # Convert the second RGB data to Circom's expected format and add to in2
    for pixel in rgb_data2:
        circom_input_r["in2"].append(str(pixel['r']))
        circom_input_g["in2"].append(str(pixel['g']))
        circom_input_b["in2"].append(str(pixel['b']))

    # Ensure the output directory exists
    if not os.path.exists(output_json_dir):
        os.makedirs(output_json_dir)

    # Write the converted data to separate JSON files
    output_r_path = os.path.join(output_json_dir, f'{file_descriptor_prefix}_r.json')
    output_g_path = os.path.join(output_json_dir, f'{file_descriptor_prefix}_g.json')
    output_b_path = os.path.join(output_json_dir, f'{file_descriptor_prefix}_b.json')
    
    with open(output_r_path, 'w') as file:
        json.dump(circom_input_r, file, indent=4)
    
    with open(output_g_path, 'w') as file:
        json.dump(circom_input_g, file, indent=4)
    
    with open(output_b_path, 'w') as file:
        json.dump(circom_input_b, file, indent=4)
    
    print(f"Converted JSONs saved to {output_r_path}, {output_g_path}, {output_b_path}")

def extract_json_snippet(input_json_path, output_json_path, start, end):
    # Read the input JSON file
    with open(input_json_path, 'r') as file:
        data = json.load(file)
    
    # Extract the snippet
    snippet = {
        "in1": data["in1"][start:end],
        "in2": data["in2"][start:end]
    }
    
    # Ensure the output directory exists
    output_dir = os.path.dirname(output_json_path)
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    # Write the snippet to the output JSON file
    with open(output_json_path, 'w') as file:
        json.dump(snippet, file, indent=4)
    
    # Print the new lengths of in1 and in2
    print(f"Snippet length of in1: {len(snippet['in1'])}")
    print(f"Snippet length of in2: {len(snippet['in2'])}")
    
    print(f"Snippet saved to {output_json_path}")

# only take the first row of the first frame of the video
def unit_test():

    convert_rgb_jsons_to_circom_input(
        './serialized_frames_original/frame_0000.json', 
        './serialized_frames_edited/frame_0000.json',
        './circuit_input',
        'frame_0000') 
    
    extract_json_snippet('./circuit_input/frame_0000_r.json',
                         './circuit_input/frame_0000_r_snippet.json',
                         0, 1024)


if __name__ == "__main__":
    # main()

    # test for a single output
    unit_test()