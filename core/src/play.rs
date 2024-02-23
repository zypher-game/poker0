use crate::{
    cards::EncodingCard,
    combination::{CryptoCardCombination, EncodingCardCombination},
    errors::{PokerError, Result},
    schnorr::{KeyPair, PublicKey, Signature},
};
use ark_bn254::Fr;
use ark_ec::AdditiveGroup;
use rand_chacha::rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use zshuffle::{
    reveal::{unmask, verify_reveal0},
    RevealProof,
};

pub const MAX_PLAYER_HAND_LEN: usize = 18;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum PlayAction {
    PAAS,
    PLAY,
}

impl From<PlayAction> for u8 {
    fn from(val: PlayAction) -> Self {
        match val {
            PlayAction::PAAS => 0,
            PlayAction::PLAY => 1,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlayerEnv {
    // The unique identifier for the game room.
    pub room_id: usize,
    // The identifier for the current game round.
    pub round_id: u8,
    // The identifier for the current turn within the round.
    pub turn_id: u8,
    pub action: PlayAction,
    pub play_cards: Option<CryptoCardCombination>,
    //  pub reveal: Vec<(EncodingCard, RevealProof, PublicKey)>,
    pub reveals: Vec<Vec<(EncodingCard, RevealProof, PublicKey)>>,
    // Currently using schnorr signatures, with plans to transition to aggregated signatures in the future.
    pub signature: Signature,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerEnv0 {
    pub room_id: usize,
    pub round_id: u8,
    pub turn_id: u8,
    pub action: PlayAction,
    pub play_crypto_cards: Option<CryptoCardCombination>,
    pub play_unmasked_cards: Option<EncodingCardCombination>,
}

impl Default for PlayerEnv {
    fn default() -> Self {
        Self {
            room_id: 0,
            turn_id: 0,
            round_id: 0,
            action: PlayAction::PAAS,
            play_cards: None,
            reveals: vec![],
            signature: Signature::default(),
        }
    }
}

/// A builder used to construct an [PlayerEnv].
#[derive(Default)]
pub struct PlayerEnvBuilder {
    pub(crate) inner: PlayerEnv,
}

impl PlayerEnv {
    /// Construct a [PlayerEnvBuilder].
    pub fn builder() -> PlayerEnvBuilder {
        PlayerEnvBuilder::default()
    }

    pub fn pack(&self) -> u128 {
        let action: u8 = self.action.into();
        (action as u128)
            + ((self.round_id as u128) << 8)
            + ((self.turn_id as u128) << 16)
            + ((self.room_id as u128) << 24)
    }

    pub fn verify_sign(&self, pk: &PublicKey) -> Result<()> {
        let pack = self.pack();
        let mut msg = vec![Fr::from(pack)];

        let mut cards = {
            if self.action != PlayAction::PAAS {
                self.play_cards.clone().unwrap().flatten()
            } else {
                vec![]
            }
        };
        cards.extend_from_slice(&[Fr::ZERO].repeat(MAX_PLAYER_HAND_LEN * 4 - cards.len()));

        msg.extend(cards);

        pk.verify(&self.signature, &msg)
    }

    pub fn verify_sign_with_params(
        &self,
        pk: &PublicKey,
        room_id: usize,
        round_id: u8,
        turn_id: u8,
    ) -> Result<()> {
        assert_eq!(self.room_id, room_id);
        assert_eq!(self.round_id, round_id);
        assert_eq!(self.turn_id, turn_id);

        let pack = self.pack();
        let mut msg = vec![Fr::from(pack)];

        let mut cards = {
            if self.action != PlayAction::PAAS {
                self.play_cards.clone().unwrap().flatten()
            } else {
                vec![]
            }
        };
        cards.extend_from_slice(&[Fr::ZERO].repeat(MAX_PLAYER_HAND_LEN * 4 - cards.len()));

        msg.extend(cards);

        pk.verify(&self.signature, &msg)
    }

    pub fn verify_and_get_reveals(&self) -> Result<Vec<EncodingCard>> {
        let cards = self
            .play_cards
            .clone()
            .ok_or(PokerError::NoCardError)?
            .to_vec();
        // let vec = cards.to_vec();
        assert_eq!(cards.len(), self.reveals.len());

        let mut unmasked_cards = Vec::new();

        for (reveals, card) in self.reveals.iter().zip(cards.iter()) {
            let mut reveal_cards = Vec::new();
            for reveal in reveals.iter() {
                verify_reveal0(&reveal.2.get_raw(), &card.0, &reveal.0 .0, &reveal.1)
                    .map_err(|_| PokerError::VerifyReVealError)?;
                reveal_cards.push(reveal.0 .0);
            }

            let unmasked_card =
                unmask(&card.0, &reveal_cards).map_err(|_| PokerError::UnmaskCardError)?;
            unmasked_cards.push(EncodingCard(unmasked_card));
        }

        Ok(unmasked_cards)
    }

    pub fn convert_to_env0(&self) -> PlayerEnv0 {
        let unmasked_cards = if self.action == PlayAction::PLAY {
            let unmasked_cards = self.verify_and_get_reveals().unwrap();
            let unmasked_cards = self
                .play_cards
                .as_ref()
                .and_then(|x| Some(x.morph_to_encoding(&unmasked_cards)))
                .unwrap();

            Some(unmasked_cards)
        } else {
            None
        };

        PlayerEnv0 {
            room_id: self.room_id,
            round_id: self.round_id,
            turn_id: self.turn_id,
            action: self.action,
            play_crypto_cards: self.play_cards.clone(),
            play_unmasked_cards: unmasked_cards,
        }
    }
}

impl PlayerEnvBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn room_id(mut self, room_id: usize) -> Self {
        self.inner.room_id = room_id;
        self
    }

    pub fn turn_id(mut self, turn_id: u8) -> Self {
        self.inner.turn_id = turn_id;
        self
    }

    pub fn round_id(mut self, round_id: u8) -> Self {
        self.inner.round_id = round_id;
        self
    }

    pub fn action(mut self, action: PlayAction) -> Self {
        self.inner.action = action;
        self
    }

    pub fn play_cards(mut self, play_cards: Option<CryptoCardCombination>) -> Self {
        self.inner.play_cards = play_cards;
        self
    }

    pub fn reveals(mut self, reveals: &[Vec<(EncodingCard, RevealProof, PublicKey)>]) -> Self {
        self.inner.reveals = reveals.to_vec();
        self
    }

    pub fn validate_rules(&self) -> Result<()> {
        match self.inner.action {
            PlayAction::PAAS => {
                if !self.inner.reveals.is_empty() || self.inner.play_cards.is_some() {
                    Err(PokerError::BuildPlayEnvParamsError)
                } else {
                    Ok(())
                }
            }
            PlayAction::PLAY => {
                if let Some(c) = &self.inner.play_cards {
                    // todo check  self.inner.others_reveal.len = participant
                    if self.inner.reveals.len() != c.len() {
                        Err(PokerError::BuildPlayEnvParamsError)
                    } else {
                        Ok(())
                    }
                } else {
                    Err(PokerError::BuildPlayEnvParamsError)
                }
            }
        }
    }

    pub fn build_and_sign<R: CryptoRng + RngCore>(
        mut self,
        key: &KeyPair,
        prng: &mut R,
    ) -> Result<PlayerEnv> {
        self.validate_rules()?;

        let pack = self.inner.pack();
        let mut msg = vec![Fr::from(pack)];

        let mut cards = {
            if self.inner.action != PlayAction::PAAS {
                self.inner.play_cards.clone().unwrap().flatten()
            } else {
                vec![]
            }
        };
        cards.extend_from_slice(&[Fr::ZERO].repeat(MAX_PLAYER_HAND_LEN * 4 - cards.len()));

        msg.extend(cards);

        let s = key.sign(&msg, prng)?;

        self.inner.signature = s;

        Ok(self.inner)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};

    #[test]
    fn test_player() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);
        let key_pair = KeyPair::sample(&mut prng);
        let player = PlayerEnvBuilder::new()
            .room_id(1)
            .round_id(1)
            .turn_id(1)
            .action(PlayAction::PAAS)
            .build_and_sign(&key_pair, &mut prng)
            .unwrap();

        assert!(player.verify_sign(&key_pair.get_public_key()).is_ok());
    }

    #[test]
    fn t() {
        let x = (1 as u128)
            + (((u8::MAX - 4) as u128) << 8)
            + (((u8::MAX - 2) as u128) << 16)
            + ((u64::MAX as u128) << 24);

        println!("{:?}", x.to_be_bytes())
    }
}
