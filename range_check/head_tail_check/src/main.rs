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

    let vec: Vec<u64> = util::read_vector_from_file("resources/vec1.txt")?;
    let vec_field: Vec<F> = vec.iter()
                                                 .map(|&x| F::from_canonical_u64(x))
                                                 .collect();

    //
    // STEP2: build the circuit by adding constraints of checking the head and tail are the exactly as expected
    // The public statement is "I have a vector, that aftering sorted, the head is X, and tail is Y "
    // The secret witness is the vecs, which generate the proof and are hidden from public
    //

    let upper = F::from_canonical_u64(10);
    let lower = upper.clone() * F::NEG_ONE;

    // Convert bounds into targets
    let lower_bound_target = builder.constant(lower);
    let upper_bound_target = builder.constant(upper);

    // Convert field elements to a type that supports sorting
    let mut vec_sorted: Vec<u64> = vec_field.iter().map(|&x| x.to_canonical_u64()).collect();
    vec_sorted.sort();

    // Convert sorted elements back to field elements
    let sorted_vec_field: Vec<F> = vec_sorted.iter().map(|&x| F::from_canonical_u64(x)).collect();
    // Convert sorted elements to Targets
    let sorted_vec_targets: Vec<Target> = sorted_vec_field.iter().map(|&x| builder.constant(x)).collect();

    let head = sorted_vec_targets[0];
    let tail = sorted_vec_targets[sorted_vec_field.len() - 1];

    builder.connect(lower_bound_target, head);
    builder.connect(upper_bound_target, tail);
    

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

