//! A simple script to generate and verify the proof of a given program.

use sp1_core::{utils, SP1Prover, SP1Stdin, SP1Verifier};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
const DATA: &[u8] = include_bytes!("../test.dat");

fn main() {
    utils::setup_tracer();

    let input = DATA;
    println!("=================================");
    println!("len = {}", input.len());

    let mut stdin = SP1Stdin::new();
    stdin.write_slice(input);

    let mut proof = SP1Prover::prove(ELF, stdin).expect("proving failed");

    // Read output.
    let ret = proof.stdout.read::<u32>();
    println!("ret: {}", ret);
    // Verify proof.
    SP1Verifier::verify(ELF, &proof).expect("verification failed");

    // Save proof.
    proof
        .save("proof-with-io.json")
        .expect("saving proof failed");

    println!("succesfully generated and verified proof for the program!")
}