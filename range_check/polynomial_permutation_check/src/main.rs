mod util;

use anyhow::Result;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use rand::Rng;

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
    poly.iter().rev().fold(F::ZERO, |acc, &coeff| acc * x + coeff)
}

/// Check if f(Ω) is a permutation of g(Ω)
fn check_permutation(f_omega: &[usize], g_omega: &[usize]) -> Result<bool> {
    // Convert to field elements
    let f_values: Vec<F> = f_omega.iter().map(|&x| F::from_canonical_usize(x as usize)).collect();
    let g_values: Vec<F> = g_omega.iter().map(|&x| F::from_canonical_usize(x as usize)).collect();

    // Construct the polynomials
    let f_poly = construct_polynomial(&f_values);
    let g_poly = construct_polynomial(&g_values);

    // Randomly sample r
    let r = {
        let mut rng = rand::thread_rng();
        // todo(jianfeng) ideally we should sample the r from the full field
        F::from_canonical_usize(rng.gen_range(0..100))
    };

    // open the polynomials at a random r
    let f_r = open_polynomial(&f_poly, r);
    let g_r = open_polynomial(&g_poly, r);

    // Perform the product check
    let prod_check = f_r / g_r;
    Ok(prod_check == F::ONE)
}
