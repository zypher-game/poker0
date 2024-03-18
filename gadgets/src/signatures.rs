use std::ops::Mul;

use ark_bn254::Fr;
use ark_ec::{AdditiveGroup, CurveGroup, PrimeGroup};
use ark_ed_on_bn254::EdwardsProjective;
use ark_ff::{BigInteger, Field, PrimeField};
use num_bigint::BigUint;
use num_integer::Integer;
use plonk::{
    anemoi::{AnemoiJive, AnemoiJive254},
    turboplonk::constraint_system::{ecc::PointVar, turbo::TurboCS, VarIndex},
};
use poker_core::{
    cards::CryptoCard,
    play::MAX_PLAYER_HAND_LEN,
    schnorr::{PublicKey, Signature},
};

use crate::get_divisor;

pub struct SignatureOutsource {
    pub signature: Signature,
    pub pack_messages: u128,
    pub play_cards: Option<Vec<CryptoCard>>,

    pub signature_var: (VarIndex, VarIndex),
    pub pack_messages_var: VarIndex,
    pub play_card_vars: Option<Vec<(PointVar, PointVar)>>,
}

impl SignatureOutsource {
    pub fn new(
        signature: &Signature,
        pack_messages: u128,
        play_cards: &Option<Vec<CryptoCard>>,
    ) -> Self {
        Self {
            signature: signature.clone(),
            pack_messages,
            play_cards: play_cards.clone(),

            signature_var: (0, 0),
            pack_messages_var: 0,
            play_card_vars: None,
        }
    }

    pub fn generate_constraints(
        &mut self,
        cs: &mut TurboCS<Fr>,
        pk: &PublicKey,
        pk_var: &PointVar,
    ) {
        let zero = Fr::ZERO;
        let one = Fr::ONE;
        let zero_var = cs.zero_var();

        let g = EdwardsProjective::generator();

        let base_0 = g.mul(self.signature.s);
        let s = Fr::from_be_bytes_mod_order(&self.signature.s.into_bigint().to_bytes_be());
        let s_var = cs.new_variable(s);
        let out_0 = cs.const_base_scalar_mul(g, s_var, 256);

        let m = get_divisor();

        let n: BigUint = self.signature.e.into();
        let n_var = cs.new_variable(self.signature.e);
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
        cs.wiring[4].push(n_var);

        cs.finish_new_gate();

        let base_1 = pk.0.mul(&remainder_251);
        let out_1 = cs.nonconst_base_scalar_mul(*pk_var, pk.0.into(), remainder_var, 256);
        let out_2 = cs.ecc_add(&out_0, &out_1, &base_0, &base_1);
        let out_2_aff = out_2.get_point().into_affine();

        let mut inputs = vec![Fr::from(self.pack_messages)];

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

        inputs.extend(vec![pk.0.x, pk.0.y, out_2_aff.x, out_2_aff.y]);
        input_vars.extend(vec![
            pk_var.get_x(),
            pk_var.get_y(),
            out_2.get_var().get_x(),
            out_2.get_var().get_y(),
        ]);

        let trace = AnemoiJive254::eval_variable_length_hash_with_trace(&inputs);
        let output = trace.output;
        let output_var = cs.new_variable(output);
        cs.anemoi_variable_length_hash::<AnemoiJive254>(&trace, &input_vars, output_var);

        cs.equal(output_var, n_var);
    }

    pub fn prepare_pi_variables(&self, cs: &mut TurboCS<Fr>) {
      cs.prepare_pi_variable(self.pack_messages_var)
    }
}

#[cfg(test)]
mod test {
    use super::SignatureOutsource;
    use crate::public_keys::PublicKeyOutsource;
    use ark_bn254::Fr;
    use plonk::{anemoi::AnemoiJive254, turboplonk::constraint_system::turbo::TurboCS};
    use poker_core::mock_data::task::mock_task;

    #[test]
    fn test_signature_constraint_system() {
        let task = mock_task();
        let env = &task.players_env[0][0];
        env.verify_sign(&task.players_keys[0]).unwrap();

        let mut cs = TurboCS::<Fr>::new();
        cs.load_anemoi_parameters::<AnemoiJive254>();

        let pk_outsource = PublicKeyOutsource::new(&mut cs, &task.players_keys);

        let message = env.pack();
        let play_cards = if let Some(x) = &env.play_crypto_cards {
            Some(x.to_vec())
        } else {
            None
        };

        let mut sign_outsource = SignatureOutsource::new(&env.signature, message, &play_cards);
        sign_outsource.generate_constraints(
            &mut cs,
            &pk_outsource.public_keys[0],
            &pk_outsource.cs_vars[0],
        );

        let witness = cs.get_and_clear_witness();
        cs.verify_witness(&witness, &[]).unwrap();

        assert_eq!(cs.size, 2847);
    }
}