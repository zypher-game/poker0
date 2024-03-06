use ark_bn254::Fr;
use ark_ff::PrimeField;
use num_bigint::BigUint;
use poker_core::{play::PlayAction, schnorr::PublicKey, task::Task};
use reveals::RevealOutsource;
use unmask::UnmaskOutsource;

pub mod build_cs;
pub mod gen_params;
pub mod public_keys;
pub mod reveals;
// pub mod signatures;
pub mod unmask;

#[cfg(test)]
pub mod test;

pub fn get_divisor() -> (Fr, BigUint) {
    let m_bytes = [
        6, 12, 137, 206, 92, 38, 52, 5, 55, 10, 8, 182, 208, 48, 43, 11, 171, 62, 237, 184, 57, 32,
        238, 10, 103, 114, 151, 220, 57, 33, 38, 241,
    ];
    let m_field = Fr::from_be_bytes_mod_order(&m_bytes);
    let m = BigUint::from_bytes_be(&m_bytes);

    (m_field, m)
}

pub fn create_outsource(
    task: &Task,
) -> (Vec<PublicKey>, Vec<RevealOutsource>, Vec<UnmaskOutsource>) {
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

    (
        task.players_keys.clone(),
        reveal_outsources,
        unmask_outsources,
    )
}

pub fn create_and_rescale_outsource(
    task: &Task,
    n_players: usize,
    n_cards: usize,
) -> (Vec<PublicKey>, Vec<RevealOutsource>, Vec<UnmaskOutsource>) {
    let (public_keys, mut reveal_outsources, mut unmask_outsources) = create_outsource(task);

    let n = reveal_outsources.len();
    let m = n % n_players;
    reveal_outsources.extend_from_slice(&reveal_outsources.clone()[m..(n_cards - 2 - n + m)]);
    unmask_outsources.extend_from_slice(&unmask_outsources.clone()[m..(n_cards - 2 - n + m)]);

    (public_keys, reveal_outsources, unmask_outsources)
}
