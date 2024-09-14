use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::cmp::{min, max};

const FILTER_BITS: i32 = 14;
const FILTER_SCALE: i32 = 1 << FILTER_BITS;

struct Context {
    filter_pos: Vec<i32>,
    filter: Vec<i16>,
    filter_size: usize,
    v_lum_filter_pos: Vec<i32>,
    v_lum_filter: Vec<i16>,
    v_lum_filter_size: usize,
    dst_w: i32,
    dst_h: i32,
    src_w: i32,
    src_h: i32,
}

impl Context {
    fn new(src_w: i32, src_h: i32, dst_w: i32, dst_h: i32) -> Option<Self> {
        let filter_size = 4;
        let mut context = Context {
            filter_pos: Vec::new(),
            filter: Vec::new(),
            filter_size,
            v_lum_filter_pos: Vec::new(),
            v_lum_filter: Vec::new(),
            v_lum_filter_size: filter_size,
            dst_w,
            dst_h,
            src_w,
            src_h,
        };

        if context.init_filter(src_w, dst_w, filter_size).is_err() {
            return None;
        }

        context.init_vfilter(src_h, dst_h, filter_size);
        Some(context)
    }

    fn init_filter(&mut self, src_w: i32, dst_w: i32, filter_size: usize) -> Result<(), ()> {
        let x_inc: i64 = (((src_w as i64) << 16) / dst_w as i64 + 1) >> 1;

        self.filter_pos = vec![0; dst_w as usize];
        self.filter = vec![0; dst_w as usize * filter_size];

        for i in 0..dst_w as usize {
            let src_pos: i64 = (i as i64 * x_inc) >> 15;
            let xx_inc = x_inc & 0xffff;
            let xx = (xx_inc * (1 << FILTER_BITS) / x_inc) as i32;

            self.filter_pos[i] = src_pos as i32;

            for j in 0..filter_size {
                let coeff = if j == 0 {
                    (1 << FILTER_BITS) - xx
                } else {
                    xx
                };
                self.filter[i * filter_size + j] = coeff as i16;
            }

            let mut sum = 0;
            for j in 0..filter_size {
                sum += self.filter[i * filter_size + j] as i64;
            }

            if sum != FILTER_SCALE as i64 {
                for j in 0..filter_size {
                    let coeff = (self.filter[i * filter_size + j] as i64 * FILTER_SCALE as i64) / sum;
                    self.filter[i * filter_size + j] = coeff as i16;
                }
            }
        }

        Ok(())
    }

    fn init_vfilter(&mut self, src_h: i32, dst_h: i32, filter_size: usize) {
        self.v_lum_filter_pos = vec![0; dst_h as usize];
        self.v_lum_filter = vec![0; dst_h as usize * filter_size];

        let scale = src_h as f64 / dst_h as f64;

        for i in 0..dst_h as usize {
            let center = (i as f64 + 0.5) * scale - 0.5;
            let top = (center - filter_size as f64 / 2.0).ceil() as i32;

            self.v_lum_filter_pos[i] = top;

            for j in 0..filter_size {
                let weight = if filter_size > 1 {
                    // weight = 1.0 - fabs((j - (center - top)) / (filterSize / 2.0));
                    1.0 - ((j as f64 - (center - top as f64)).abs() / (filter_size as f64 / 2.0))
                } else {
                    1.0
                };
                self.v_lum_filter[i * filter_size + j] = (weight * FILTER_SCALE as f64) as i16;
            }

            // Normalize filter coefficients
            // int sum = 0;
            // for (j = 0; j < filterSize; j++)
            //     sum += c->vLumFilter[i * filterSize + j];
            // for (j = 0; j < filterSize; j++)
            //     c->vLumFilter[i * filterSize + j] = c->vLumFilter[i * filterSize + j] * FILTER_SCALE / sum;
            let sum: i32 = self.v_lum_filter[i * filter_size..(i + 1) * filter_size]
                .iter()
                .map(|&val| val as i32)
                .sum();

            for j in 0..filter_size {
                self.v_lum_filter[i * filter_size + j] =
                    (self.v_lum_filter[i * filter_size + j] as i32 * FILTER_SCALE / sum) as i16;
            }
        }
    }
}

fn scale_image(
    c: &Context,
    src: &[u8],
    src_stride: i32,
    dst: &mut [u8],
    dst_stride: i32,
) {
    let mut tmp = vec![0u8; c.dst_w as usize * c.src_h as usize];

    // Horizontal scaling
    for y in 0..c.src_h as usize {
        for x in 0..c.dst_w as usize {
            let src_pos = c.filter_pos[x];
            let mut val = 0;

            for z in 0..c.filter_size {
                if src_pos + (z as i32) < c.src_w {
                    val += src[y * src_stride as usize + (src_pos as usize + z)] as u32
                        * c.filter[x * c.filter_size + z] as u32;
                }
            }

            tmp[y * c.dst_w as usize + x] = ((val + (1 << (FILTER_BITS - 1))) >> FILTER_BITS) as u8;
        }
    }

    // Vertical scaling
    for y in 0..c.dst_h as usize {
        for x in 0..c.dst_w as usize {
            let src_pos = c.v_lum_filter_pos[y];
            let mut val = 0;

            for z in 0..c.v_lum_filter_size {
                if src_pos + (z as i32) < c.src_h {
                    val += tmp[((src_pos + z as i32) as usize) * c.dst_w as usize + x] as u32
                        * c.v_lum_filter[y * c.v_lum_filter_size + z] as u32;
                }
            }

            dst[y * dst_stride as usize + x] = ((val + (1 << (FILTER_BITS - 1))) >> FILTER_BITS) as u8;
        }
    }
}

fn resize_image(
    input: &[u8],
    output: &mut [u8],
    input_width: i32,
    input_height: i32,
    output_width: i32,
    output_height: i32,
) {
    if let Some(c) = Context::new(input_width, input_height, output_width, output_height) {
        scale_image(&c, input, input_width, output, output_width);
    } else {
        eprintln!("Failed to initialize Context");
    }
}

fn print_image_summary(width: usize, height: usize, data: &[u8]) {
    let mut sum: u64 = 0;
    let mut min_val: u8 = 255;
    let mut max_val: u8 = 0;

    for &pixel in data.iter() {
        sum += pixel as u64;
        min_val = min(min_val, pixel);
        max_val = max(max_val, pixel);
    }

    let avg = sum as f64 / (width * height) as f64;

    println!("Image Summary ({}x{}):", width, height);
    println!("Min value: {}", min_val);
    println!("Max value: {}", max_val);
    println!("Average value: {:.2}", avg);
}

fn print_data_sample(data: &[u8], sample_size: usize) {
    println!("Data sample (first {} values):", sample_size);
    for (i, &value) in data.iter().take(sample_size).enumerate() {
        print!("{:3} ", value);
        if (i + 1) % 16 == 0 {
            println!();
        }
    }
    println!();
}

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