mod util;

use anyhow::Result;
use plonky2::field::extension::Extendable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::fri::verifier::verify_fri_proof;
use plonky2::hash::hash_types::HashOut;
use plonky2::hash::merkle_tree::MerkleCap;
use rand::Rng;
use std::time::Instant;
use plonky2::fri::structure::{FriBatchInfo, FriInstanceInfo, FriOpeningBatch, FriOpenings, FriOracleInfo, FriPolynomialInfo};
use plonky2::fri::oracle::PolynomialBatch;
use plonky2::iop::challenger::Challenger;
use plonky2::util::timing::TimingTree;
use plonky2::fri::{FriParams, FriConfig};
use plonky2::fri::reduction_strategies::FriReductionStrategy;
use plonky2::field::polynomial::{PolynomialCoeffs, PolynomialValues};
use log::{log, Level};
use plonky2::plonk::config::{GenericConfig, Hasher, PoseidonGoldilocksConfig};
use plonky2::field::extension::FieldExtension;
use plonky2::fri::proof::FriChallenges;
use plonky2::hash::poseidon::PoseidonHash;


type F = GoldilocksField;
type H = PoseidonHash;

fn main() -> Result<()> {
    // Example sets Ω and their mappings f and g
    let f_omega = util::read_vector_from_file("resources/vec1.txt")?;
    let g_omega = util::read_vector_from_file("resources/vec2.txt")?;

    // Construct the polynomials and perform the permutation check
    let permutation_check = check_permutation(&f_omega, &g_omega);

    // if permutation_check {
    //     println!("f(Ω) is a permutation of g(Ω)");
    // } else {
    //     println!("f(Ω) is not a permutation of g(Ω)");
    // }

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
fn check_permutation(f_omega: &[usize], g_omega: &[usize]) -> () {//Result<bool> {
    const D: usize = 2;
    // PoseidonGoldilocksConfig provides poseidon hash function and the Goldilocks field.
    // C is type alias for PoseidonGoldilocksConfig. 
    type C = PoseidonGoldilocksConfig; 

    // Convert to field elements
    let f_values: Vec<F> = f_omega.iter().map(|&x| F::from_canonical_usize(x as usize)).collect();
    let g_values: Vec<F> = g_omega.iter().map(|&x| F::from_canonical_usize(x as usize)).collect();

    let f_hat = construct_polynomial(&f_values);
    // print_polynomial(&f_hat);
    let g_hat = construct_polynomial(&g_values);
    // print_polynomial(&g_hat);

    // Timing setup
    let mut timing = TimingTree::new("main", log::Level::Info);

    // Construct the polynomial commitments, commit to the quotient
    let start_time = Instant::now();
    let (quotient, remainder) = polynomial_division(&f_hat, &g_hat);
    print_polynomial(&quotient);
    print_polynomial(&remainder);
    let quotient_commitment: PolynomialBatch<GoldilocksField, C, D> = PolynomialBatch::from_values(
        vec![PolynomialValues::new(quotient.clone())],
        2, // rate_bits
        false, // blinding
    2, // cap_height
        &mut timing,
        None, // fft_root_table
    );

    let construction_duration = start_time.elapsed();
    println!("Time taken to construct polynomial commitments: {:?}", construction_duration);
   
    // open the polynomials at a random r
    let mut rng = rand::thread_rng();
    // todo(jianfeng) ideally we should sample the r from the full field
    let r = F::from_canonical_usize(rng.gen_range(0..100));
    let r_ext = <GoldilocksField as Extendable<D>>::Extension::from_basefield_array([r; D]);

    let start_time = Instant::now();
    let pokynomial_opening_duration = start_time.elapsed();
    println!("Time taken to open polynomials at a random point r: {:?}", pokynomial_opening_duration);

    // Generate proof for polynomial evaluation
    let fri_params = FriParams {
        config: FriConfig {
            num_query_rounds: 10,
            rate_bits: 2,
            proof_of_work_bits: 0,
            cap_height: 2, 
            reduction_strategy: FriReductionStrategy::ConstantArityBits(2, 4),
        },
        hiding: true,
        degree_bits: quotient_commitment.polynomials[0].len(),
        reduction_arity_bits: vec![2],
    };

    
    let fri_polynomial_info = FriPolynomialInfo {
        oracle_index: 0,
        polynomial_index: 0,
    };
    let instance_info = FriInstanceInfo {
        oracles: vec![FriOracleInfo {
            blinding: true,
            num_polys: 1,
        }],
        batches: vec![FriBatchInfo {
            point: r_ext,
            polynomials: vec![fri_polynomial_info],
        }],
    };

    let mut challenger = Challenger::<F, H>::new();
    // Generate proof for polynomial openings
    let fri_proof = PolynomialBatch::prove_openings(
        &instance_info,
        &[&quotient_commitment],
        &mut challenger,
        &fri_params,
        &mut timing,
    );
    let fri_openings = FriOpenings {
        batches: vec![FriOpeningBatch {
            values: vec![r_ext],
        }],
    };

    let mut verifier_challenger = Challenger::<F, H>::new();

    // Generate the FRI challenges
    
    let fri_challenges = FriChallenges {
        fri_alpha: r_ext,
        fri_betas: (0..fri_params.reduction_arity_bits.len())
            .map(|_| r_ext)
            .collect(),
        fri_pow_response: F::from_canonical_usize(0), // Example, set accordingly
        fri_query_indices: (0..fri_params.config.num_query_rounds)
            .map(|_| rng.gen_range(0..fri_params.degree_bits))
            .collect(),
    };
   
    let initial_merkle_caps: Vec<MerkleCap<F, <PoseidonGoldilocksConfig as GenericConfig<D>>::Hasher>> = vec![MerkleCap(vec![HashOut::default()])];

    let is_valid = verify_fri_proof::<F,C,D>(
        &instance_info,
        &fri_openings,
        &fri_challenges,
        &initial_merkle_caps,
        &fri_proof,
        &fri_params
    );
    
    match is_valid {
        Ok(_) => println!("Proof is valid"),
        Err(e) => eprintln!("Proof verification failed: {:?}", e),
    }
}
