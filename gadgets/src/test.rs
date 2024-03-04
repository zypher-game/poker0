use crate::{
    build_cs::{prove_outsource, verify_outsource, N_CARDS, N_PLAYS},
    gen_params::params::{ProverParams, VerifierParams},
    reveals::RevealOutsource,
    unmask::UnmaskOutsource,
};
use ark_std::rand::SeedableRng;
use poker_core::{mock_data::task::mock_task, play::PlayAction};
use rand_chacha::ChaChaRng;

// cargo test --release --package gadgets --lib -- test::test_outsource --exact --nocapture
#[test]
fn test_outsource() {
    let mut rng = ChaChaRng::from_seed([0u8; 32]);
    let task = mock_task();

    let mut reveal_outsources = vec![];
    let mut unmask_outsources = vec![];

    for plays in task.players_env.iter() {
        for env in plays.iter() {
            if let PlayAction::PLAY = env.action {
                let crypto_cards = env.play_cards.clone().unwrap().to_vec();

                for (crypto_card, reveal) in crypto_cards.iter().zip(env.reveals.iter()) {
                    let reveal_cards = reveal.iter().map(|x| x.0).collect::<Vec<_>>();
                    let proofs = reveal.iter().map(|x| x.1).collect::<Vec<_>>();
                    let reveal_outsource =
                        RevealOutsource::new(crypto_card, &reveal_cards, &proofs);
                    reveal_outsources.push(reveal_outsource);

                    let reveal_cards_projective =
                        reveal_cards.iter().map(|x| x.0.into()).collect::<Vec<_>>();
                    let unmasked_card = zshuffle::reveal::unmask(
                        &crypto_card.0.to_ciphertext(),
                        &reveal_cards_projective,
                    )
                    .unwrap();
                    let unmask_outsource =
                        UnmaskOutsource::new(crypto_card, &reveal_cards, &unmasked_card);
                    unmask_outsources.push(unmask_outsource);
                }
            }
        }
    }

    assert_eq!(reveal_outsources.len(), unmask_outsources.len());

    let n = reveal_outsources.len();
    let m = n % N_PLAYS;
    reveal_outsources.extend_from_slice(&reveal_outsources.clone()[m..(N_CARDS - 2 - n + m)]);
    unmask_outsources.extend_from_slice(&unmask_outsources.clone()[m..(N_CARDS - 2 - n + m)]);

    println!("-------------start------------");

    let start = std::time::Instant::now();
    let prover_params = ProverParams::gen().unwrap();
    println!("Gen params time: {:.2?}", start.elapsed());
    let verifier_params = VerifierParams::get().unwrap();

    let start = std::time::Instant::now();
    let proof = prove_outsource(
        &mut rng,
        &task.players_keys,
        &reveal_outsources,
        &unmask_outsources,
        &prover_params,
    )
    .unwrap();
    println!("Prove time: {:.2?}", start.elapsed());

    verify_outsource(
        &verifier_params,
        &task.players_keys,
        &reveal_outsources,
        &unmask_outsources,
        &proof,
    )
    .unwrap();
}
