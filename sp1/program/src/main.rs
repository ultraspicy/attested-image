//! A simple program to be proven inside the zkVM.

#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let len = 10000;
    
    let mut vec1 = Vec::new();
    for _i in 0..len {
        let pixel = sp1_zkvm::io::read::<u8>();
        vec1.push(pixel as u32)
    }
    let mut vec2 = Vec::new();
    for _i in 0..len {
        let pixel = sp1_zkvm::io::read::<u8>();
        vec2.push(pixel as u32)
    }
    let result: u32 = vec1
        .iter()
        .zip(vec2.iter()) // Combine the two vectors
        .map(|(a, b)| std::cmp::max(a,b) - std::cmp::min(a,b)) 
        .max()
        .expect("No max found");

    // let result: u32 = vec1
    //     .iter()
    //     .zip(vec2.iter()) // Combine the two vectors
    //     .map(|(a, b)| (a - b).pow(2)) // Calculate the difference squared for each pair
    //     .sum();    
    sp1_zkvm::io::write(&result);
}