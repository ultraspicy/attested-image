use sp1_sdk::{utils, ProverClient, SP1Stdin};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {
    utils::setup_tracer();

    let input: &mut [u8] = &mut [3, 1, 2, 3, 1, 2, 4];

    let mut stdin = SP1Stdin::new();
    stdin.write_slice(input);

    // let mut proof = SP1Prover::prove(ELF, stdin).expect("proving failed");

    // // Read output.
    // let ret = proof.stdout.read::<u8>();

    // println!("ret: {}", ret);
    // // Verify proof.
    // SP1Verifier::verify(ELF, &proof).expect("verification failed");

    // Generate and verify the proof
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let mut proof = client.prove(&pk, stdin).unwrap();

    let ret = proof.public_values.read::<bool>();
    println!("circuit returns {}", ret);

    client.verify(&proof, &vk).expect("verification failed");

    // Save proof.
    proof
        .save("proof-with-io.json")
        .expect("saving proof failed");

    println!("succesfully generated and verified proof for the program!")
}