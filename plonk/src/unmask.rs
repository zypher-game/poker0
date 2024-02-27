use ark_bn254::Fr;
use ark_ed_on_bn254::EdwardsAffine;
use poker_core::cards::{CryptoCard, RevealCard};
use std::ops::Neg;
use zplonk::turboplonk::constraint_system::{ecc::PointVar, turbo::TurboCS};

pub struct UnmaskOutsource {
    pub crypto_card: CryptoCard,
    pub reveal_cards: Vec<RevealCard>,
    pub unmasked_card: EdwardsAffine,

    pub crypto_card_var: (PointVar, PointVar),
    pub reveal_card_vars: Vec<PointVar>,
    pub unmasked_card_var: PointVar,
}

impl UnmaskOutsource {
    pub fn new(
        crypto_card: &CryptoCard,
        crypto_card_var: (PointVar, PointVar),
        reveal_cards: &[RevealCard],
        reveal_card_vars: &[PointVar],
        unmasked_card: &EdwardsAffine,
    ) -> Self {
        Self {
            crypto_card: *crypto_card,
            reveal_cards: reveal_cards.to_vec(),
            unmasked_card: *unmasked_card,
            crypto_card_var,
            reveal_card_vars: reveal_card_vars.to_vec(),
            unmasked_card_var: PointVar::default(),
        }
    }

    pub fn generate_constraints(&mut self, cs: &mut TurboCS<Fr>) {
        self.unmasked_card_var = cs.new_point_variable(self.unmasked_card.into());

        let mut sum = self.reveal_cards[0].0.into();
        let mut sum_var = self.reveal_card_vars[0];

        for (reveal, var) in self
            .reveal_cards
            .iter()
            .zip(self.reveal_card_vars.iter())
            .skip(1)
        {
            let res = cs.ecc_add(&sum_var, var, &sum, &reveal.0.into());
            sum = *res.get_point();
            sum_var = res.into_point_var();
        }

        let neg_var = PointVar::new(cs.sub(cs.zero_var(), sum_var.get_x()), sum_var.get_y());
        let unmask_card_var = cs.ecc_add(
            &neg_var,
            &self.crypto_card_var.1,
            &sum.neg(),
            &self.crypto_card.0.e2.into(),
        );

        cs.equal(
            unmask_card_var.get_var().get_x(),
            self.unmasked_card_var.get_x(),
        );

        cs.equal(
            unmask_card_var.get_var().get_y(),
            self.unmasked_card_var.get_y(),
        );
    }

    pub fn prepare_pi_variables(&self, cs: &mut TurboCS<Fr>) {
        cs.prepare_pi_point_variable(self.unmasked_card_var);
    }
}

#[cfg(test)]
mod test {
    use crate::{
        public_keys::PublicKeyOutsource, reveals::RevealOutsource, unmask::UnmaskOutsource,
    };
    use ark_bn254::Fr;
    use ark_ec::CurveGroup;
    use poker_core::mock_data::task::mock_task;
    use zplonk::{anemoi::AnemoiJive254, turboplonk::constraint_system::turbo::TurboCS};

    #[test]
    fn test_unmask_constraint_system() {
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

        let size = cs.size;
        let reveal_cards_projective = reveal_cards.iter().map(|x| x.0.into()).collect::<Vec<_>>();
        let unmasked_card =
            zshuffle::reveal::unmask(&card.0.to_ciphertext(), &reveal_cards_projective).unwrap();
        let mut unmask_outsource = UnmaskOutsource::new(
            &card,
            reveal_outsource.crypto_card_var,
            &reveal_cards,
            &reveal_outsource.reveal_card_vars,
            &unmasked_card.into_affine(),
        );
        unmask_outsource.generate_constraints(&mut cs);
        assert_eq!(9, cs.size - size);

        let witness = cs.get_and_clear_witness();
        cs.verify_witness(&witness, &[]).unwrap();
    }
}
