use ark_bn254::Fr;
use ark_ec::CurveGroup;
use plonk::{
    anemoi::AnemoiJive254,
    errors::Result,
    poly_commit::{kzg_poly_commitment::KZGCommitmentSchemeBN254, transcript::Transcript},
    turboplonk::{
        constraint_system::turbo::TurboCS, indexer::PlonkProof, prover::prover_with_lagrange,
        verifier::verifier,
    },
};
use poker_core::schnorr::PublicKey;
use rand_chacha::rand_core::{CryptoRng, RngCore};

use crate::{
    gen_params::params::{ProverParams, VerifierParams}, public_keys::PublicKeyOutsource, reveals::RevealOutsource, signatures::{self, SignatureOutsource}, unmask::UnmaskOutsource
};

pub type Proof = PlonkProof<KZGCommitmentSchemeBN254>;

const PLONK_PROOF_TRANSCRIPT: &[u8] = b"Plonk poker Proof";

pub const N_CARDS: usize = 52;
pub const N_PLAYS: usize = 3;

pub fn build_cs(
    public_keys: &[PublicKey],
    reveal_outsources: &[RevealOutsource],
    unmask_outsources: &[UnmaskOutsource],
    signature_outsources: &[SignatureOutsource],
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


    // let play_card_vars = reveal_outsources.iter()


    for (i,signature_outsource) in signature_outsources.iter_mut().enumerate() {
        signature_outsource.generate_constraints(&mut cs, &pk_outsources.public_keys[i],&pk_outsources.cs_vars[i]);
        signature_outsource.prepare_pi_variables(&mut cs);
    }

    cs.pad();

    cs
}

pub fn prove_outsource<R: CryptoRng + RngCore>(
    prng: &mut R,
    public_keys: &[PublicKey],
    reveal_outsources: &[RevealOutsource],
    unmask_outsources: &[UnmaskOutsource],
    signature_outsources: &[SignatureOutsource],
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
    )
    .unwrap();

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
    )
    .unwrap())
}

#[cfg(test)]
mod test {
    use crate::create_outsource;
    use ark_ec::CurveGroup;
    use poker_core::mock_data::task::mock_task;

    use super::build_cs;

    #[test]
    fn test_build_cs() {
        let (public_keys, reveal_outsources, unmask_outsources) = create_outsource(&mock_task());

        let mut cs = build_cs(&public_keys, &reveal_outsources, &unmask_outsources);

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

        let witness = cs.get_and_clear_witness();
        cs.verify_witness(&witness, &online_inputs).unwrap();
    }
}
