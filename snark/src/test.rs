use crate::{
    build_cs::{prove_outsource, verify_outsource, N_CARDS, N_PLAYS},
    create_and_rescale_outsource,
    gen_params::{params::VerifierParams, PROVER_PARAMS},
};
use ark_std::rand::SeedableRng;
use poker_core::mock_data::task::mock_task;
use rand_chacha::ChaChaRng;

// cargo test --release --package poker-snark --lib -- test::test_outsource --exact --nocapture
#[test]
fn test_outsource() {
    let mut rng = ChaChaRng::from_seed([0u8; 32]);
    let (players_key, reveal_outsources, unmask_outsources, signature_outsources) =
        create_and_rescale_outsource(&mock_task(), N_PLAYS, N_CARDS);

    println!("-------------start------------");

    let verifier_params = VerifierParams::get().unwrap();

    let start = std::time::Instant::now();
    let proof = prove_outsource(
        &mut rng,
        &players_key,
        &reveal_outsources,
        &unmask_outsources,
        &signature_outsources,
        &PROVER_PARAMS,
    )
    .unwrap();
    println!("Prove time: {:.2?}", start.elapsed());

    // println!("proof:{}",export_solidity_proof(&proof));

    verify_outsource(
        &verifier_params,
        &players_key,
        &reveal_outsources,
        &unmask_outsources,
        &signature_outsources,
        &proof,
    )
    .unwrap();

    let start = std::time::Instant::now();
    let proof = prove_outsource(
        &mut rng,
        &players_key,
        &reveal_outsources,
        &unmask_outsources,
        &signature_outsources,
        &PROVER_PARAMS,
    )
    .unwrap();
    println!("Prove time: {:.2?}", start.elapsed());

    verify_outsource(
        &verifier_params,
        &players_key,
        &reveal_outsources,
        &unmask_outsources,
        &signature_outsources,
        &proof,
    )
    .unwrap();
}
