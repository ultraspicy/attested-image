import psutil
import subprocess
import numpy as np
from math import sqrt
import random
import subprocess
import re
import json
import os

def measure_command(command, time = True, memory = True):
    """
    Measure the time and memory usage of a specified command.

    :param command: The command to execute and measure.
    :param time: True if you want to measure time, False otherwise.
    :param memory: True if you want to measure memory usage, False otherwise.

    :return: A tuple containing the elapsed time (if time=True) and memory usage (if memory=True).
    """
    command = f'/usr/bin/time -p -f "%e %M" {command} > /dev/null'
    if memory:
        init_swap_used = psutil.swap_memory().used
        max_swap_used = init_swap_used
    
    process = subprocess.Popen(command, 
                               shell=True,
                               stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE)
    
    if memory:
        while process.poll() is None:
            max_swap_used = max(max_swap_used, psutil.swap_memory().used)

    command_output = process.communicate()[1].decode('utf-8')
    t,mem = command_output.split('\n')[0].split(' ')
    t = float(t)

    if memory:
        swap = (max_swap_used-init_swap_used)/1024
        m = float(mem)+(swap if swap > 0 else 0)
    
    return t if time else None, m if memory else None

def extract_contraints(r1cs_file):
    infos = subprocess.check_output(f'snarkjs r1cs info {r1cs_file}',shell=True).decode('utf-8')
    return int(re.search(r'# of Constraints: (\d+)',infos).group(1))

def generate_circuit(info, circuit_template, id = None):
    """
    Generate a circuit from a template
    :param info: dictionary with the information to replace in the template
    :param circuit_template: path to the template
    :param id: id of the circuit

    """
    out_circuit = circuit_template.split('/')[-1].split('.')[0]
    os.makedirs('circuits/benchmark',exist_ok=True)

    with open(circuit_template, 'r') as infile:
        circuit = infile.read()
        for k,v in info.items():
            circuit = circuit.replace(k, str(v))
        circuit = circuit.replace('//MAIN', '')
        
        id = f'_{id}' if id is not None else ''
        out_path = f'circuits/benchmark/{out_circuit}{id}.circom'
        with open(out_path, 'w') as outfile:
            outfile.write(circuit)
    return out_path

def generate_input(output_path, size):
    """
    Generate a random input for a circuit of a given size
    :param output_path: path to the output file
    :param size: size of the input
    """
    json_input = {'in':[str(random.randint(0, 255)) for _ in range(size)] }
    os.makedirs('input',exist_ok=True)
    with open(output_path, 'w') as outfile:
        json.dump(json_input, outfile)