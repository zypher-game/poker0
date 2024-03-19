use ark_bn254::{Fq, Fr, G1Affine};
use ark_ec::AffineRepr;
use ark_ff::{AdditiveGroup, BigInteger, Field, PrimeField};
use num_bigint::BigUint;
use plonk::{
    poly_commit::{field_polynomial::FpPolynomial, kzg_poly_commitment::KZGCommitmentSchemeBN254},
    turboplonk::indexer::{PlonkProof, PlonkVerifierParams},
};
use poker_core::{play::PlayAction, schnorr::PublicKey, task::Task};
use reveals::RevealOutsource;
use std::{fs, fs::File, io::Write, path::Path};
use unmask::UnmaskOutsource;

use crate::signatures::SignatureOutsource;

pub mod build_cs;
pub mod gen_params;
pub mod public_keys;
pub mod reveals;
pub mod signatures;
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
) -> (
    Vec<PublicKey>,
    Vec<RevealOutsource>,
    Vec<UnmaskOutsource>,
    Vec<SignatureOutsource>,
) {
    let mut reveal_outsources = vec![];
    let mut unmask_outsources = vec![];
    let mut signature_outsources = vec![];

    for plays in task.players_env.iter() {
        for env in plays.iter() {
            let signature_outsource = SignatureOutsource::new(&env.signature, &env.pack());
            signature_outsources.push(signature_outsource);

            if let PlayAction::PLAY = env.action {
                let crypto_cards = env.play_crypto_cards.clone().unwrap().to_vec();
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
        task.players_key.clone(),
        reveal_outsources,
        unmask_outsources,
        signature_outsources,
    )
}

pub fn create_and_rescale_outsource(
    task: &Task,
    n_players: usize,
    n_cards: usize,
) -> (
    Vec<PublicKey>,
    Vec<RevealOutsource>,
    Vec<UnmaskOutsource>,
    Vec<SignatureOutsource>,
) {
    let (public_keys, mut reveal_outsources, mut unmask_outsources, mut signature_outsources) =
        create_outsource(task);

    let n = reveal_outsources.len();
    let m = n % n_players;
    reveal_outsources.extend_from_slice(&reveal_outsources.clone()[m..(n_cards - 2 - n + m)]);
    unmask_outsources.extend_from_slice(&unmask_outsources.clone()[m..(n_cards - 2 - n + m)]);

    let n = signature_outsources.len();
    let m = n % n_players;
    signature_outsources.extend_from_slice(&signature_outsources.clone()[m..(n_cards - 2 - n + m)]);

    (
        public_keys,
        reveal_outsources,
        unmask_outsources,
        signature_outsources,
    )
}

pub fn export_solidity_vk(verifier_params: &PlonkVerifierParams<KZGCommitmentSchemeBN254>) {
    let dir = "solidity";
    if !Path::new(dir).exists() {
        fs::create_dir(dir).unwrap();
    }

    let mut file = File::create(format!("{}/VerifierKey.sol", dir)).unwrap();

    let mut content = String::from("// SPDX-License-Identifier: UNLICENSED\n");
    content.push_str("pragma solidity ^0.8.20;\n");
    content.push_str("\n");
    content.push_str(&format!("library {} ", format!("VerifierKey")));
    content.push_str("{\n");

    content.push_str("function load(uint256 vk,uint256 pi) internal pure  {\n");
    content.push_str("assembly {\n");

    let mut index = 0x00;

    // The commitments of the selectors.
    content.push_str("// The commitments of the selectors (9).\n");
    for cm_q in verifier_params.cm_q_vec.iter() {
        let tmp: G1Affine = cm_q.0.into();
        let x = tmp.x().unwrap_or(Fq::ZERO);
        let x = x.into_bigint().to_string();

        let y = tmp.y().unwrap_or(Fq::ZERO);
        let y = y.into_bigint().to_string();

        content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, x));
        index += 0x20;
        content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, y));
        index += 0x20;
    }
    content.push_str("\n");

    // The commitments of perm1, perm2, ..., perm_{n_wires_per_gate}.
    content.push_str("// The commitments of perm1, perm2, ..., perm_{n_wires_per_gate}.\n");
    for cm_s in verifier_params.cm_s_vec.iter() {
        let tmp: G1Affine = cm_s.0.into();
        let x = tmp.x().unwrap_or(Fq::ZERO);
        let x = x.into_bigint().to_string();

        let y = tmp.y().unwrap_or(Fq::ZERO);
        let y = y.into_bigint().to_string();

        content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, x));
        index += 0x20;
        content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, y));
        index += 0x20;
    }
    content.push_str("\n");

    // The commitment of the boolean selector.
    content.push_str("// The commitment of the boolean selector.\n");
    {
        let tmp: G1Affine = verifier_params.cm_qb.0.into();
        let x = tmp.x().unwrap_or(Fq::ZERO);
        let x = x.into_bigint().to_string();

        let y = tmp.y().unwrap_or(Fq::ZERO);
        let y = y.into_bigint().to_string();

        content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, x));
        index += 0x20;
        content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, y));
        index += 0x20;
    }
    content.push_str("\n");

    // The commitments of the preprocessed round key selectors.
    content.push_str("// The commitments of the preprocessed round key selectors.\n");
    for cm_qrk in verifier_params.cm_prk_vec.iter() {
        let tmp: G1Affine = cm_qrk.0.into();
        let x = tmp.x().unwrap_or(Fq::ZERO);
        let x = x.into_bigint().to_string();

        let y = tmp.y().unwrap_or(Fq::ZERO);
        let y = y.into_bigint().to_string();

        content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, x));
        index += 0x20;
        content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, y));
        index += 0x20;
    }
    content.push_str("\n");

    // The Anemoi generator.
    content.push_str("// The Anemoi generator.\n");
    let x = verifier_params.anemoi_generator.into_bigint().to_string();
    content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, x));
    index += 0x20;
    content.push_str("\n");

    // The Anemoi generator's inverse.
    content.push_str("// The Anemoi generator's inverse.\n");
    let x = verifier_params
        .anemoi_generator_inv
        .into_bigint()
        .to_string();
    content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, x));
    index += 0x20;
    content.push_str("\n");

    // `n_wires_per_gate` different quadratic non-residue in F_q-{0}.
    content.push_str("// `n_wires_per_gate` different quadratic non-residue in F_q-{0}.\n");
    for k in verifier_params.k.iter() {
        let x = k.into_bigint().to_string();

        content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, x));
        index += 0x20;
    }
    content.push_str("\n");

    // The domain's group generator with csSize.
    content.push_str("// The domain's group generator with csSize.\n");
    let domain = FpPolynomial::<Fr>::evaluation_domain(verifier_params.cs_size).unwrap();
    let root = domain.group_gen;
    let x = root.into_bigint().to_string();
    content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, x));
    index += 0x20;
    content.push_str("\n");

    // The size of constraint system.
    content.push_str("// The size of constraint system.\n");
    let x = verifier_params.cs_size;
    content.push_str(&format!("mstore(add(vk,0x{:x}), {})\n", index, x));
    content.push_str("\n");

    index = 0x00;
    content.push_str(&format!(
        "mstore(add(pi,0x{:x}), {})\n",
        index,
        verifier_params.public_vars_constraint_indices.len()
    ));
    index += 0x20;

    // The public constrain variables indices.
    content.push_str("// The public constrain variables indices.\n");
    for k in verifier_params.public_vars_constraint_indices.iter() {
        let root_pow = root.pow(&[*k as u64]);
        let x = root_pow.into_bigint().to_string();
        content.push_str(&format!("mstore(add(pi,0x{:x}), {})\n", index, x));
        index += 0x20;
    }
    content.push_str("\n");

    // The constrain lagrange base by public constrain variables.
    content.push_str("// The constrain lagrange base by public constrain variables.\n");
    for k in verifier_params.lagrange_constants.iter() {
        let x = k.into_bigint().to_string();
        content.push_str(&format!("mstore(add(pi,0x{:x}), {})\n", index, x));
        index += 0x20;
    }
    content.push_str("\n");

    content.push_str("}\n");
    content.push_str("}\n");
    content.push_str("}\n");

    file.write_all(content.as_bytes()).unwrap();
}

pub fn export_solidity_proof(proof: &PlonkProof<KZGCommitmentSchemeBN254>) -> String {
    fn fr_to_hex<F: PrimeField>(x: &F) -> String {
        let x = x.into_bigint().to_bytes_be();
        let code = hex::encode(&x);
        code
    }

    let mut res = String::from("0x");

    for cm_q in proof.cm_w_vec.iter() {
        let tmp: G1Affine = cm_q.0.into();

        let x = tmp.x().unwrap_or(Fq::ZERO);
        let x = fr_to_hex(&x);
        res += &x;

        let y = tmp.y().unwrap_or(Fq::ZERO);
        let y = fr_to_hex(&y);
        res += &y;
    }

    for cm_t in proof.cm_t_vec.iter() {
        let tmp: G1Affine = cm_t.0.into();

        let x = tmp.x().unwrap_or(Fq::ZERO);
        let x = fr_to_hex(&x);
        res += &x;

        let y = tmp.y().unwrap_or(Fq::ZERO);
        let y = fr_to_hex(&y);
        res += &y;
    }

    {
        let tmp: G1Affine = proof.cm_z.0.into();

        let x = tmp.x().unwrap_or(Fq::ZERO);
        let x = fr_to_hex(&x);
        res += &x;

        let y = tmp.y().unwrap_or(Fq::ZERO);
        let y = fr_to_hex(&y);
        res += &y;
    }

    {
        let x = fr_to_hex(&proof.prk_3_poly_eval_zeta);
        res += &x;
    }

    {
        let x = fr_to_hex(&proof.prk_4_poly_eval_zeta);
        res += &x;
    }

    for x in proof.w_polys_eval_zeta.iter() {
        let x = fr_to_hex(x);
        res += &x;
    }

    for x in proof.w_polys_eval_zeta_omega.iter() {
        let x = fr_to_hex(x);
        res += &x;
    }

    {
        let x = fr_to_hex(&proof.z_eval_zeta_omega);
        res += &x;
    }

    for x in proof.s_polys_eval_zeta.iter() {
        let x = fr_to_hex(x);
        res += &x;
    }

    {
        let tmp: G1Affine = proof.opening_witness_zeta.0.into();

        let x = tmp.x().unwrap_or(Fq::ZERO);
        let x = fr_to_hex(&x);
        res += &x;

        let y = tmp.y().unwrap_or(Fq::ZERO);
        let y = fr_to_hex(&y);
        res += &y;
    }

    {
        let tmp: G1Affine = proof.opening_witness_zeta_omega.0.into();

        let x = tmp.x().unwrap_or(Fq::ZERO);
        let x = fr_to_hex(&x);
        res += &x;

        let y = tmp.y().unwrap_or(Fq::ZERO);
        let y = fr_to_hex(&y);
        res += &y;
    }

    res
}
