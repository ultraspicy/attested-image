// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use fibonacci_lib::{fibonacci, PublicValuesStruct, print_data_sample, print_image_summary, resize_image};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

// pub fn main() {
//     // Read an input to the program.
//     //
//     // Behind the scenes, this compiles down to a custom system call which handles reading inputs
//     // from the prover.
//     let n = sp1_zkvm::io::read::<u32>();

//     // Compute the n'th fibonacci number using a function from the workspace lib crate.
//     let (a, b) = fibonacci(n);

//     // Encode the public values of the program.
//     let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct { n, a, b });

//     // Commit to the public values of the program. The final proof will have a commitment to all the
//     // bytes that were committed to.
//     sp1_zkvm::io::commit_slice(&bytes);
// }

const FILTER_BITS: i32 = 14;
const FILTER_SCALE: i32 = 1 << FILTER_BITS;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 7 {
        println!("Usage: {} <input_file> <input_width> <input_height> <output_file> <output_width> <output_height>", args[0]);
        return;
    }

    let input_file = &args[1];
    let input_width = args[2].parse::<i32>().unwrap();
    let input_height = args[3].parse::<i32>().unwrap();
    let output_file = &args[4];
    let output_width = args[5].parse::<i32>().unwrap();
    let output_height = args[6].parse::<i32>().unwrap();

    let mut input: Vec<u8> = Vec::new(); 
    let mut output = vec![0u8; (output_width * output_height) as usize];

    let input_path = Path::new(input_file);
    let file = File::open(&input_path).expect("Failed to open input file");
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        match line {
            Ok(line_data) => {
                // Split the line into separate numbers
                for value_str in line_data.split_whitespace() {
                    match value_str.parse::<u8>() {
                        Ok(value) => {
                            input.push(value);
                        }
                        Err(_) => {
                            println!("Error parsing value on line {}: '{}'", i + 1, value_str);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error reading line {}: {:?}", i + 1, e);
            }
        }
    }

    println!("Original image:");
    print_image_summary(input_width as usize, input_height as usize, &input);
    print_data_sample(&input, 64);

    resize_image(&input, &mut output, input_width, input_height, output_width, output_height);

    println!("Resized image:");
    print_image_summary(output_width as usize, output_height as usize, &output);
    print_data_sample(&output, 64);

    let output_path = Path::new(output_file);
    let mut output_file = File::create(&output_path).expect("Failed to open output file");

    for i in 0..output_height {
        for j in 0..output_width {
            write!(output_file, "{} ", output[(i * output_width + j) as usize]).unwrap();
        }
        writeln!(output_file).unwrap();
    }

    println!("Resized image written to {}", output_path.display());
}