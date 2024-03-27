// #[derive(Debug)]
// struct Pixel {
//     R: u8,
//     G: u8,
//     B: u8,
// }

// fn generate_random_pixel() -> Pixel {
//     // Rust infers the type of the value that rng.gen() should produce 
//     // based on the expected type for the fields R, G, and B of the Pixel struct
//     let mut rng = rand::thread_rng();

//     Pixel {
//         R: rng.gen(),
//         G: rng.gen(),
//         B: rng.gen(),
//     }
// }

// fn generate_pixel_slice(len: usize) -> Vec<Pixel> {
//     (0..len).map(|_| generate_random_pixel()).collect()
// }