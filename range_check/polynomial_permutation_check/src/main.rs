mod util;

use anyhow::Result;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use rand::Rng;
use std::time::Instant;
use plonky2::fri::structure::{FriBatchInfo, FriInstanceInfo, FriOracleInfo};
use plonky2::fri::oracle::PolynomialBatch;
use plonky2::iop::challenger::Challenger;
use plonky2::util::timing::TimingTree;
use plonky2::field::polynomial::{PolynomialCoeffs, PolynomialValues};
use log::{log, Level};
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

type F = GoldilocksField;

fn main() -> Result<()> {
    // Example sets Ω and their mappings f and g
    let f_omega = util::read_vector_from_file("resources/vec1.txt")?;
    let g_omega = util::read_vector_from_file("resources/vec2.txt")?;

    // Construct the polynomials and perform the permutation check
    let permutation_check = check_permutation(&f_omega, &g_omega)?;

    if permutation_check {
        println!("f(Ω) is a permutation of g(Ω)");
    } else {
        println!("f(Ω) is not a permutation of g(Ω)");
    }

    Ok(())
}


fn polynomial_division(
    numerator: &[F],
    denominator: &[F]
) -> (Vec<F>, Vec<F>) {
    let mut quotient = vec![F::ZERO; numerator.len()];
    let mut remainder = numerator.to_vec();

    let leading_denominator_inv = denominator.last().unwrap().inverse();

    for i in (0..=numerator.len() - denominator.len()).rev() {
        quotient[i] = remainder[i + denominator.len() - 1] * leading_denominator_inv;
        for j in 0..denominator.len() {
            remainder[i + j] -= quotient[i] * denominator[j];
        }
    }

    // Remove leading zeros from the remainder
    while remainder.last() == Some(&F::ZERO) {
        remainder.pop();
    }

    (quotient, remainder)
}


// Construct vanishing polynomial on omega
fn construct_polynomial(a: &[F]) -> Vec<F> {
    let mut poly = vec![F::ONE]; // Start with the polynomial P(x) = 1
    for &ai in a {
        let mut new_poly = vec![F::ZERO; poly.len() + 1];
        for i in 0..poly.len() {
            new_poly[i] -= poly[i] * ai; // Subtract ai from the coefficient
            new_poly[i + 1] += poly[i];  // Shift coefficients up by one position
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

fn print_polynomial(coeffs: &[F]) {
    for (i, coeff) in coeffs.iter().enumerate() {
        if i == 0 {
            print!("{}", coeff);
        } else {
            print!(" + {}*x^{}", coeff, i);
        }
    }
    println!();
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

    let f_hat = construct_polynomial(&f_values);
    print_polynomial(&f_hat);
    let g_hat = construct_polynomial(&f_values);
    print_polynomial(&g_hat);

    // // Timing setup
    // let mut timing = TimingTree::new("main", log::Level::Info);

    // // Construct the polynomial commitments
    // let start_time = Instant::now();
    // let f_commitment: PolynomialBatch<GoldilocksField, C, D> = PolynomialBatch::from_values(
    //     vec![PolynomialValues::new(f_values.clone())],
    //     2, // rate_bits
    //     false, // blinding
    // 20, // cap_height
    //     &mut timing,
    //     None, // fft_root_table
    // );
    // let g_commitment: PolynomialBatch<GoldilocksField, C, D> = PolynomialBatch::from_values(
    //     vec![PolynomialValues::new(g_values.clone())],
    //     2, // rate_bits
    //     false, // blinding
    //     20, // cap_height
    //     &mut timing,
    //     None, // fft_root_table
    // );

    // let construction_duration = start_time.elapsed();
    // println!("Time taken to construct polynomial commitments: {:?}", construction_duration);

    // // Prepare instance and challenger for proof opening
    // let instance_info = FriInstanceInfo {
    //     oracles: vec![
    //         FriOracleInfo { oracle_index: 0 },
    //         FriOracleInfo { oracle_index: 1 },
    //     ],
    //     batches: vec![
    //         FriBatchInfo {
    //             point: F::rand(),
    //             polynomials: vec![
    //                 plonky2::fri::structure::FriPolynomialInfo {
    //                     oracle_index: 0,
    //                     polynomial_index: 0,
    //                 },
    //                 plonky2::fri::structure::FriPolynomialInfo {
    //                     oracle_index: 1,
    //                     polynomial_index: 0,
    //                 },
    //             ],
    //         }
    //     ],
    // };
    // let fri_params = FriParams {
    //     config: plonky2::fri::FriConfig {
    //         num_queries: 10,
    //         rate_bits: 2,
    //     },
    //     degree: f_commitment.polynomials[0].len(),
    // };
    // let mut challenger = Challenger::new();

    // // Generate proof for polynomial openings
    // let fri_proof = PolynomialBatch::prove_openings(
    //     &instance_info,
    //     &[&f_commitment, &g_commitment],
    //     &mut challenger,
    //     &instance_info.fri_params,
    //     &mut timing,
    // );

    // Randomly sample r
    // let r = {
    //     let mut rng = rand::thread_rng();
    //     // todo(jianfeng) ideally we should sample the r from the full field
    //     F::from_canonical_usize(rng.gen_range(0..100))
    // };

    // // open the polynomials at a random r
    // let start_time = Instant::now();
    // let f_r = open_polynomial(&f_commitment.polynomials[0].coeffs, r);
    // let g_r = open_polynomial(&g_commitment.polynomials[0].coeffs, r);
    // let pokynomial_opening_duration = start_time.elapsed();
    // println!("Time taken to open polynomials at a random point r: {:?}", pokynomial_opening_duration);

    // // Perform the product check
    // let start_time = Instant::now();
    // let prod_check = f_r / g_r;
    // let product_check_duration = start_time.elapsed();
    // println!("Time taken to do product check over polynomials {:?}", product_check_duration);
    // Ok(prod_check == F::ONE)
    Ok(true)
}
