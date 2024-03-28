use rand::{distributions::Uniform, Rng};
use std::fs::File;
use std::io::{self, Write};

fn generate_random_data_file(length: usize, file_path: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;

    // Create a random number generator
    let mut rng = rand::thread_rng();
    let die = Uniform::from(0..=255);

    // Generate and write the random u8 numbers to the file
    for _ in 0..length {
        let number: u8 = rng.sample(die);
        file.write_all(&[number])?;
    }

    Ok(())
}


// assuming for picture of 1024 * 1024, the input size would be around 3M u8 numbers
// simulating this by generate random <length> number 
fn main() -> io::Result<()> {
    let length = 3000000;
    let file_path = "random_u8_numbers.dat";

    generate_random_data_file(length, file_path)?;

    println!("Generated file with {} random bytes at '{}'", length, file_path);
    Ok(())
}

// fn generate_pixel_slice(len: usize) -> Vec<Pixel> {
//     (0..len).map(|_| generate_random_pixel()).collect()
// }