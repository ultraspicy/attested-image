mod util;

use anyhow::Result;
use plonky2::field::extension::Extendable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::util::{log2_ceil, log2_strict};
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

static PIXELS : usize = 16; // assume a 16-pixel image
static EXPONENT : u32 = 5; // each pixel can be 0..31
static PIXEL_RANGE : i32 = 2_i32.pow(EXPONENT);
static HASH_LENGTH : usize = 128;
const D: usize = 2;
// The max degree of polynomial
const DEGREE : usize = 1 << 8;
// PoseidonGoldilocksConfig provides poseidon hash function and the Goldilocks field.
type C = PoseidonGoldilocksConfig; // C is type alias for PoseidonGoldilocksConfig. 
type F = <C as GenericConfig<D>>::F;

fn main() {  

    // FRI commitment constants
    let rate_bits = 2;
    let cap_height = 4;
    let max_quotient_degree_factor = 4;
    let degree_bits = log2_strict(DEGREE);
    let omega = F::primitive_root_of_unity(degree_bits);

    let mut vanishing_poly_coefficient = Vec::new();
    vanishing_poly_coefficient.push(F::ONE);
    for _ in 0..DEGREE - 1 {
        vanishing_poly_coefficient.push(F::ZERO);
    }
    vanishing_poly_coefficient.push(F::ZERO - F::ONE);
    let vanishing_poly = PolynomialCoeffs::new(vanishing_poly_coefficient);

    // z = v || w
    let mut z_vals_usize: Vec<usize> = Vec::new();

    // w_vals = [0, 1,...,PIXEL_RANGE - 1, 0, 0, ..., 0], w.len() = degree
    let mut w_vals: Vec<_> = (0..PIXEL_RANGE).map(|i| GoldilocksField(i as u64)).collect();
    z_vals_usize.extend(0..PIXEL_RANGE as usize);
    w_vals.extend((0..DEGREE - PIXEL_RANGE as usize).map(|_| F::ZERO));
    // w in its coefficient form
    println!("{:?}", w_vals);
    let w = PolynomialValues::new(w_vals).ifft();

    // todo replace v_vals with actual image data
    // v_vals = [0, 1,...,14, 15, 0, 0, ..., 0], v.len() = degree
    let mut v_vals: Vec<_> = (0..PIXELS).map(|i| GoldilocksField(i as u64)).collect();
    z_vals_usize.extend(0..PIXELS);
    v_vals.extend((0..DEGREE - PIXELS).map(|_| F::ZERO));
    println!("{:?}", v_vals);
    let v = PolynomialValues::new(v_vals).ifft();


    z_vals_usize.extend((0..DEGREE - z_vals_usize.len()).map(|_|0));    
    println!("{:?}", z_vals_usize);
    z_vals_usize.sort();
    let z_vals: Vec<_> = z_vals_usize.into_iter().map(|i| GoldilocksField(i as u64)).collect();
    let z = PolynomialValues::new(z_vals.clone()).ifft();

    // commit to w, v, z
    let gamma = GoldilocksField(123 as u64);

    // Permutation argument
    // We want to prove:
    //           product_{i=0}^{D-1}(v_i + gamma) * product_{i=0}^{PIXEL_RANGE-1}(w_i + gamma) = product_{i=0}^{D + PIXEL_RANGE - 1}(z_i + gamma) 
    // where v holds the image pixels, w is the range that the pixel values must lie in [0, PIXEL_RANGE-1],
    // and z is the sorted concatentation of v and w
    
}

