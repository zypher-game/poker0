use poker_core::{mock_data::task::mock_task, play::PlayAction};
use zplonk::{
    errors::ZplonkError,
    gen_params::{VerifierParamsSplitCommon, VerifierParamsSplitSpecific, VERIFIER_COMMON_PARAMS},
    turboplonk::{constraint_system::ConstraintSystem, indexer::indexer_with_lagrange},
};

use crate::{
    build_cs::{build_cs, N_CARDS, N_PLAYS},
    gen_params::VERIFIER_SPECIFIC_PARAMS,
    reveals::RevealOutsource,
    unmask::UnmaskOutsource,
};

// re-export
pub use zplonk::gen_params::{ProverParams, VerifierParams};

use super::{load_lagrange_params, load_srs_params};

/// Obtain the parameters for prover.
pub fn gen_prover_params() -> Result<ProverParams, ZplonkError> {
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

    let cs = build_cs(&task.players_keys, &reveal_outsources, &unmask_outsources);
    let pcs = load_srs_params(cs.size())?;
    let lagrange_pcs = load_lagrange_params(cs.size());

    let verifier_params = if let Ok(v) = load_verifier_params() {
        Some(v.verifier_params)
    } else {
        None
    };

    let prover_params =
        indexer_with_lagrange(&cs, &pcs, lagrange_pcs.as_ref(), verifier_params).unwrap();

    Ok(ProverParams {
        pcs,
        lagrange_pcs,
        cs,
        prover_params,
    })
}

/// Get the verifier parameters.
pub fn get_verifier_params() -> Result<VerifierParams, ZplonkError> {
    match load_verifier_params() {
        Ok(vk) => Ok(vk),
        Err(_e) => Ok(VerifierParams::from(gen_prover_params()?)),
    }
}

/// Load the verifier parameters from prepare.
pub fn load_verifier_params() -> Result<VerifierParams, ZplonkError> {
    match (VERIFIER_COMMON_PARAMS, VERIFIER_SPECIFIC_PARAMS) {
        (Some(c_bytes), Some(s_bytes)) => {
            let common: VerifierParamsSplitCommon =
                bincode::deserialize(c_bytes).map_err(|_| ZplonkError::DeserializationError)?;

            let special: VerifierParamsSplitSpecific =
                bincode::deserialize(s_bytes).map_err(|_| ZplonkError::DeserializationError)?;

            Ok(VerifierParams {
                shrunk_vk: common.shrunk_pcs,
                shrunk_cs: special.shrunk_cs,
                verifier_params: special.verifier_params,
            })
        }
        _ => Err(ZplonkError::MissingVerifierParamsError),
    }
}
