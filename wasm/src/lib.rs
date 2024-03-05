mod poker;
mod utils;

use ark_ec::CurveGroup;
use ark_ed_on_bn254::{EdwardsProjective, Fr};
use poker::{morph_to_card_combination, CryptoCard};
use poker_core::{
    cards::RevealCard,
    play::{PlayAction, PlayerEnvBuilder},
    schnorr::{KeyPair, PrivateKey, PublicKey},
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use zshuffle::RevealProof;

use utils::{default_prng, error_to_jsvalue, hex_to_point, hex_to_scalar, uncompress_to_point};

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
    pub action: u8,
    pub types: u8,
    pub play_cards: Vec<CryptoCard>,
    pub reveals: Vec<Vec<RevealedCardWithProof>>,
    pub private_key: String,
}

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
                .action(PlayAction::PAAS)
                .play_cards(Some(play_cards))
                .reveals(&reveals)
                .build_and_sign(&key_pair, &mut prng)
                .map_err(error_to_jsvalue)?
        }

        1 => PlayerEnvBuilder::new()
            .room_id(player_env.room_id)
            .round_id(player_env.round_id)
            .turn_id(player_env.turn_id)
            .action(PlayAction::PLAY)
            .build_and_sign(&key_pair, &mut prng)
            .map_err(error_to_jsvalue)?,
            
        _ => return Err(error_to_jsvalue("Incorrect play action")),
    };

    let res = serde_json::to_string(&env_with_sign).map_err(error_to_jsvalue)?;
    
    Ok(res)
}
