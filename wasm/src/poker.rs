use poker_core::{
    cards::CryptoCard as ZCryptoCard, combination::CryptoCardCombination, CiphertextAffineRepr,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::utils::uncompress_to_point;

///  e1.0, e1.1, e2.0, e2.1,
#[derive(Serialize, Deserialize, Clone)]
pub struct CryptoCard(pub String, pub String, pub String, pub String);

impl CryptoCard {
    pub fn deserialize(&self) -> Result<ZCryptoCard, JsValue> {
        let e1 = uncompress_to_point(&self.0, &self.1)?;
        let e2 = uncompress_to_point(&self.2, &self.3)?;
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
            Ok(CryptoCardCombination::Single(play_cards[0]))
        }
        1 => {
            assert_eq!(play_cards.len(), 2);
            Ok(CryptoCardCombination::Pair(play_cards[0], play_cards[1]))
        }
        2 => {
            assert!(play_cards.len() >= 6);
            let tmp = play_cards
                .chunks_exact(2)
                .into_iter()
                .map(|x| (x[0], x[1]))
                .collect::<Vec<_>>();
            Ok(CryptoCardCombination::ConnectedPairs(tmp))
        }

        3 => {
            assert_eq!(play_cards.len(), 3);
            Ok(CryptoCardCombination::Three(
                play_cards[0],
                play_cards[1],
                play_cards[2],
            ))
        }
        4 => {
            assert_eq!(play_cards.len(), 4);
            Ok(CryptoCardCombination::ThreeWithOne(
                play_cards[0],
                play_cards[1],
                play_cards[2],
                play_cards[3],
            ))
        }
        5 => {
            assert_eq!(play_cards.len(), 5);
            Ok(CryptoCardCombination::ThreeWithPair(
                play_cards[0],
                play_cards[1],
                play_cards[2],
                play_cards[3],
                play_cards[4],
            ))
        }
        6 => {
            assert!(play_cards.len() >= 5);
            Ok(CryptoCardCombination::Straight(play_cards))
        }
        7 => {
            assert_eq!(play_cards.len(), 6);
            Ok(CryptoCardCombination::FourWithTwo(
                play_cards[0],
                play_cards[1],
                play_cards[2],
                play_cards[3],
                play_cards[4],
                play_cards[5],
            ))
        }
        8 => {
            assert_eq!(play_cards.len(), 4);
            Ok(CryptoCardCombination::Bomb(
                play_cards[0],
                play_cards[1],
                play_cards[2],
                play_cards[3],
            ))
        }
        9 => {
            assert_eq!(play_cards.len(), 3);
            Ok(CryptoCardCombination::AAA(
                play_cards[0],
                play_cards[1],
                play_cards[2],
            ))
        }
        _ => Err(JsValue::from_str("Incorrect card combination")),
    }
}
