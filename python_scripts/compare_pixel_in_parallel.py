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

def convert_rgb_jsons_to_circom_input(rgb_json_path1, rgb_json_path2, output_json_path):
    # Read the first RGB JSON file
    with open(rgb_json_path1, 'r') as file:
        rgb_data1 = json.load(file)
    
    # Read the second RGB JSON file
    with open(rgb_json_path2, 'r') as file:
        rgb_data2 = json.load(file)
    
    # Prepare the new structure for Circom input
    circom_input = {"in1": [], "in2": []}
    
    # Convert the first RGB data to Circom's expected format and add to in1
    for pixel in rgb_data1:
        circom_input["in1"].append(str(pixel['r']))
        circom_input["in1"].append(str(pixel['g']))
        circom_input["in1"].append(str(pixel['b']))
    
    # Convert the second RGB data to Circom's expected format and add to in2
    for pixel in rgb_data2:
        circom_input["in2"].append(str(pixel['r']))
        circom_input["in2"].append(str(pixel['g']))
        circom_input["in2"].append(str(pixel['b']))

    # Write the converted data to a new JSON file
    with open(output_json_path, 'w') as file:
        json.dump(circom_input, file, indent=4)
    
    print(f"Converted JSON saved to {output_json_path}")