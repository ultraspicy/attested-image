mod util;

use anyhow::Result;
use plonky2::field::types::Field;
use plonky2::iop::witness::PartialWitness;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use std::time::Instant;

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
    // STEP2: build the circuit by adding constraints of checking adjacency
    // The public statement is "I have a vector, that aftering sorted, the adjacent element "
    // The secret witness is the vecs, which generate the proof and are hidden from public
    // 
    // NOTE: this is an inefficient implement of permutation check 
                                                
    // we check permutation by comparing (val, freq) pair
    let count1 = util::count_elements(&vec1_field);
    let count2 = util::count_elements(&vec2_field);

    for (key, &value1) in &count1 {
        let value2 = *count2.get(key).unwrap_or(&0);

        let value1_field = builder.constant(F::from_canonical_usize(value1));
        let value2_field = builder.constant(F::from_canonical_usize(value2));

        builder.connect(value1_field, value2_field);
    }
    for (key, &value2) in &count2 {
        let value1 = *count1.get(key).unwrap_or(&0);

        let value1_field = builder.constant(F::from_canonical_usize(value1));
        let value2_field = builder.constant(F::from_canonical_usize(value2));

        builder.connect(value1_field, value2_field);
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

