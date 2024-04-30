mod poker;
mod utils;

use ark_ec::CurveGroup;
use ark_ed_on_bn254::{EdwardsProjective, Fr};
use poker::{morph_to_card_combination, CryptoCard};
use poker_core::{
    cards::{reveal0, unmask, RevealCard, DECK, ENCODING_CARDS_MAPPING},
    play::{PlayAction, PlayerEnvBuilder},
    schnorr::{KeyPair, PrivateKey, PublicKey},
    RevealProof,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use utils::{
    default_prng, error_to_jsvalue, hex_to_point, hex_to_scalar, point_to_hex, point_to_uncompress,
    scalar_to_hex, uncompress_to_point,
};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Serialize, Deserialize)]
pub struct RevealedCardWithProof {
    /// reveal card
    pub card: (String, String),
    /// hex string
    pub proof: String,
    /// public key
    pub public_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerEnv {
    // The unique identifier for the game room.
    pub room_id: usize,
    // The identifier for the current game round.
    pub round_id: u8,
    // The identifier for the current turn within the round.
    pub turn_id: u8,
    // 0 : Play,
    // 1 : Pass.
    pub action: u8,
    // 0 : Single,            1 : Pair,            2 : ConnectedPairs,
    // 3 : Three,             4 : ThreeWithOne,    5 : ThreeWithPair,
    // 6 : Straight,          7 : FourWithTwo,     8 : Bomb,
    // 9 : AAA.
    pub types: u8,
    pub play_cards: Vec<CryptoCard>,
    pub reveals: Vec<Vec<RevealedCardWithProof>>,
    pub private_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct Keypair {
    pub sk: String,
    pub pk: String,
    pub pkxy: (String, String),
}

/// generate keypair
#[wasm_bindgen]
pub fn generate_key() -> Result<JsValue, JsValue> {
    let mut prng = default_prng();
    let keypair = KeyPair::sample(&mut prng);
    let pkxy = point_to_uncompress(&keypair.get_public_key().get_raw(), true);

    let ret = Keypair {
        sk: scalar_to_hex(&keypair.get_private_key().0, true),
        pk: point_to_hex(&keypair.get_public_key().get_raw(), true),
        pkxy,
    };

    Ok(serde_wasm_bindgen::to_value(&ret)?)
}

/// compute masked to revealed card and the revealed proof
#[wasm_bindgen]
pub fn reveal_card(sk: String, card: JsValue) -> Result<JsValue, JsValue> {
    let card_wasm: CryptoCard = serde_wasm_bindgen::from_value(card)?;
    let crypto_card = card_wasm.deserialize()?;

    let mut prng = default_prng();
    let key_pair = KeyPair::from_private_key(PrivateKey(hex_to_scalar(&sk)?));

    let (reveal_card, reveal_proof) =
        reveal0(&mut prng, &key_pair, &crypto_card.0).map_err(error_to_jsvalue)?;
    let reveal_card_projective: EdwardsProjective = reveal_card.0.into();

    let ret = RevealedCardWithProof {
        card: point_to_uncompress(&reveal_card_projective, true),
        proof: format!("0x{}", hex::encode(&reveal_proof.to_uncompress())),
        public_key: point_to_hex(&key_pair.get_public_key().get_raw(), true),
    };

    Ok(serde_wasm_bindgen::to_value(&ret)?)
}

/// unmask the card use others' reveals
#[wasm_bindgen]
pub fn unmask_card(sk: String, card: JsValue, reveals: JsValue) -> Result<usize, JsValue> {
    let card_wasm: CryptoCard = serde_wasm_bindgen::from_value(card)?;
    let crypto_card = card_wasm.deserialize()?;

    let reveals: Vec<(String, String)> = serde_wasm_bindgen::from_value(reveals)?;
    let mut reveal_cards = vec![];
    for reveal in reveals {
        reveal_cards.push(RevealCard(uncompress_to_point(&reveal.0, &reveal.1)?));
    }

    let mut prng = default_prng();
    let keypair = KeyPair::from_private_key(PrivateKey(hex_to_scalar(&sk)?));

    let (reveal_card, _proof) =
        reveal0(&mut prng, &keypair, &crypto_card.0).map_err(error_to_jsvalue)?;
    reveal_cards.push(reveal_card);

    let unmasked_card = unmask(&crypto_card.0, &reveal_cards);
    let classic_card = ENCODING_CARDS_MAPPING
        .get(&unmasked_card.0)
        .ok_or(error_to_jsvalue("failed to map to classic card"))?;
    DECK.iter()
        .position(|x| x == classic_card)
        .ok_or(error_to_jsvalue(
            "Failed to obtain the index of the classic card",
        ))
}

// create player env.
#[wasm_bindgen]
pub fn create_play_env(player_env: JsValue) -> Result<String, JsValue> {
    let player_env: PlayerEnv = serde_wasm_bindgen::from_value(player_env)?;

    let mut prng = default_prng();

    let sk: Fr = hex_to_scalar(&player_env.private_key)?;
    let key_pair = KeyPair::from_private_key(PrivateKey(sk));

    let env_with_sign = match player_env.action {
        0 => {
            let play_cards = morph_to_card_combination(player_env.types, &player_env.play_cards)?;

            let mut reveals = vec![];

            for reveal in player_env.reveals.iter() {
                let mut tmp = vec![];

                for proof in reveal.iter() {
                    let reveal_card = uncompress_to_point(&proof.card.0, &proof.card.1)?;

                    let reveal_proof = RevealProof::from_uncompress(
                        &hex::decode(proof.proof.trim_start_matches("0x"))
                            .map_err(error_to_jsvalue)?,
                    )
                    .map_err(error_to_jsvalue)?;

                    let pk: EdwardsProjective = hex_to_point(&proof.public_key)?;

                    tmp.push((
                        RevealCard(reveal_card),
                        reveal_proof,
                        PublicKey(pk.into_affine()),
                    ))
                }

                reveals.push(tmp);
            }

            PlayerEnvBuilder::new()
                .room_id(player_env.room_id)
                .round_id(player_env.round_id)
                .turn_id(player_env.turn_id)
                .action(PlayAction::PLAY)
                .play_cards(Some(play_cards))
                .reveals(&reveals)
                .build_and_sign(&key_pair, &mut prng)
                .map_err(error_to_jsvalue)?
        }

        1 => PlayerEnvBuilder::new()
            .room_id(player_env.room_id)
            .round_id(player_env.round_id)
            .turn_id(player_env.turn_id)
            .action(PlayAction::PAAS)
            .build_and_sign(&key_pair, &mut prng)
            .map_err(error_to_jsvalue)?,

        _ => return Err(error_to_jsvalue("Incorrect play action")),
    };

    let res = serde_json::to_string(&env_with_sign).map_err(error_to_jsvalue)?;

    Ok(res)
}
