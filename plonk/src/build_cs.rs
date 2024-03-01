use ark_bn254::Fr;
use ark_ec::CurveGroup;
use poker_core::schnorr::PublicKey;
use rand_chacha::rand_core::{CryptoRng, RngCore};
use zplonk::{
    anemoi::AnemoiJive254,
    errors::Result,
    gen_params::{ProverParams, VerifierParams},
    poly_commit::kzg_poly_commitment::KZGCommitmentSchemeBN254,
    turboplonk::{
        constraint_system::turbo::TurboCS, indexer::PlonkProof, prover::prover_with_lagrange,
        verifier::verifier,
    },
    utils::transcript::Transcript,
};

use crate::{public_keys::PublicKeyOutsource, reveals::RevealOutsource, unmask::UnmaskOutsource};

pub type Proof = PlonkProof<KZGCommitmentSchemeBN254>;

const PLONK_PROOF_TRANSCRIPT: &[u8] = b"Plonk poker Proof";

pub const N_CARDS: usize = 52;
pub const N_PLAYS: usize = 3;

pub(crate) fn build_cs(
    public_keys: &[PublicKey],
    reveal_outsources: &[RevealOutsource],
    unmask_outsources: &[UnmaskOutsource],
) -> TurboCS<Fr> {
    let mut reveal_outsources = reveal_outsources.to_vec();
    let mut unmask_outsources = unmask_outsources.to_vec();

    let mut cs = TurboCS::new();
    cs.load_anemoi_parameters::<AnemoiJive254>();

    let pk_outsources = PublicKeyOutsource::new(&mut cs, &public_keys);
    pk_outsources.prepare_pi_variables(&mut cs);

    for reveal_outsource in reveal_outsources.iter_mut() {
        reveal_outsource.generate_constraints(&mut cs, &pk_outsources);
        reveal_outsource.prepare_pi_variables(&mut cs);
    }

    for (unmask_outsource, reveal_outsource) in
        unmask_outsources.iter_mut().zip(reveal_outsources.iter())
    {
        unmask_outsource.set_crypto_card_var(reveal_outsource.crypto_card_var);
        unmask_outsource.set_reveal_cards_var(&reveal_outsource.reveal_card_vars);
        unmask_outsource.generate_constraints(&mut cs);
        unmask_outsource.prepare_pi_variables(&mut cs);
    }

    cs.pad();

    cs
}

pub fn prove_outsource<R: CryptoRng + RngCore>(
    prng: &mut R,
    public_keys: &[PublicKey],
    reveal_outsources: &[RevealOutsource],
    unmask_outsources: &[UnmaskOutsource],
    prover_params: &ProverParams,
) -> Result<Proof> {
    assert_eq!(public_keys.len(), N_PLAYS);
    assert_eq!(reveal_outsources.len(), N_CARDS - N_PLAYS + 1);
    assert_eq!(unmask_outsources.len(), N_CARDS - N_PLAYS + 1);

    let mut cs = build_cs(public_keys, reveal_outsources, unmask_outsources);
    let witness = cs.get_and_clear_witness();

    let mut transcript = Transcript::new(PLONK_PROOF_TRANSCRIPT);

    let proof = prover_with_lagrange(
        prng,
        &mut transcript,
        &prover_params.pcs,
        prover_params.lagrange_pcs.as_ref(),
        &cs,
        &prover_params.prover_params,
        &witness,
    )?;

    Ok(proof)
}

pub fn verify_outsource(
    verifier_params: &VerifierParams,
    public_keys: &[PublicKey],
    reveal_outsources: &[RevealOutsource],
    unmask_outsources: &[UnmaskOutsource],
    proof: &Proof,
) -> Result<()> {
    assert_eq!(public_keys.len(), N_PLAYS);
    assert_eq!(reveal_outsources.len(), N_CARDS - N_PLAYS + 1);
    assert_eq!(unmask_outsources.len(), N_CARDS - N_PLAYS + 1);

    let mut transcript = Transcript::new(PLONK_PROOF_TRANSCRIPT);

    let mut online_inputs = vec![];
    for pk in public_keys.iter() {
        online_inputs.push(pk.0.x);
        online_inputs.push(pk.0.y);
    }
    for reveal_outsource in reveal_outsources.iter() {
        online_inputs.push(reveal_outsource.crypto_card.0.e1.x);
        online_inputs.push(reveal_outsource.crypto_card.0.e1.y);
        online_inputs.push(reveal_outsource.crypto_card.0.e2.x);
        online_inputs.push(reveal_outsource.crypto_card.0.e2.y);
    }
    for unmask_outsource in unmask_outsources.iter() {
        let aff = unmask_outsource.unmasked_card.into_affine();
        online_inputs.push(aff.x);
        online_inputs.push(aff.y);
    }

    Ok(verifier(
        &mut transcript,
        &verifier_params.shrunk_vk,
        &verifier_params.shrunk_cs,
        &verifier_params.verifier_params,
        &online_inputs,
        proof,
    )?)
}

#[cfg(test)]
mod test {
    use crate::{
        reveals::RevealOutsource,
        unmask::UnmaskOutsource,
    };
    use ark_ec::CurveGroup;
    use poker_core::{mock_data::task::mock_task, play::PlayAction};

    use super::build_cs;

    #[test]
    fn test_build_cs() {
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

        let mut cs = build_cs(&task.players_keys, &reveal_outsources, &unmask_outsources);

        let mut online_inputs = vec![];
        for pk in task.players_keys.iter() {
            online_inputs.push(pk.0.x);
            online_inputs.push(pk.0.y);
        }
        for reveal_outsource in reveal_outsources.iter() {
            online_inputs.push(reveal_outsource.crypto_card.0.e1.x);
            online_inputs.push(reveal_outsource.crypto_card.0.e1.y);
            online_inputs.push(reveal_outsource.crypto_card.0.e2.x);
            online_inputs.push(reveal_outsource.crypto_card.0.e2.y);
        }
        for unmask_outsource in unmask_outsources.iter() {
            let aff = unmask_outsource.unmasked_card.into_affine();
            online_inputs.push(aff.x);
            online_inputs.push(aff.y);
        }

        let witness = cs.get_and_clear_witness();
        cs.verify_witness(&witness, &online_inputs).unwrap();
    }
}
