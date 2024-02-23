use std::ops::Mul;

use ark_bn254::Fr;
use ark_ec::{AdditiveGroup, CurveGroup, PrimeGroup};
use ark_ed_on_bn254::EdwardsProjective;
use ark_ff::{BigInteger, Field, PrimeField};
use num_bigint::BigUint;
use num_integer::Integer;
use zplonk::{
    anemoi::{AnemoiJive, AnemoiJive254},
    turboplonk::constraint_system::{ecc::PointVar, turbo::TurboCS, VarIndex},
};

use poker_core::{cards::CryptoCard, play::MAX_PLAYER_HAND_LEN, schnorr::Signature};

use super::public_keys::PublicKeyOutsource;

pub struct SignatureOutsource {
    pub signature: Signature,
    pub message: u128,
    pub play_cards: Option<Vec<CryptoCard>>,

    pub signature_var: (VarIndex, VarIndex),
    pub message_var: VarIndex,
    pub play_card_vars: Option<Vec<(PointVar, PointVar)>>,
}

impl SignatureOutsource {
    pub fn new(signature: &Signature, message: u128, play_cards: &Option<Vec<CryptoCard>>) -> Self {
        Self {
            signature: signature.clone(),
            message,
            play_cards: play_cards.clone(),

            signature_var: (0, 0),
            message_var: 0,
            play_card_vars: None,
        }
    }

    pub fn generate_constraints(&mut self, cs: &mut TurboCS<Fr>, pks: &PublicKeyOutsource) {
        let zero = Fr::ZERO;
        let one = Fr::ONE;
        let zero_var = cs.zero_var();

        let g = EdwardsProjective::generator();

        let base_0 = g.mul(self.signature.s);
        let s = Fr::from_be_bytes_mod_order(&self.signature.s.into_bigint().to_bytes_be());
        let s_var = cs.new_variable(s);
        let out_0 = cs.const_base_scalar_mul(g, s_var, 256);

        let m_bytes = [
            6, 12, 137, 206, 92, 38, 52, 5, 55, 10, 8, 182, 208, 48, 43, 11, 171, 62, 237, 184, 57,
            32, 238, 10, 103, 114, 151, 220, 57, 33, 38, 241,
        ];
        let m_field = Fr::from_be_bytes_mod_order(&m_bytes);
        // let m_var = cs.new_variable(m_field); // todo constant
        let m = BigUint::from_bytes_be(&m_bytes);

        let n: BigUint = self.signature.e.into();
        let n_var = cs.new_variable(self.signature.e);
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
        cs.wiring[4].push(n_var);

        cs.finish_new_gate();

        let base_1 = pks.public_keys[0].mul(&remainder_251);
        let out_1 =
            cs.nonconst_base_scalar_mul(pks.cs_vars[0], pks.public_keys[0], remainder_var, 256);
        let out_2 = cs.ecc_add(&out_0, &out_1, &base_0, &base_1);
        let out_2_aff = out_2.get_point().into_affine();

        let mut inputs = vec![Fr::from(self.message)];

        let mut cards = {
            if let Some(x) = &self.play_cards {
                let mut tmp = vec![];
                x.iter().for_each(|y| tmp.extend(y.0.flatten()));
                tmp
            } else {
                vec![]
            }
        };
        cards.extend_from_slice(&[Fr::ZERO].repeat(MAX_PLAYER_HAND_LEN * 4 - cards.len()));

        inputs.extend(cards);

        let mut input_vars = inputs
            .iter()
            .map(|x| cs.new_variable(*x))
            .collect::<Vec<_>>();

        inputs.extend(vec![
            pks.affine_reps[0].x,
            pks.affine_reps[0].y,
            out_2_aff.x,
            out_2_aff.y,
        ]);
        input_vars.extend(vec![
            pks.cs_vars[0].get_x(),
            pks.cs_vars[0].get_y(),
            out_2.get_var().get_x(),
            out_2.get_var().get_y(),
        ]);

        let trace = AnemoiJive254::eval_variable_length_hash_with_trace(&inputs);
        let output = trace.output;
        let output_var = cs.new_variable(output);
        cs.anemoi_variable_length_hash::<AnemoiJive254>(&trace, &input_vars, output_var);

        cs.equal(output_var, n_var);
    }
}

#[cfg(test)]
mod test {
    use crate::public_keys::PublicKeyOutsource;
    use ark_bn254::Fr;
    use poker_core::mock_data::mock_task;
    use zplonk::{anemoi::AnemoiJive254, turboplonk::constraint_system::turbo::TurboCS};

    use super::SignatureOutsource;

    #[test]
    fn test_signature_constraint_system() {
        let task = mock_task();
        let env = &task.players_env[0][0];

        env.verify_sign(&task.players_keys[0]).unwrap();

        let mut cs = TurboCS::<Fr>::new();
        cs.load_anemoi_parameters::<AnemoiJive254>();

        let pk_outsource = PublicKeyOutsource::new(&mut cs, &task.players_keys);

        let message = env.pack();
        let play_cards = if let Some(x) = &env.play_cards {
            Some(x.to_vec())
        } else {
            None
        };

        let mut sign_outsource = SignatureOutsource::new(&env.signature, message, &play_cards);
        sign_outsource.generate_constraints(&mut cs, &pk_outsource);

        let witness = cs.get_and_clear_witness();
        cs.verify_witness(&witness, &[]).unwrap();

        assert_eq!(cs.size, 2849);
    }
}
