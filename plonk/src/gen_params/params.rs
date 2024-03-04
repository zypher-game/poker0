use crate::build_cs::{N_CARDS, N_PLAYS};
use crate::gen_params::errors::Result;
use crate::poly_commit::pcs::PolyComScheme;
use crate::reveals::RevealOutsource;
use crate::unmask::UnmaskOutsource;
use crate::{build_cs::build_cs, gen_params::VERIFIER_SPECIFIC_PARAMS};
use crate::{
    poly_commit::kzg_poly_commitment::KZGCommitmentSchemeBN254,
    turboplonk::{
        constraint_system::{turbo::TurboCS, ConstraintSystem},
        indexer::{indexer_with_lagrange, PlonkProverParams, PlonkVerifierParams},
    },
};
use ark_bn254::{Fr, G1Projective};
use poker_core::mock_data::task::mock_task;
use poker_core::play::PlayAction;
use serde::{Deserialize, Serialize};

use super::errors::SetUpError;
use super::{LAGRANGE_BASES, SRS};
use super::{PERMUTATION, VERIFIER_COMMON_PARAMS};

#[derive(Serialize, Deserialize)]
/// The verifier parameters.
pub struct VerifierParams {
    /// The shrunk version of the polynomial commitment scheme.
    pub shrunk_vk: KZGCommitmentSchemeBN254,
    /// The shrunk version of the constraint system.
    pub shrunk_cs: TurboCS<Fr>,
    /// The TurboPlonk verifying key.
    pub verifier_params: PlonkVerifierParams<KZGCommitmentSchemeBN254>,
}

#[derive(Serialize, Deserialize)]
/// The common part of the verifier parameters.
pub struct VerifierParamsSplitCommon {
    /// The shrunk version of the polynomial commitment scheme.
    pub shrunk_pcs: KZGCommitmentSchemeBN254,
}

#[derive(Serialize, Deserialize)]
/// The specific part of the verifier parameters.
pub struct VerifierParamsSplitSpecific {
    /// The shrunk version of the constraint system.
    pub shrunk_cs: TurboCS<Fr>,
    /// The verifier parameters.
    pub verifier_params: PlonkVerifierParams<KZGCommitmentSchemeBN254>,
}

#[derive(Serialize, Deserialize)]
/// The prover parameters.
pub struct ProverParams {
    /// The full SRS for the polynomial commitment scheme.
    pub pcs: KZGCommitmentSchemeBN254,
    /// The Lagrange basis format of SRS.
    pub lagrange_pcs: Option<KZGCommitmentSchemeBN254>,
    /// The constraint system.
    pub cs: TurboCS<Fr>,
    /// The TurboPlonk proving key.
    pub prover_params: PlonkProverParams<KZGCommitmentSchemeBN254>,
}

impl ProverParams {
    /// Obtain the parameters for 2048 game.
    pub fn gen() -> Result<ProverParams> {
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

        let verifier_params = if let Ok(v) = VerifierParams::load() {
            Some(v.verifier_params)
        } else {
            None
        };

        let start = std::time::Instant::now();
        let perm = load_permutation_params();
        println!(
            "load_permutation_params time: {:.2?}, {}",
            start.elapsed(),
            perm.is_some()
        );

        let start = std::time::Instant::now();
        let prover_params =
            indexer_with_lagrange(&cs, &pcs, lagrange_pcs.as_ref(), perm, verifier_params).unwrap();
        println!("indexer_with_lagrange time: {:.2?}", start.elapsed());

        Ok(ProverParams {
            pcs,
            lagrange_pcs,
            cs,
            prover_params,
        })
    }
}

impl VerifierParams {
    /// Load the verifier parameters.
    pub fn get() -> Result<VerifierParams> {
        match Self::load() {
            Ok(vk) => Ok(vk),
            Err(_e) => Ok(Self::from(ProverParams::gen()?)),
        }
    }

    /// Load the verifier parameters from prepare.
    pub fn load() -> Result<VerifierParams> {
        match (VERIFIER_COMMON_PARAMS, VERIFIER_SPECIFIC_PARAMS) {
            (Some(c_bytes), Some(s_bytes)) => {
                let common: VerifierParamsSplitCommon =
                    bincode::deserialize(c_bytes).map_err(|_| SetUpError::DeserializationError)?;

                let special: VerifierParamsSplitSpecific =
                    bincode::deserialize(s_bytes).map_err(|_| SetUpError::DeserializationError)?;

                Ok(VerifierParams {
                    shrunk_vk: common.shrunk_pcs,
                    shrunk_cs: special.shrunk_cs,
                    verifier_params: special.verifier_params,
                })
            }
            _ => Err(SetUpError::MissingVerifierParamsError),
        }
    }

    /// Split the verifier parameters to the common part and the sspecific part.
    pub fn split(self) -> Result<(VerifierParamsSplitCommon, VerifierParamsSplitSpecific)> {
        Ok((
            VerifierParamsSplitCommon {
                shrunk_pcs: self.shrunk_vk.shrink_to_verifier_only().unwrap(),
            },
            VerifierParamsSplitSpecific {
                shrunk_cs: self.shrunk_cs.shrink_to_verifier_only(),
                verifier_params: self.verifier_params,
            },
        ))
    }
}

impl From<ProverParams> for VerifierParams {
    fn from(params: ProverParams) -> Self {
        VerifierParams {
            shrunk_vk: params.pcs.shrink_to_verifier_only().unwrap(),
            shrunk_cs: params.cs.shrink_to_verifier_only(),
            verifier_params: params.prover_params.get_verifier_params(),
        }
    }
}

pub fn load_lagrange_params(size: usize) -> Option<KZGCommitmentSchemeBN254> {
    match LAGRANGE_BASES.get(&size) {
        None => None,
        Some(bytes) => KZGCommitmentSchemeBN254::from_unchecked_bytes(&bytes).ok(),
    }
}

pub fn load_permutation_params() -> Option<Vec<usize>> {
    match PERMUTATION {
        None => None,
        Some(bytes) => {
            let common: Vec<usize> = bincode::deserialize(bytes).unwrap();

            Some(common)
        }
    }
}

pub fn load_srs_params(size: usize) -> Result<KZGCommitmentSchemeBN254> {
    let srs = SRS.ok_or(SetUpError::MissingSRSError)?;

    let KZGCommitmentSchemeBN254 {
        public_parameter_group_1,
        public_parameter_group_2,
    } = KZGCommitmentSchemeBN254::from_unchecked_bytes(&srs)
        .map_err(|_| SetUpError::DeserializationError)?;

    let mut new_group_1 = vec![G1Projective::default(); core::cmp::max(size + 3, 2051)];
    new_group_1[0..2051].copy_from_slice(&public_parameter_group_1[0..2051]);

    if size == 4096 {
        new_group_1[4096..4099].copy_from_slice(&public_parameter_group_1[2051..2054]);
    }

    if size == 8192 {
        new_group_1[8192..8195].copy_from_slice(&public_parameter_group_1[2054..2057]);
    }

    if size == 16384 {
        new_group_1[16384..16387].copy_from_slice(&public_parameter_group_1[2057..2060]);
    }

    if size == 65536 {
        new_group_1[65536..65539].copy_from_slice(&public_parameter_group_1[2063..2066]);
    }

    if size == 1048576 {
        new_group_1[1048576..1048579].copy_from_slice(&public_parameter_group_1[2075..2078]);
    }

    // if size > 16384 {
    //     return Err(SetUpError::ParameterError);
    // }

    Ok(KZGCommitmentSchemeBN254 {
        public_parameter_group_2,
        public_parameter_group_1: new_group_1,
    })
}
