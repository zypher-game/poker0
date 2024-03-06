use crate::{
    build_cs::{build_cs, N_CARDS, N_PLAYS},
    create_and_rescale_outsource,
    gen_params::VERIFIER_SPECIFIC_PARAMS,
};
use ark_bn254::{Fr, G1Projective};
use plonk::{
    errors::{PlonkError, Result},
    poly_commit::{kzg_poly_commitment::KZGCommitmentSchemeBN254, pcs::PolyComScheme},
    turboplonk::{
        constraint_system::{turbo::TurboCS, ConstraintSystem},
        indexer::{indexer_with_lagrange, PlonkProverParams, PlonkVerifierParams},
    },
};
use poker_core::mock_data::task::mock_task;
use serde::{Deserialize, Serialize};

use super::{LAGRANGE_BASES, PERMUTATION, SRS, VERIFIER_COMMON_PARAMS};

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
    /// Obtain the parameters.
    pub fn gen() -> Result<ProverParams> {
        let (players_keys, reveal_outsources, unmask_outsources) =
            create_and_rescale_outsource(&mock_task(), N_PLAYS, N_CARDS);

        let cs = build_cs(&players_keys, &reveal_outsources, &unmask_outsources);
        let pcs = load_srs_params(cs.size())?;
        let lagrange_pcs = load_lagrange_params(cs.size());

        let verifier_params = if let Ok(v) = VerifierParams::load() {
            Some(v.verifier_params)
        } else {
            None
        };

        let perm = load_permutation_params();

        let prover_params =
            indexer_with_lagrange(&cs, &pcs, lagrange_pcs.as_ref(), perm, verifier_params).unwrap();

        Ok(ProverParams {
            pcs,
            lagrange_pcs,
            cs,
            prover_params,
        })
    }
}

impl VerifierParams {
    /// Get the verifier parameters.
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
                    bincode::deserialize(c_bytes).map_err(|_| PlonkError::DeserializationError)?;

                let special: VerifierParamsSplitSpecific =
                    bincode::deserialize(s_bytes).map_err(|_| PlonkError::DeserializationError)?;

                Ok(VerifierParams {
                    shrunk_vk: common.shrunk_pcs,
                    shrunk_cs: special.shrunk_cs,
                    verifier_params: special.verifier_params,
                })
            }
            _ => Err(PlonkError::MissingVerifierParamsError),
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
    if size != 1048576 {
        return Err(PlonkError::ParameterError);
    }

    let srs = SRS.ok_or(PlonkError::MissingSRSError)?;

    let KZGCommitmentSchemeBN254 {
        public_parameter_group_1,
        public_parameter_group_2,
    } = KZGCommitmentSchemeBN254::from_unchecked_bytes(&srs)
        .map_err(|_| PlonkError::DeserializationError)?;

    let mut new_group_1 = vec![G1Projective::default(); core::cmp::max(size + 3, 2051)];
    new_group_1[0..2051].copy_from_slice(&public_parameter_group_1[0..2051]);

    if size == 1048576 {
        new_group_1[1048576..1048579].copy_from_slice(&public_parameter_group_1[2075..2078]);
    }

    Ok(KZGCommitmentSchemeBN254 {
        public_parameter_group_2,
        public_parameter_group_1: new_group_1,
    })
}
