use std::ops::Mul;

use ark_bn254::Fr;
use ark_ec::AdditiveGroup;
use ark_ec::{CurveGroup, PrimeGroup};
use ark_ed_on_bn254::EdwardsProjective;
use ark_ff::{BigInteger, Field, PrimeField};
use num_bigint::BigUint;
use num_integer::Integer;
use poker_core::cards::{CryptoCard, RevealCard};
use zplonk::{
    anemoi::{AnemoiJive, AnemoiJive254},
    chaum_pedersen::dl::ChaumPedersenDLProof,
    turboplonk::constraint_system::{ecc::PointVar, turbo::TurboCS},
};
use zshuffle::RevealProof;

use crate::get_divisor;

use super::public_keys::PublicKeyOutsource;

#[derive(Default)]
pub struct RevealOutsource {
    pub crypto_card: CryptoCard,
    pub reveal_cards: Vec<RevealCard>,
    pub proofs: Vec<ChaumPedersenDLProof>,

    pub crypto_card_var: (PointVar, PointVar),
    pub reveal_card_vars: Vec<PointVar>,
}

impl RevealOutsource {
    pub fn new(
        crypto_card: &CryptoCard,
        reveal_cards: &[RevealCard],
        proofs: &[RevealProof],
    ) -> Self {
        assert_eq!(reveal_cards.len(), proofs.len());

        Self {
            crypto_card: crypto_card.clone(),
            reveal_cards: reveal_cards.to_vec(),
            proofs: proofs.to_vec(),
            crypto_card_var: (PointVar::default(), PointVar::default()),
            reveal_card_vars: vec![],
        }
    }

    pub fn generate_constraints(
        &mut self,
        cs: &mut TurboCS<Fr>,
        pk_outsource: &PublicKeyOutsource,
    ) {
        assert_eq!(pk_outsource.public_keys.len(), self.reveal_cards.len());
        let zero = Fr::ZERO;
        let one = Fr::ONE;
        let zero_var = cs.zero_var();

        let g = EdwardsProjective::generator();
        let g_aff = g.into_affine();
        let g_var = cs.new_point_variable(g);
        cs.insert_constant_gate(g_var.get_x(), g_aff.x);
        cs.insert_constant_gate(g_var.get_y(), g_aff.y);

        let m = get_divisor();

        self.crypto_card_var = (
            cs.new_point_variable(self.crypto_card.0.e1.into()),
            cs.new_point_variable(self.crypto_card.0.e2.into()),
        );

        for (reveal_card, (proof, (public_key, pk_var))) in self.reveal_cards.iter().zip(
            self.proofs.iter().zip(
                pk_outsource
                    .public_keys
                    .iter()
                    .zip(pk_outsource.cs_vars.iter()),
            ),
        ) {
            let proof_a_aff = proof.a.into_affine();
            let proof_b_aff = proof.b.into_affine();
            let proof_a_var = cs.new_point_variable(proof.a);
            let proof_b_var = cs.new_point_variable(proof.b);

            let reveal_card_var = cs.new_point_variable(reveal_card.0.into());
            self.reveal_card_vars.push(reveal_card_var);

            let inputs = vec![
                self.crypto_card.0.e1.x,
                self.crypto_card.0.e1.y,
                g_aff.x,
                g_aff.y,
                reveal_card.0.x,
                reveal_card.0.y,
                public_key.0.x,
                public_key.0.y,
                proof_a_aff.x,
                proof_a_aff.y,
                proof_b_aff.x,
                proof_b_aff.y,
            ];
            let input_vars = vec![
                self.crypto_card_var.0.get_x(),
                self.crypto_card_var.0.get_y(),
                g_var.get_x(),
                g_var.get_y(),
                reveal_card_var.get_x(),
                reveal_card_var.get_y(),
                pk_var.get_x(),
                pk_var.get_y(),
                proof_a_var.get_x(),
                proof_a_var.get_y(),
                proof_b_var.get_x(),
                proof_b_var.get_y(),
            ];
            let trace = AnemoiJive254::eval_variable_length_hash_with_trace(&inputs);
            let output = trace.output;
            let output_var = cs.new_variable(output);
            cs.anemoi_variable_length_hash::<AnemoiJive254>(&trace, &input_vars, output_var);

            let n: BigUint = output.into();
            let (quotient, remainder) = n.div_rem(&m.1);

            let quotient = Fr::from(quotient);
            let remainder_254 = Fr::from(remainder.clone());
            let remainder_251 = ark_ed_on_bn254::Fr::from(remainder);

            let quotient_var = cs.new_variable(quotient);
            let remainder_var = cs.new_variable(remainder_254);
            cs.range_check(remainder_var, 251);

            cs.push_add_selectors(m.0, one, zero, zero);
            cs.push_mul_selectors(zero, zero);
            cs.push_constant_selector(zero);
            cs.push_ecc_selector(zero);
            cs.push_out_selector(one);

            cs.wiring[0].push(quotient_var);
            cs.wiring[1].push(remainder_var);
            cs.wiring[2].push(zero_var);
            cs.wiring[3].push(zero_var);
            cs.wiring[4].push(output_var);

            cs.finish_new_gate();

            let scalar = Fr::from_be_bytes_mod_order(&proof.r.into_bigint().to_bytes_be());
            let scalar_var = cs.new_variable(scalar);
            let out_0 = cs.nonconst_base_scalar_mul(
                self.crypto_card_var.0,
                self.crypto_card.0.e1.into(),
                scalar_var,
                251,
            );

            let base = reveal_card.0.mul(&remainder_251);
            let out_1 = cs.nonconst_base_scalar_mul(
                reveal_card_var,
                reveal_card.0.into(),
                remainder_var,
                251,
            );
            let out_2 = cs.ecc_add(&out_1, &proof_a_var, &base, &proof.a);

            cs.equal(out_0.get_x(), out_2.get_var().get_x());
            cs.equal(out_0.get_y(), out_2.get_var().get_y());

            let out_3 = cs.const_base_scalar_mul(g, scalar_var, 252);
            let base = public_key.0.mul(&remainder_251);
            let out_4 =
                cs.nonconst_base_scalar_mul(*pk_var, public_key.0.into(), remainder_var, 251);
            let out_5 = cs.ecc_add(&out_4, &proof_b_var, &base, &proof.b);

            cs.equal(out_3.get_x(), out_5.get_var().get_x());
            cs.equal(out_3.get_y(), out_5.get_var().get_y());
        }
    }

    pub fn prepare_pi_variables(&self, cs: &mut TurboCS<Fr>) {
        cs.prepare_pi_point_variable(self.crypto_card_var.0);
        cs.prepare_pi_point_variable(self.crypto_card_var.1);
    }
}

#[cfg(test)]
mod test {
    use super::RevealOutsource;
    use crate::public_keys::PublicKeyOutsource;
    use ark_bn254::Fr;
    use poker_core::mock_data::task::mock_task;
    use zplonk::{anemoi::AnemoiJive254, turboplonk::constraint_system::turbo::TurboCS};

    #[test]
    fn test_reveals_constraint_system() {
        let task = mock_task();
        let env = &task.players_env[0][0];
        let card = env.play_cards.clone().unwrap().to_vec()[0];
        let reveals = &env.reveals[0];

        let reveal_cards = reveals.iter().map(|x| x.0).collect::<Vec<_>>();
        let reveal_proofs = reveals.iter().map(|x| x.1).collect::<Vec<_>>();
        let mut reveal_outsource = RevealOutsource::new(&card, &reveal_cards, &reveal_proofs);

        let mut cs = TurboCS::<Fr>::new();
        cs.load_anemoi_parameters::<AnemoiJive254>();
        let pk_outsource = PublicKeyOutsource::new(&mut cs, &task.players_keys);

        reveal_outsource.generate_constraints(&mut cs, &pk_outsource);
        reveal_outsource.prepare_pi_variables(&mut cs);

        let witness = cs.get_and_clear_witness();
        cs.verify_witness(
            &witness,
            &[card.0.e1.x, card.0.e1.y, card.0.e2.x, card.0.e2.y],
        )
        .unwrap();

        assert_eq!(cs.size, 16604);
    }
}
