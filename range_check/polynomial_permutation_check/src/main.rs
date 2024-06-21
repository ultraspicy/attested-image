mod util;

use anyhow::Result;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use rand::Rng;
use std::time::Instant;
use plonky2::fri::structure::{FriBatchInfo, FriInstanceInfo};
use plonky2::fri::oracle::PolynomialBatch;
use plonky2::iop::challenger::Challenger;
use plonky2::util::timing::TimingTree;
use plonky2::field::polynomial::{PolynomialCoeffs, PolynomialValues};
use log::{log, Level};
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

type F = GoldilocksField;

fn main() -> Result<()> {
    // Example sets Ω and their mappings f and g
    let f_omega = util::read_vector_from_file("resources/gen_10.txt")?;
    let g_omega = util::read_vector_from_file("resources/gen_10.txt")?;

    // Construct the polynomials and perform the permutation check
    let permutation_check = check_permutation(&f_omega, &g_omega)?;

    if permutation_check {
        println!("f(Ω) is a permutation of g(Ω)");
    } else {
        println!("f(Ω) is not a permutation of g(Ω)");
    }

    Ok(())
}

/// Construct the polynomial \(\hat{p}(X)\) = \(\prod_{a \in \Omega} (X - p(a))\)
fn construct_polynomial(values: &[F]) -> Vec<F> {
    // Initialize polynomial as 1 (constant term)
    let mut poly = vec![F::ONE];
    for &val in values {
        let mut new_poly = vec![F::ZERO; poly.len() + 1];
        for i in 0..poly.len() {
            new_poly[i] -= poly[i] * val; 
            new_poly[i + 1] += poly[i]; 
        }
        poly = new_poly;
    }
    poly
}

/// open the polynomial at a given point using Horner's method
fn open_polynomial(poly: &[F], x: F) -> F {
    poly.iter().rev().fold(F::ZERO,
         |acc, &coeff| acc * x + coeff)
}

/// Check if f(Ω) is a permutation of g(Ω)
fn check_permutation(f_omega: &[usize], g_omega: &[usize]) -> Result<bool> {
    const D: usize = 2;
    // PoseidonGoldilocksConfig provides poseidon hash function and the Goldilocks field.
    // C is type alias for PoseidonGoldilocksConfig. 
    type C = PoseidonGoldilocksConfig; 

    // Convert to field elements
    let f_values: Vec<F> = f_omega.iter().map(|&x| F::from_canonical_usize(x as usize)).collect();
    let g_values: Vec<F> = g_omega.iter().map(|&x| F::from_canonical_usize(x as usize)).collect();

    // Timing setup
    let mut timing = TimingTree::new("main", log::Level::Info);

    // Construct the polynomial commitments
    let start_time = Instant::now();
    let f_commitment: PolynomialBatch<GoldilocksField, C, D> = PolynomialBatch::from_values(
        vec![PolynomialValues::new(f_values.clone())],
        2, // rate_bits
        false, // blinding
    4, // cap_height
        &mut timing,
        None, // fft_root_table
    );
    let g_commitment: PolynomialBatch<GoldilocksField, C, D> = PolynomialBatch::from_values(
        vec![PolynomialValues::new(g_values.clone())],
        2, // rate_bits
        false, // blinding
        4, // cap_height
        &mut timing,
        None, // fft_root_table
    );
    let construction_duration = start_time.elapsed();
    println!("Time taken to construct polynomial commitments: {:?}", construction_duration);

    // Randomly sample r
    let r = {
        let mut rng = rand::thread_rng();
        // todo(jianfeng) ideally we should sample the r from the full field
        F::from_canonical_usize(rng.gen_range(0..100))
    };

    // open the polynomials at a random r
    let start_time = Instant::now();
    let f_r = open_polynomial(&f_commitment.polynomials[0].coeffs, r);
    let g_r = open_polynomial(&g_commitment.polynomials[0].coeffs, r);
    let pokynomial_opening_duration = start_time.elapsed();
    println!("Time taken to open polynomials at a random point r: {:?}", pokynomial_opening_duration);

    // Perform the product check
    let start_time = Instant::now();
    let prod_check = f_r / g_r;
    let product_check_duration = start_time.elapsed();
    println!("Time taken to do product check over polynomials {:?}", product_check_duration);
    Ok(prod_check == F::ONE)
}
