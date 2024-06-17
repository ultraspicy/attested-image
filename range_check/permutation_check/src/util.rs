use plonky2::field::types::Field;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn count_elements<F: Field>(vec: &Vec<F>) -> HashMap<F, usize> {
    let mut counts = HashMap::new();
    for &elem in vec {
        *counts.entry(elem).or_insert(0) += 1;
    }
    counts
}

pub fn read_vector_from_file(filename: &str) -> io::Result<Vec<u64>> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);
    
    let mut vector = Vec::new();
    for line in reader.lines() {
        let line = line?;
        for value in line.split(',') {
            if let Ok(num) = value.trim().parse::<u64>() {
                vector.push(num);
            }
        }
    }
    
    Ok(vector)
}