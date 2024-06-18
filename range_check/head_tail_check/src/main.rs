mod util;

use anyhow::Result;
use plonky2::field::types::{Field, PrimeField64};
use plonky2::iop::witness::PartialWitness;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use std::time::Instant;
use plonky2::iop::target::Target;

fn main() -> Result<()> {
    //
    // STEP1: boilerplate code of circuit setup and data preparation
    const D: usize = 2;
    // PoseidonGoldilocksConfig provides poseidon hash function and the Goldilocks field.
    // C is type alias for PoseidonGoldilocksConfig. 
    type C = PoseidonGoldilocksConfig; 
    type F = <C as GenericConfig<D>>::F;
    // CircuitConfig defines number of gates, number of wires, and other parameters that dictate how the circuit is built and operates.
    let config: CircuitConfig = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // todo: set target later so the range is part of the public statement
    let range = builder.add_virtual_target();
    let field_modulus: u64 = 18446744069414584321; // (1u64 << 64) - (1u64 << 32) + 1;

    // load data from file and load them onto Goldilocks field
    let vec: Vec<i64> = util::read_vector_from_file("resources/vec1.txt")?;
    let vec_field: Vec<F> = vec.iter().map(|&x| {
        if x >= 0 {
            F::from_canonical_u64(x as u64)
        } else {

            F::from_canonical_u64((field_modulus as i64 + x) as u64)
        }
    }).collect();

    // Bound values
    // todo convert target into number
    let lower_bound = -10i64;
    let upper_bound = 10i64;

    // Convert valid values to field elements
    let valid_values: Vec<F> = (lower_bound..=upper_bound)
        .map(|x| {
            if x >= 0 {
                F::from_canonical_u64(x as u64)
            } else {
                F::from_canonical_u64((field_modulus as i64 + x) as u64)
            }
        })
        .collect();

    let valid_value_targets: Vec<Target> = valid_values.iter().map(|&x| builder.constant(x)).collect();

    // STEP2: Build the circuit by adding constraints to check the range
    for &value in &vec_field {
        let value_target = builder.constant(value);
        let mut any_valid = builder.zero();
        let one = builder.one();
        let zero  = builder.zero();
        
        for &valid_target in &valid_value_targets {
            let is_equal = builder.is_equal(value_target, valid_target);
            let is_valid = builder.select(is_equal, one, zero);
            any_valid = builder.add(any_valid, is_valid);
        }

        builder.connect(any_valid, one);
    }

    // boilerplate code for benchmark, prove and verify
    let start_build = Instant::now();
    let data = builder.build::<C>();
    let build_duration = start_build.elapsed();
    println!("Circuit built in: {:?}", build_duration);

    let start_prove = Instant::now();
    let pw = PartialWitness::new();
    let proof = data.prove(pw)?;
    let prove_duration = start_prove.elapsed();
    println!("Proof generated in: {:?}", prove_duration);

    println!("Permutation proof generated successfully.");

    let start_verify = Instant::now();
    data.verify(proof)?;
    let verify_duration = start_verify.elapsed();
    println!("Proof verified in: {:?}", verify_duration);

    println!("Proof verified successfully.");

    Ok(())

}

