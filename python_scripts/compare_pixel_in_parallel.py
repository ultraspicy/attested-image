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
