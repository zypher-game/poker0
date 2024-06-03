use crate::{
    cards::{unmask, verify_reveal0, CryptoCard, EncodingCard, RevealCard},
    combination::{
        ClassicCardCombination, CryptoCardCombination, EncodingCardCombination, IndexCombination,
    },
    errors::{PokerError, Result},
    schnorr::{KeyPair, PublicKey, Signature},
    RevealProof,
};
use ark_serialize::CanonicalSerialize;
use rand_chacha::rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum PlayAction {
    PASS,
    PLAY,
    OFFLINE,
}

impl From<PlayAction> for u8 {
    fn from(val: PlayAction) -> Self {
        match val {
            PlayAction::PASS => 0,
            PlayAction::PLAY => 1,
            PlayAction::OFFLINE => 2,
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
    pub play_crypto_cards: Option<CryptoCardCombination>,
    pub play_classic_cards: Option<ClassicCardCombination>,
    // Note! The order of revealing is based on the order of the players.
    pub reveals: Vec<Vec<(RevealCard, RevealProof, PublicKey)>>,
    // Currently using schnorr signatures, with plans to transition to aggregated signatures in the future.
    pub signature: Signature,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerEnv0 {
    pub room_id: usize,
    pub round_id: u8,
    pub turn_id: u8,
    pub action: PlayAction,
    pub play_crypto_cards: Option<IndexCombination>,
    pub play_unmasked_cards: Option<EncodingCardCombination>,
}

impl Default for PlayerEnv {
    fn default() -> Self {
        Self {
            room_id: 0,
            turn_id: 0,
            round_id: 0,
            action: PlayAction::PASS,
            play_classic_cards: None,
            play_crypto_cards: None,
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

    pub fn pack(&self) -> Vec<u8> {
        let action: u8 = self.action.into();
        let mut msg = vec![action, self.round_id, self.turn_id];
        msg.extend(self.room_id.to_be_bytes());

        if self.action == PlayAction::PLAY {
            for crypto_card in self.play_crypto_cards.clone().unwrap().to_vec().iter() {
                let mut e1_bytes = vec![];
                crypto_card
                    .0
                    .e1
                    .serialize_uncompressed(&mut e1_bytes)
                    .unwrap();
                msg.extend(e1_bytes);

                let mut e2_bytes = vec![];
                crypto_card
                    .0
                    .e2
                    .serialize_uncompressed(&mut e2_bytes)
                    .unwrap();
                msg.extend(e2_bytes);
            }
        }

        msg
    }

    // Synchronize the order of revealing with the order in which players play their cards.
    pub fn sync_reveal_order(&mut self, pks: &[PublicKey]) {
        if self.action == PlayAction::PLAY {
            self.reveals.iter_mut().for_each(|x| {
                x.sort_by_key(|y| pks.iter().rev().map(|pk| *pk == y.2).collect::<Vec<_>>())
            })
        }
    }

    pub fn verify_sign(&self, pk: &PublicKey) -> Result<()> {
        let msg = self.pack();
        pk.verify(&self.signature, &msg)
    }

    pub fn verify_sign_with_external_params(
        &self,
        pk: &PublicKey,
        room_id: usize,
        round_id: u8,
        turn_id: u8,
    ) -> Result<()> {
        assert_eq!(self.room_id, room_id);
        assert_eq!(self.round_id, round_id);
        assert_eq!(self.turn_id, turn_id);

        self.verify_sign(pk)
    }

    pub fn unmask_cards(&self) -> Result<Vec<EncodingCard>> {
        let cards = self
            .play_crypto_cards
            .clone()
            .ok_or(PokerError::NoCardError)?
            .to_vec();
        assert_eq!(cards.len(), self.reveals.len());

        let mut unmasked_cards = Vec::with_capacity(cards.len());

        for (reveals, card) in self.reveals.iter().zip(cards.iter()) {
            let mut reveal_cards = Vec::with_capacity(reveals.len());
            for reveal in reveals.iter() {
                verify_reveal0(&reveal.2, &card.0, &reveal.0, &reveal.1)
                    .map_err(|_| PokerError::VerifyReVealError)?;
                reveal_cards.push(reveal.0);
            }

            let unmasked_card = unmask(&card.0, &reveal_cards);
            unmasked_cards.push(unmasked_card);
        }

        Ok(unmasked_cards)
    }

    pub fn convert0(&self, hand: &[CryptoCard]) -> PlayerEnv0 {
        let (unmasked_cards, crypto_cards) = if self.action == PlayAction::PLAY {
            let unmasked_cards = self.unmask_cards().unwrap();
            let (unmasked_cards, crypto_cards) = self
                .play_crypto_cards
                .as_ref()
                .and_then(|x| {
                    Some((
                        Some(x.morph_to_encoding(&unmasked_cards)),
                        Some(x.morph_to_index(hand)),
                    ))
                })
                .unwrap();

            (unmasked_cards, crypto_cards)
        } else {
            (None, None)
        };

        PlayerEnv0 {
            room_id: self.room_id,
            round_id: self.round_id,
            turn_id: self.turn_id,
            action: self.action,
            play_crypto_cards: crypto_cards,
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

    pub fn play_cards(mut self, play_crypto_cards: Option<CryptoCardCombination>) -> Self {
        self.inner.play_crypto_cards = play_crypto_cards;
        self
    }

    pub fn reveals(mut self, reveals: &[Vec<(RevealCard, RevealProof, PublicKey)>]) -> Self {
        self.inner.reveals = reveals.to_vec();
        self
    }

    pub fn sanity_check(&self) -> Result<()> {
        match self.inner.action {
            PlayAction::PLAY => {
                if let Some(c) = &self.inner.play_crypto_cards {
                    if self.inner.reveals.len() != c.len() {
                        return Err(PokerError::BuildPlayEnvParamsError);
                    }

                    Ok(())
                } else {
                    Err(PokerError::BuildPlayEnvParamsError)
                }
            }
            _ => Ok(()),
        }
    }

    pub fn build_and_sign<R: CryptoRng + RngCore>(
        mut self,
        key: &KeyPair,
        prng: &mut R,
    ) -> Result<PlayerEnv> {
        self.sanity_check()?;

        if self.inner.action == PlayAction::PLAY {
            let reveals = self.inner.unmask_cards().unwrap();
            let encode_card = self
                .inner
                .play_crypto_cards
                .clone()
                .unwrap()
                .morph_to_encoding(&reveals);
            let classic = encode_card.morph_to_classic().unwrap();
            self.inner.play_classic_cards = Some(classic);
        }

        let msg = self.inner.pack();
        self.inner.signature = key.sign(&msg, prng)?;

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
            .action(PlayAction::PASS)
            .build_and_sign(&key_pair, &mut prng)
            .unwrap();

        assert!(player.verify_sign(&key_pair.get_public_key()).is_ok());
    }
}
