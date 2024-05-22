import json

def generate_json_file(length, filename='./../circuits/diff_square_sum/input_1000.json'):
    # Ensure length is a positive integer
    if length <= 0:
        print("Please provide a positive integer for length.")
        return
    
    # Generate numbers for each list, applying % 256 to each number
    numbers = [str(i % 256) for i in range(1, length + 1)]
    
    # Create a dictionary with two identical lists
    data = {"in1": numbers, "in2": numbers}
    
    # Write the dictionary to a JSON file
    with open(filename, 'w') as file:
        json.dump(data, file, indent=4)
    
    print(f"File '{filename}' has been created with the data.")

# Example usage
generate_json_file(1000)