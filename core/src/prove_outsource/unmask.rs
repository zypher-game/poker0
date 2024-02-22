use std::ops::Neg;

use ark_bn254::Fr;
use ark_ed_on_bn254::EdwardsProjective;
use zplonk::turboplonk::constraint_system::{ecc::PointVar, turbo::TurboCS};
use zshuffle::MaskedCard;

pub struct UnmaskOutsource {
    pub masked_card: MaskedCard,
    pub reveal_cards: Vec<EdwardsProjective>,
    pub unmasked_card: EdwardsProjective,

    pub masked_card_var: (PointVar, PointVar),
    pub reveal_card_vars: Vec<PointVar>,
    pub unmasked_card_var: PointVar,
}

impl UnmaskOutsource {
    pub fn new(
        masked_card: &MaskedCard,
        masked_card_var: &(PointVar, PointVar),
        reveal_cards: &[EdwardsProjective],
        reveal_card_vars: &[PointVar],
        unmasked_card: &EdwardsProjective,
    ) -> Self {
        Self {
            masked_card: *masked_card,
            reveal_cards: reveal_cards.to_vec(),
            unmasked_card: *unmasked_card,
            masked_card_var: *masked_card_var,
            reveal_card_vars: reveal_card_vars.to_vec(),
            unmasked_card_var: PointVar::default(),
        }
    }

    pub fn generate_constraints(&mut self, cs: &mut TurboCS<Fr>) {
        self.unmasked_card_var = cs.new_point_variable(self.unmasked_card);

        let mut sum = self.reveal_cards[0];
        let mut sum_var = self.reveal_card_vars[0];

        for (card, var) in self
            .reveal_cards
            .iter()
            .zip(self.reveal_card_vars.iter())
            .skip(1)
        {
            let res = cs.ecc_add(&sum_var, var, &sum, card);
            sum = *res.get_point();
            sum_var = res.into_point_var();
        }

        let neg = sum.neg();
        let neg_x_var = cs.sub(cs.zero_var(), sum_var.get_x());
        let neg_var = PointVar::new(neg_x_var, sum_var.get_y());

        let unmask_card_var = cs.ecc_add(
            &neg_var,
            &self.masked_card_var.1,
            &neg,
            &self.masked_card.e2,
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
}

#[cfg(test)]
mod test {
    use crate::prove_outsource::{
        public_keys::PublicKeyOutsource, reveals::RevealOutsource, unmask::UnmaskOutsource,
    };
    use crate::task::mock_task;
    use ark_bn254::Fr;
    use zplonk::{anemoi::AnemoiJive254, turboplonk::constraint_system::turbo::TurboCS};

    #[test]
    fn test_unmask_constraint_system() {
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

        let size = cs.size;
        let unmasked_card = zshuffle::reveal::unmask(&card.0, &reveal_cards).unwrap();
        let mut unmask_outsource = UnmaskOutsource::new(
            &card.0,
            &reveal_outsource.masked_card_var,
            &reveal_cards,
            &reveal_outsource.reveal_card_vars,
            &unmasked_card,
        );
        unmask_outsource.generate_constraints(&mut cs);
        assert_eq!(9, cs.size - size);

        let witness = cs.get_and_clear_witness();
        cs.verify_witness(&witness, &[]).unwrap();
    }
}
