mod util;

use plonky2::field::extension::Extendable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::util::{log2_ceil, log2_strict};
use core::cmp::max;
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
use plonky2::field::fft::fft_root_table;
use plonky2_maybe_rayon::MaybeParIter;
use plonky2::field::extension::quadratic::QuadraticExtension;

static PIXELS : usize = 16; // assume a 16-pixel image
static EXPONENT : u32 = 5; // each pixel can be 0..31
static PIXEL_RANGE : usize = 2_usize.pow(EXPONENT);
static HASH_LENGTH : usize = 128;
const D: usize = 2;
// The max degree of polynomial, this value needs to be a power of 2 for IFFT
const DEGREE : usize = 1 << 8;
type C = PoseidonGoldilocksConfig; // PoseidonGoldilocksConfig provides poseidon hash function and the Goldilocks field.
type F = <C as GenericConfig<D>>::F;
type H = <C as GenericConfig<D>>::Hasher;

// Function to evaluate a polynomial given by its coefficients at a point x
fn evaluate_polynomial(coeffs: &[GoldilocksField], x: GoldilocksField) -> GoldilocksField {
    coeffs.iter().rev().fold(GoldilocksField::ZERO, |acc, &coeff| acc * x + coeff)
}

fn main() {  
    // FRI commitment constants
    let rate_bits = 2;
    let cap_height = 4;
    let max_quotient_degree_factor = 4;
    let degree_bits = log2_strict(DEGREE);
    let omega = F::primitive_root_of_unity(degree_bits);
    println!("omega = {:?}\n", omega);
    let max_fft_points = 1 << (degree_bits + max(rate_bits, log2_ceil(max_quotient_degree_factor)));
    let fft_root_table = fft_root_table(max_fft_points);

    // vanishing polynomial 
    let mut vanishing_poly_coefficient = Vec::new();
    vanishing_poly_coefficient.push(F::ONE);
    for _ in 0..DEGREE - 1 {
        vanishing_poly_coefficient.push(F::ZERO);
    }
    vanishing_poly_coefficient.push(F::ZERO - F::ONE);
    let vanishing_poly = PolynomialCoeffs::new(vanishing_poly_coefficient);

    // z = v || w
    let mut z_vals_usize: Vec<usize> = Vec::new();

    // w is the range of the pixel, typically from 0 to 255. We pad w 0 up to len of `DEGREE`
    // w_vals = [0, 1,...,PIXEL_RANGE - 1, 0, 0, ..., 0]
    // w.len() = degree
    let mut w_vals: Vec<_> = (0..PIXEL_RANGE).map(|i| GoldilocksField(i as u64)).collect();
    z_vals_usize.extend(0..PIXEL_RANGE as usize);
    w_vals.extend((0..DEGREE - PIXEL_RANGE as usize).map(|_| F::ZERO));
    println!("w_val = {:?}\n", w_vals);
    let w = PolynomialValues::new(w_vals).ifft();

    // sanity check
    // let eval_omega_on_w = w.eval(F::ONE);
    // println!("eval_omega_on_w = {:?}\n", eval_omega_on_w);

    // todo replace v_vals with actual image data
    // v is the value read from the image
    // ATM v_vals = [0, 1,...,14, 15, 0, 0, ..., 0], padded 0 up to len of `DEGREE`
    // v.len() = degree
    let mut v_vals: Vec<_> = (0..PIXELS).map(|i| GoldilocksField(i as u64)).collect();
    z_vals_usize.extend(0..PIXELS);
    v_vals.extend((0..DEGREE - PIXELS).map(|_| F::ZERO));
    println!("{:?}", v_vals);
    let v = PolynomialValues::new(v_vals).ifft();
    // sanity check
    // let eval_omega_on_v = v.eval(omega * omega * omega * omega * omega);
    // println!("eval_omega_on_v = {:?}\n", eval_omega_on_v);

    z_vals_usize.extend((0..DEGREE - z_vals_usize.len()).map(|_|0));    
    z_vals_usize.sort();
    println!("z_vals_usize = {:?}", z_vals_usize);
    let z_vals: Vec<_> = z_vals_usize.into_iter().map(|i| GoldilocksField(i as u64)).collect();
    let z = PolynomialValues::new(z_vals.clone()).ifft();

    // commit to w, v, z
    let mut values_vec_0 = Vec::new();
    values_vec_0.push(w.clone());
    values_vec_0.push(v.clone());
    values_vec_0.push(z.clone());
    println!("Degree of polynomial w: {}", w.clone().coeffs.len() - 1);

    let commit0 = PolynomialBatch::<F, C, D>::from_coeffs(
        values_vec_0,
        rate_bits, // trade-off between proof size and computational efficiency.
        true,
        cap_height, // try using the greatest-as-possible value to reduce the work of the verifier
        &mut TimingTree::default(),
        Some(&fft_root_table), // pre-compute  [1, ω, ω^2, ω^3, ..., ω^(max_fft_points-1)] table instead of computing on the fly
    );

    // todo: make gamma a uniformly sampled r on the field
    let gamma = GoldilocksField(123 as u64);

    // Permutation argument
    // We want to prove:
    //           product_{i=0}^{D-1}(v_i + gamma) * product_{i=0}^{PIXEL_RANGE-1}(w_i + gamma) = product_{i=0}^{D + PIXEL_RANGE - 1}(z_i + gamma) 
    // where v holds the image pixels, w is the range that the pixel values must lie in [0, PIXEL_RANGE-1],
    // and z is the sorted concatentation of v and w
 
    // w_prod_vals = [1, (gamma), [(gamma)(1 + gamma)],...,[(gamma)...(PIXEL_RANGE - 1 + gamma)]]
    let w_prod_vals = std::iter::once(F::ONE)
        .chain((0..PIXEL_RANGE).scan(F::ONE, |product, i| {
            *product *= GoldilocksField(i as u64) + gamma;
            Some(*product)
        }))
        .chain({
            let mut product = (0..PIXEL_RANGE)
                .fold(F::ONE, |acc, i| acc * (GoldilocksField(i as u64) + gamma));
            std::iter::repeat_with(move || {
                product *= gamma;
                product
            }).take(DEGREE - PIXEL_RANGE - 1)
        })
        .collect::<Vec<F>>();

    let mut w_prod_omega_vals = w_prod_vals[1..].to_vec(); 
    w_prod_omega_vals.push(w_prod_vals[0]);
    let w_prod = PolynomialValues::new(w_prod_vals).ifft();
    let w_prod_omega = PolynomialValues::new(w_prod_omega_vals).ifft();
    // n_1(X) = ω^(DEGREE-1) * X - 1
    let n_1_coeffs = vec![omega.exp_u64((DEGREE - 1) as u64), F::ZERO - F::ONE];
    let n_1 = PolynomialCoeffs::from(n_1_coeffs);
    // p(X) = γ
    let mut gamma_coeffs = Vec::new();
    gamma_coeffs.push(gamma);
    let gamma_poly = PolynomialCoeffs::from(gamma_coeffs);
    let (q_w, r_w) = (&(&w_prod_omega - &(&w_prod * &(&gamma_poly + &w))) * &n_1).div_rem(&vanishing_poly);
    assert!(r_w.is_zero());
    let mut values_vec_1 = Vec::new();
    println!("w_prod = {:?}", w_prod);
    println!("w_prod_omega = {:?}", w_prod_omega);
    values_vec_1.push(w_prod); // prove that we construct the w_prod honestly
    values_vec_1.push(q_w);


    let mut v_prod_vals = Vec::new(); // v_prod_vals = [1, (pixel_0 + gamma), [(pixel_0 + gamma)(pixel_1 + gamma)],...,[(pixel_0 + gamma)...(pixel_{D-1} + gamma)]]
    let mut product = F::ONE;
    v_prod_vals.push(product);

    for i in 0..PIXELS {
        let pixel_in_fr = GoldilocksField(i as u64);
        product *= pixel_in_fr + gamma;
        v_prod_vals.push(product)
    }
    for _ in 0..DEGREE - PIXELS - 1 {
        product *= gamma;
        v_prod_vals.push(product);
    }
    let mut v_prod_omega_vals = Vec::new(); // v_prod_omega_vals = [(pixel_0 + gamma), [(pixel_0 + gamma)(pixel_1 + gamma)],...,[(pixel_0 + gamma)...(pixel_{D-1} + gamma)], 1]
    for i in 1..v_prod_vals.len() {
        v_prod_omega_vals.push(v_prod_vals[i]);
    }
    v_prod_omega_vals.push(v_prod_vals[0]);
    let v_prod = PolynomialValues::from(v_prod_vals).ifft();
    let v_prod_omega = PolynomialValues::from(v_prod_omega_vals).ifft();
    let (q_v, r_v) = (&(&v_prod_omega - &(&v_prod * &(&gamma_poly + &v))) * &n_1).div_rem(&vanishing_poly);
    assert!(r_v.is_zero());
    values_vec_1.push(v_prod); // prove that we construct the v_prod honestly
    values_vec_1.push(q_v);

    let mut z_prod_vals = Vec::new(); // z_prod_vals = [1, z_vals_0 + gamma, [(z_0 + gamma)(z_vals_1 + gamma)],...,[(z_vals_0 + gamma)...(z_vals_{PIXEL_RANGE + D - 1} + gamma)]]
    let mut product = F::ONE;
    z_prod_vals.push(product);
    for i in 0..z_vals.len() - 1 {
        product *= z_vals[i] + gamma;
        z_prod_vals.push(product);
    }
    let mut z_prod_omega_vals = Vec::new();
    for i in 1..z_prod_vals.len() {
        z_prod_omega_vals.push(z_prod_vals[i]);
    }
    z_prod_omega_vals.push(z_prod_vals[0]);
    let z_prod = PolynomialValues::from(z_prod_vals).ifft();
    let z_prod_omega = PolynomialValues::from(z_prod_omega_vals).ifft();
    let (q_z, r_z) = (&(&z_prod_omega - &(&z_prod * &(&gamma_poly + &z))) * &n_1).div_rem(&vanishing_poly);
    assert!(r_z.is_zero());
    values_vec_1.push(z_prod.clone()); // prove that we construct the z_prod honestly
    values_vec_1.push(q_z.clone());
    println!("Degree of polynomial z_prod: {}", z_prod.clone().coeffs.len() - 1);
    println!("Degree of polynomial q_z: {}", q_z.clone().coeffs.len() - 1);

    let mut z_omega_vals = z_vals[1..].to_vec(); // z_omega_vals = [z_vals_0 + gamma,...,[(z_vals_0 + gamma)...(z_vals_{PIXEL_RANGE + D - 1} + gamma)], 1]
    z_omega_vals.push(z_vals.last().unwrap().clone()); // padding the last instead of z_vals[0]
    let z_omega = PolynomialValues::from(z_omega_vals.clone()).ifft();
    println!("z_omega = {:?}\n", z_omega);
    println!("z = {:?}\n", z);
    println!("z_vals = {:?}\n", z_vals);
    println!("z_omega_vals = {:?}\n", z_omega_vals);
    println!("vanishing_poly = {:?}\n", vanishing_poly); 
    
    
    let one = PolynomialCoeffs::from(vec![F::ONE]);
    // by trisha
    // bottleneck here :(
    // q_range[X] = (z[X] - z[omega*X])(1 - (z[X] - z[omega*X]) * n_1[X] / Z_H[X]
    // by jianfeng 
    // the coeff of z_omega and z are the same, makeing degree of q_range 1 less than it should be. So we cannot include q_range in batch with other polys with d = DEGREE
    // todo use a separate poly batch
    let (q_range, r_range) = (&(&(&z_omega - &z) * &(&one - &(&z_omega - &z))) * &n_1).div_rem(&vanishing_poly);
    //println!("z_val = {:?}\n", z_val);
    // println!("r_range = {:?}\n", r_range);
    assert!(r_range.is_zero());  // FAILS here
    values_vec_1.push(q_range.clone());
    println!("Degree of polynomial q_range: {}", q_range.clone().coeffs.len() - 1);

    let commit1 = PolynomialBatch::<F, C, D>::from_coeffs(
        values_vec_1,
        rate_bits,
        true,
        cap_height,
        &mut TimingTree::default(),
        Some(&fft_root_table),
    );
    
    let mut challenger = Challenger::<F, H>::new();
    challenger.observe_cap::<H>(&commit0.merkle_tree.cap);
    challenger.observe_cap::<H>(&commit1.merkle_tree.cap);

    let zeta = challenger.get_extension_challenge::<D>();
    let g = <F as Extendable<D>>::Extension::primitive_root_of_unity(degree_bits);
    //assert!(zeta.exp_power_of_2(degree_bits) != <<PoseidonGoldilocksConfig as GenericConfig<D>>::F as Extendable<D>>::Extension::ONE);

    let commit0_polys = FriPolynomialInfo::from_range(
        0,
        0..commit0.polynomials.len(),
    );
    let commit1_polys = FriPolynomialInfo::from_range(
        1,
        0..commit1.polynomials.len(),
    );
    let all_polys = [commit0_polys, commit1_polys].concat();

    // evaluation at a random point
    let zeta_batch: FriBatchInfo<F, D> = FriBatchInfo {
        point: zeta,
        polynomials: all_polys.clone(),
    };
    // evaluation at a random point shifted by ω
    let zeta_next = g * zeta;
    let zeta_next_batch: FriBatchInfo<F, D> = FriBatchInfo {
        point: zeta_next,
        polynomials: all_polys.clone(),
    };
    // evaluation at ω^D
    let pixels = g.exp_u64((PIXELS) as u64);
    let pixels_batch: FriBatchInfo<F, D> = FriBatchInfo {
        point: pixels,
        polynomials: all_polys.clone(),
    };
    let openings = vec![zeta_batch, zeta_next_batch, pixels_batch];

    let fri_oracles = vec![
            FriOracleInfo {
                num_polys: commit0.polynomials.len(),
                blinding: true,
            },
            FriOracleInfo {
                num_polys: commit1.polynomials.len(),
                blinding: true,
            },
        ];
    let instance = FriInstanceInfo {
        oracles: fri_oracles,
        batches: openings,
    };

    let fri_config = FriConfig {
        rate_bits: rate_bits,
        cap_height: cap_height,
        proof_of_work_bits: 16,
        reduction_strategy: FriReductionStrategy::ConstantArityBits(4, 5),
        num_query_rounds: 28,
    };

    let opening_proof = PolynomialBatch::<F, C, D>::prove_openings(
        &instance,
        &[
            &commit0,
            &commit1,
        ],
        &mut challenger,
        &fri_config.fri_params(degree_bits, true),
        &mut TimingTree::default(),
    );
    
    // verifier
    let mut challenger = Challenger::<F, <PoseidonGoldilocksConfig as GenericConfig<D>>::Hasher>::new();

    let merkle_caps = &[
        commit0.merkle_tree.cap.clone(),
        commit1.merkle_tree.cap.clone()
    ];

    let fri_challenges = challenger.fri_challenges::<C, D>(
        &opening_proof.commit_phase_merkle_caps,
        &opening_proof.final_poly,
        opening_proof.pow_witness,
        degree_bits,
        &fri_config,
    );

    let eval_commitment = |z: <<PoseidonGoldilocksConfig as GenericConfig<D>>::F as Extendable<D>>::Extension, c: &PolynomialBatch<F, C, D>| {
        c.polynomials
            .par_iter()
            .map(|p| p.to_extension::<D>().eval(z))
            .collect::<Vec<_>>()   
    };

    let commit0_zeta_eval = eval_commitment(zeta, &commit0);
    let commit0_zeta_next_eval = eval_commitment(zeta_next, &commit0);
    let commit0_pixels_eval = eval_commitment(pixels, &commit0);

    let commit1_zeta_eval = eval_commitment(zeta, &commit1);
    let commit1_zeta_next_eval = eval_commitment(zeta_next, &commit1);
    let commit1_pixels_eval = eval_commitment(pixels, &commit1);

    let vanishing_poly_zeta_eval = vanishing_poly.to_extension::<D>().eval(zeta);
    let n_1_zeta_eval = n_1.to_extension::<D>().eval(zeta);

    let zeta_batch: FriOpeningBatch<F, D> = FriOpeningBatch {
        values: [
            commit0_zeta_eval.as_slice(),
            commit1_zeta_eval.as_slice()
        ].concat(),
    };
    let zeta_next_batch: FriOpeningBatch<F, D> =  FriOpeningBatch {
        values: [
            commit0_zeta_next_eval.as_slice(),
            commit1_zeta_next_eval.as_slice(),
        ].concat()
    };
    
    let fri_openings = FriOpenings {
        batches: vec![zeta_batch, zeta_next_batch],
    };

    let res = verify_fri_proof::<F, C, D>(
        &instance,
        &fri_openings,
        &fri_challenges,
        merkle_caps,
        &opening_proof,
        &fri_config.fri_params(degree_bits, true),
    );
    assert!(res.is_ok(), "FRI proof verification failed: {:?}", res.err());
    
    let gamma_ext = QuadraticExtension::from(gamma);
    // Check (w_prod[omega*zeta] - w_prod[zeta](gamma + w[zeta])) * n_1[zeta] = q_w[zeta] * Z_H[zeta]
    assert!((commit1_zeta_next_eval[0] - commit1_zeta_eval[0] * (gamma_ext  + commit0_zeta_eval[0])) * n_1_zeta_eval == commit1_zeta_eval[1] * vanishing_poly_zeta_eval);

}

