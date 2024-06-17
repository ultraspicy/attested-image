mod util;

use anyhow::Result;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use std::collections::HashMap;

fn main() -> Result<()> {
    ///
    /// STEP1: circuit setup and data preparation
    /// 
    const D: usize = 2;
    // PoseidonGoldilocksConfig provides poseidon hash function and the Goldilocks field.
    // C is type alias for PoseidonGoldilocksConfig. 
    type C = PoseidonGoldilocksConfig; 
    type F = <C as GenericConfig<D>>::F;
    // CircuitConfig defines number of gates, number of wires, and other parameters that dictate how the circuit is built and operates.
    let config: CircuitConfig = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // we hardcode vec1 and vec2 for now. It should be modified 
    // to load vec1, vec2 from file 
    let vec1: Vec<u64> = vec![1, 2, 3, 4, 10];
    let vec1_field: Vec<F> = vec1.iter()
                                                 .map(|&x| F::from_canonical_u64(x))
                                                 .collect();
    let vec2: Vec<u64> = vec![10, 4, 3, 2, 1];
    let vec2_field: Vec<F> = vec2.iter()
                                                 .map(|&x| F::from_canonical_u64(x))
                                                 .collect();

    ///
    /// STEP2: build the circuit by adding constraints of checking permutation  
    /// 
                                                
    // we check permutation by comparing (val, freq) pair
    let count1 = count_elements(&vec1_field);
    let count2 = count_elements(&vec2_field);

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

    let pw = PartialWitness::new();

    let data = builder.build::<C>();
    let proof = data.prove(pw)?;

    println!("Permutation proof generated successfully.");

    // Verify the proof
    data.verify(proof)?;

    println!("Proof verified successfully.");

    Ok(())

}

fn count_elements<F: Field>(vec: &Vec<F>) -> HashMap<F, usize> {
    let mut counts = HashMap::new();
    for &elem in vec {
        *counts.entry(elem).or_insert(0) += 1;
    }
    counts
}

