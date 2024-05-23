import subprocess

# def run_setup_circuit_params():
#     try:
#         # Run the shell script
#         result = subprocess.run(['./setup_circuit_params.sh'], check=True, capture_output=True, text=True)

#         # Print the standard output and standard error
#         print("Standard Output:")
#         print(result.stdout)
#         print("Standard Error:")
#         print(result.stderr)

#     except subprocess.CalledProcessError as e:
#         print(f"An error occurred while running the script: {e}")
#         print(f"Return Code: {e.returncode}")
#         print(f"Output: {e.output}")
#         print(f"Error: {e.stderr}")

# # Run the function
# run_setup_circuit_params()