use std::ops::Mul;

use ark_bn254::Fr;
use ark_ec::AdditiveGroup;
use ark_ec::{CurveGroup, PrimeGroup};
use ark_ed_on_bn254::EdwardsProjective;
use ark_ff::{BigInteger, Field, PrimeField};
use num_bigint::BigUint;
use num_integer::Integer;
use zplonk::{
    anemoi::{AnemoiJive, AnemoiJive254},
    chaum_pedersen::dl::ChaumPedersenDLProof,
    turboplonk::constraint_system::{ecc::PointVar, turbo::TurboCS, VarIndex},
};
use zshuffle::{MaskedCard, RevealCard, RevealProof};

use super::public_keys::PublicKeyOutsource;

#[derive(Default)]
pub struct RevealOutsource {
    // e1
    pub masked_card: MaskedCard,
    pub reveal_cards: Vec<RevealCard>,
    pub proofs: Vec<ChaumPedersenDLProof>,

    pub masked_card_var: (PointVar, PointVar),
    pub reveal_card_vars: Vec<PointVar>,
    pub proof_vars: Vec<(PointVar, PointVar, VarIndex)>,
}

impl RevealOutsource {
    pub fn new(
        masked_card: &MaskedCard,
        reveal_cards: &[RevealCard],
        proofs: &[RevealProof],
    ) -> Self {
        assert_eq!(reveal_cards.len(), proofs.len());

        Self {
            masked_card: masked_card.clone(),
            reveal_cards: reveal_cards.to_vec(),
            proofs: proofs.to_vec(),
            masked_card_var: (PointVar::default(), PointVar::default()),
            reveal_card_vars: vec![],
            proof_vars: vec![],
        }
    }

    pub fn generate_constraints(&mut self, cs: &mut TurboCS<Fr>, pks: &PublicKeyOutsource) {
        let n = pks.affine_reps.len();
        let zero = Fr::ZERO;
        let one = Fr::ONE;
        let zero_var = cs.zero_var();

        let g = EdwardsProjective::generator();
        let g_aff = g.into_affine();
        let g_var = cs.new_point_variable(g); // todo set g is constanst

        // let pk_aff = pk.into_affine();
        // let pk_var = cs.new_point_variable(*pk);

        let masked_card_aff = self.masked_card.e1.into_affine();
        self.masked_card_var = (
            cs.new_point_variable(self.masked_card.e1),
            cs.new_point_variable(self.masked_card.e2),
        );

        let m_bytes = [
            6, 12, 137, 206, 92, 38, 52, 5, 55, 10, 8, 182, 208, 48, 43, 11, 171, 62, 237, 184, 57,
            32, 238, 10, 103, 114, 151, 220, 57, 33, 38, 241,
        ];
        let m_field = Fr::from_be_bytes_mod_order(&m_bytes);
        // let m_var = cs.new_variable(m_field); // todo constant
        let m = BigUint::from_bytes_be(&m_bytes);

        for (i, (reveal_card, proof)) in
            self.reveal_cards.iter().zip(self.proofs.iter()).enumerate()
        {
            let i = i % n;
            let reveal_card_aff = reveal_card.into_affine();
            let proof_a_aff = proof.a.into_affine();
            let proof_b_aff = proof.b.into_affine();

            let reveal_card_var = cs.new_point_variable(*reveal_card);
            let proof_a_var = cs.new_point_variable(proof.a);
            let proof_b_var = cs.new_point_variable(proof.b);

            //todo  push to self
            self.reveal_card_vars.push(reveal_card_var);

            let inputs = vec![
                masked_card_aff.x,
                masked_card_aff.y,
                g_aff.x,
                g_aff.y,
                reveal_card_aff.x,
                reveal_card_aff.y,
                pks.affine_reps[i].x,
                pks.affine_reps[i].y,
                proof_a_aff.x,
                proof_a_aff.y,
                proof_b_aff.x,
                proof_b_aff.y,
            ];
            let input_vars = vec![
                self.masked_card_var.0.get_x(),
                self.masked_card_var.0.get_y(),
                g_var.get_x(),
                g_var.get_y(),
                reveal_card_var.get_x(),
                reveal_card_var.get_y(),
                pks.cs_vars[i].get_x(),
                pks.cs_vars[i].get_y(),
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
            let (quotient, remainder) = n.div_rem(&m);

            let quotient = Fr::from(quotient);
            let remainder_254 = Fr::from(remainder.clone());
            let remainder_251 = ark_ed_on_bn254::Fr::from(remainder);

            let quotient_var = cs.new_variable(quotient);
            let remainder_var = cs.new_variable(remainder_254);
            cs.range_check(remainder_var, 251); // todo

            cs.push_add_selectors(m_field, one, zero, zero);
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
                self.masked_card_var.0,
                self.masked_card.e1,
                scalar_var,
                256,
            );

            let base = reveal_card.mul(&remainder_251);
            let out_1 =
                cs.nonconst_base_scalar_mul(reveal_card_var, *reveal_card, remainder_var, 256);
            let out_2 = cs.ecc_add(&out_1, &proof_a_var, &base, &proof.a);

            cs.equal(out_0.get_x(), out_2.get_var().get_x());
            cs.equal(out_0.get_y(), out_2.get_var().get_y());

            let out_3 = cs.const_base_scalar_mul(g, scalar_var, 256);
            let base = pks.public_keys[i].mul(&remainder_251);
            let out_4 =
                cs.nonconst_base_scalar_mul(pks.cs_vars[i], pks.public_keys[i], remainder_var, 256);
            let out_5 = cs.ecc_add(&out_4, &proof_b_var, &base, &proof.b);

            cs.equal(out_3.get_x(), out_5.get_var().get_x());
            cs.equal(out_3.get_y(), out_5.get_var().get_y());
        }
    }
}

#[cfg(test)]
mod test {
    use super::RevealOutsource;
    use crate::prove_outsource::public_keys::PublicKeyOutsource;
    use crate::task::mock_task;
    use ark_bn254::Fr;
    use zplonk::{anemoi::AnemoiJive254, turboplonk::constraint_system::turbo::TurboCS};

    #[test]
    fn test_reveals_constraint_system() {
        let task = mock_task();
        let env = &task.players_env[0][0];
        let card = env.play_cards.clone().unwrap().to_vec()[0];
        let reveals = &env.reveals[0];

        let reveal_cards = reveals.iter().map(|x| x.0 .0).collect::<Vec<_>>();
        let reveal_proofs = reveals.iter().map(|x| x.1).collect::<Vec<_>>();
        let mut reveal_outsource = RevealOutsource::new(&card.0, &reveal_cards, &reveal_proofs);

        let mut cs = TurboCS::<Fr>::new();
        cs.load_anemoi_parameters::<AnemoiJive254>();

        let pk_outsource = PublicKeyOutsource::new(&mut cs, &task.players_keys);
        reveal_outsource.generate_constraints(&mut cs, &pk_outsource);

        let witness = cs.get_and_clear_witness();
        cs.verify_witness(&witness, &[]).unwrap();

        assert_eq!(cs.size, 16904);
    }
}
