use poker_core::{
    cards::CryptoCard as ZCryptoCard, combination::CryptoCardCombination, CiphertextAffineRepr,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::utils::uncompress_to_point;

/// e2.0, e2.1, e1.0, e1.1
#[derive(Serialize, Deserialize, Clone)]
pub struct CryptoCard(pub String, pub String, pub String, pub String);

impl CryptoCard {
    fn deserialize(&self) -> Result<ZCryptoCard, JsValue> {
        let e2 = uncompress_to_point(&self.0, &self.1)?;
        let e1 = uncompress_to_point(&self.2, &self.3)?;
        Ok(ZCryptoCard(CiphertextAffineRepr::new(e1, e2)))
    }
}

pub fn morph_to_card_combination(
    types: u8,
    play_cards: &[CryptoCard],
) -> Result<CryptoCardCombination, JsValue> {
    let play_cards = play_cards
        .iter()
        .map(|x| x.deserialize().unwrap())
        .collect::<Vec<_>>();

    match types {
        0 => {
            assert_eq!(play_cards.len(), 1);
            let card = CryptoCardCombination::Single(play_cards[0]);
            // todo check rules

            Ok(card)
        }
        1 => {
            assert_eq!(play_cards.len(), 2);
            let card = CryptoCardCombination::Pair(play_cards[0], play_cards[1]);
            Ok(card)
        }
        2 => {
            assert_eq!(play_cards.len(), 3);
            let card =
                CryptoCardCombination::ThreeOfAKind(play_cards[0], play_cards[1], play_cards[2]);
            Ok(card)
        }
        3 => {
            assert_eq!(play_cards.len(), 4);
            let card = CryptoCardCombination::ThreeWithOne(
                play_cards[0],
                play_cards[1],
                play_cards[2],
                play_cards[3],
            );
            Ok(card)
        }
        4 => {
            assert_eq!(play_cards.len(), 5);
            let card = CryptoCardCombination::ThreeWithPair(
                play_cards[0],
                play_cards[1],
                play_cards[2],
                play_cards[3],
                play_cards[4],
            );
            Ok(card)
        }
        5 => {
            assert!(play_cards.len() >= 5);
            let card = CryptoCardCombination::Straight(play_cards);
            Ok(card)
        }
        6 => {
            assert!(play_cards.len() >= 6);
            let tmp = play_cards
                .chunks(2)
                .into_iter()
                .map(|x| (x[0], x[1]))
                .collect::<Vec<_>>();
            let card = CryptoCardCombination::DoubleStraight(tmp);
            Ok(card)
        }
        7 => {
            assert!(play_cards.len() >= 6);
            let tmp = play_cards
                .chunks(3)
                .into_iter()
                .map(|x| (x[0], x[1], x[2]))
                .collect::<Vec<_>>();
            let card = CryptoCardCombination::TripleStraight(tmp);
            Ok(card)
        }
        8 => {
            assert!(play_cards.len() >= 8);
            let tmp = play_cards
                .chunks(4)
                .into_iter()
                .map(|x| (x[0], x[1], x[2], x[3]))
                .collect::<Vec<_>>();
            let card = CryptoCardCombination::TripleStraightWithOne(tmp);
            Ok(card)
        }
        9 => {
            assert!(play_cards.len() >= 10);
            let tmp = play_cards
                .chunks(5)
                .into_iter()
                .map(|x| (x[0], x[1], x[2], x[3], x[4]))
                .collect::<Vec<_>>();
            let card = CryptoCardCombination::TripleStraightWithPair(tmp);
            Ok(card)
        }
        10 => {
            assert_eq!(play_cards.len(), 6);
            let card = CryptoCardCombination::FourWithTwoSingle(
                play_cards[0],
                play_cards[1],
                play_cards[2],
                play_cards[3],
                play_cards[4],
                play_cards[5],
            );
            Ok(card)
        }
        11 => {
            assert_eq!(play_cards.len(), 8);
            let card = CryptoCardCombination::FourWithTwoPairs(
                play_cards[0],
                play_cards[1],
                play_cards[2],
                play_cards[3],
                play_cards[4],
                play_cards[5],
                play_cards[6],
                play_cards[7],
            );
            Ok(card)
        }
        12 => {
            assert_eq!(play_cards.len(), 4);
            let card = CryptoCardCombination::Bomb(
                play_cards[0],
                play_cards[1],
                play_cards[2],
                play_cards[3],
            );
            Ok(card)
        }
        // todo Bomb
        _ => Err(JsValue::from_str("Incorrect morph-card combination")),
    }
}
