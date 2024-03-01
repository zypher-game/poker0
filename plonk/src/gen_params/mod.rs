use ark_std::collections::BTreeMap;

use ark_bn254::G1Projective;
use lazy_static::lazy_static;
use zplonk::{errors::ZplonkError, poly_commit::kzg_poly_commitment::KZGCommitmentSchemeBN254};

pub mod params;

#[cfg(not(feature = "no_vk"))]
/// The specific part of the verifier parameters.
pub static VERIFIER_SPECIFIC_PARAMS: Option<&'static [u8]> =
    Some(include_bytes!("../../parameters/vk-specific.bin"));

#[cfg(feature = "no_vk")]
/// The specific part of the verifier parameters.
pub static VERIFIER_SPECIFIC_PARAMS: Option<&'static [u8]> = None;

#[cfg(not(feature = "no_srs"))]
/// The SRS.
pub static SRS: Option<&'static [u8]> = Some(include_bytes!("../../parameters/srs-padding.bin"));

#[cfg(feature = "no_srs")]
/// The SRS.
pub static SRS: Option<&'static [u8]> = None;

#[cfg(feature = "no_srs")]
lazy_static! {
    /// The Lagrange format of the SRS.
    pub static ref LAGRANGE_BASES: BTreeMap<usize, &'static [u8]> = BTreeMap::default();
}

#[cfg(all(not(feature = "no_srs"), not(feature = "lightweight")))]
static LAGRANGE_BASE_1048576: &'static [u8] =
    include_bytes!("../../parameters/lagrange-srs-1048576.bin");

#[cfg(not(feature = "no_srs"))]
lazy_static! {
    /// The Lagrange format of the SRS.
    pub static ref LAGRANGE_BASES: BTreeMap<usize, &'static [u8]> = {
        let mut m = BTreeMap::new();
        #[cfg(not(feature = "lightweight"))]
        {
            m.insert(1048576, LAGRANGE_BASE_1048576);
        }

        m
    };
}

pub fn load_lagrange_params(size: usize) -> Option<KZGCommitmentSchemeBN254> {
    match LAGRANGE_BASES.get(&size) {
        None => None,
        Some(bytes) => KZGCommitmentSchemeBN254::from_unchecked_bytes(&bytes).ok(),
    }
}

pub fn load_srs_params(size: usize) -> Result<KZGCommitmentSchemeBN254, ZplonkError> {
    if size > 1048576 {
        return Err(ZplonkError::ParameterError);
    }

    let srs = SRS.ok_or(ZplonkError::MissingSRSError)?;

    let KZGCommitmentSchemeBN254 {
        public_parameter_group_1,
        public_parameter_group_2,
    } = KZGCommitmentSchemeBN254::from_unchecked_bytes(&srs)
        .map_err(|_| ZplonkError::DeserializationError)?;
    assert_eq!(public_parameter_group_1.len(), 3);

    let mut new_group_1 = vec![G1Projective::default(); size + 3];
    new_group_1[1048576..1048579].copy_from_slice(&public_parameter_group_1[0..3]);

    Ok(KZGCommitmentSchemeBN254 {
        public_parameter_group_2,
        public_parameter_group_1: new_group_1,
    })
}
