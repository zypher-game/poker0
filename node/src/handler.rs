use ark_ed_on_bn254::{EdwardsAffine, EdwardsProjective, Fr};
use ark_ff::One;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use once_cell::sync::Lazy;
use poker_bonsai::{snark::stark_to_snark, stark::prove_bonsai};
use poker_core::{
    cards::{ClassicCard, CryptoCard, Suite, Value, ENCODING_CARDS_MAPPING},
    play::{PlayAction, PlayerEnv},
    schnorr::PublicKey,
    task::{Task as CoreTask, HAND_NUM},
    CiphertextAffine,
};
use poker_snark::{
    build_cs::{prove_outsource, N_CARDS, N_PLAYS},
    create_and_rescale_outsource,
    gen_params::PROVER_PARAMS,
};
use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use std::{collections::HashMap, sync::Mutex};
use z4_engine::{
    Address, DefaultParams, Error, HandleResult, Handler, PeerId, Result, RoomId, Tasks,
};
use zshuffle::{
    build_cs::prove_shuffle,
    gen_params::{gen_shuffle_prover_params, refresh_prover_params_public_key, ProverParams},
    keygen::aggregate_keys,
    mask::*,
    Ciphertext,
};

use crate::errors::NodeError;

static PARAMS: Lazy<Mutex<HashMap<usize, ProverParams>>> = Lazy::new(|| {
    let m = HashMap::new();
    Mutex::new(m)
});

pub fn init_prover_key(n: usize) {
    let mut params = PARAMS.lock().unwrap();
    if params.get(&n).is_none() {
        let pp = gen_shuffle_prover_params(n).unwrap();
        params.insert(n, pp);
    }
    drop(params);
}

pub struct PokerHandler {
    pub room_id: RoomId,
    pub accounts: HashMap<PeerId, PublicKey>,
    pub first_player: usize,
    pub players_hand: HashMap<PeerId, Vec<CryptoCard>>,
    pub players_order: Vec<PeerId>,
    // round_id => Vec<PlayerEnv>
    pub players_envs: HashMap<u8, HashMap<u8, PlayerEnv>>,
}

impl PokerHandler {
    /// when game over, prove the game process and the result,
    /// use risc0 to prove the process of player,
    /// use plonk to prove the shuffle & reveal cards is corrent.
    pub fn prove(&self) {
        let players_key = self
            .players_order
            .iter()
            .map(|x| self.accounts.get(x).cloned().unwrap())
            .collect::<Vec<_>>();
        let players_hand = self
            .players_order
            .iter()
            .map(|x| self.players_hand.get(x).cloned().unwrap())
            .collect::<Vec<_>>();
        let mut players_env = vec![];

        let max_round = self.players_envs.len() as u8;
        for round in 0..max_round {
            let mut round_env = vec![];
            let inner = self.players_envs.get(&round).unwrap();
            let max_turn = inner.len() as u8;
            for turn in 0..max_turn {
                let play_env = inner.get(&turn).cloned().unwrap();
                round_env.push(play_env)
            }
            players_env.push(round_env);
        }

        let task = CoreTask {
            room_id: self.room_id as usize,
            first_player: self.first_player,
            players_key,
            players_env,
            players_hand,
        };

        {
            let task0 = task.convert0();
            let (_receipt, session_id) = prove_bonsai(&task0).unwrap();

            let _snark_proof = stark_to_snark(session_id).unwrap();
        }

        {
            let mut rng = ChaChaRng::from_entropy();
            let (players_key, reveal_outsources, unmask_outsources, signature_outsources) =
                create_and_rescale_outsource(&task, N_PLAYS, N_CARDS);

            let _proof = prove_outsource(
                &mut rng,
                &players_key,
                &reveal_outsources,
                &unmask_outsources,
                &signature_outsources,
                &PROVER_PARAMS,
            )
            .unwrap();
        }

        // TODO serialize snark proof & plonk proof for onchain verify
    }
}

#[async_trait::async_trait]
impl Handler for PokerHandler {
    type Param = DefaultParams;

    async fn accept(peers: &[(Address, PeerId, [u8; 32])]) -> Vec<u8> {
        println!("Begin Handler Accept");
        assert_eq!(peers.len(), N_PLAYS);
        let mut rng = ChaChaRng::from_entropy();

        let pks_affine = peers
            .iter()
            .map(|(_, _, pk)| {
                EdwardsAffine::deserialize_with_mode(pk.as_slice(), Compress::Yes, Validate::Yes)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let pks: Vec<EdwardsProjective> = pks_affine.into_iter().map(|x| x.into()).collect();

        let joint_pk = aggregate_keys(&pks).unwrap();

        let mut deck = vec![];
        for card in ENCODING_CARDS_MAPPING.keys() {
            let card = (*card).into();
            let (masked_card, _) = mask(&mut rng, &joint_pk, &card, &Fr::one()).unwrap();
            deck.push(masked_card)
        }

        assert_eq!(deck.len(), N_CARDS);

        let mut params = PARAMS.lock().unwrap();
        let prover_params = if let Some(param) = params.get_mut(&N_CARDS) {
            param
        } else {
            let pp = gen_shuffle_prover_params(N_CARDS).unwrap();
            params.insert(N_CARDS, pp);
            params.get_mut(&N_CARDS).unwrap()
        };
        refresh_prover_params_public_key(prover_params, &joint_pk).unwrap();

        let (_proof, shuffle_deck) =
            prove_shuffle(&mut rng, &joint_pk, &deck, &prover_params).unwrap();

        // {
        //     let mut verifier_params = get_shuffle_verifier_params(N_CARDS).unwrap();
        //     verifier_params.verifier_params = prover_params.prover_params.verifier_params.clone();
        //     verify_shuffle(&verifier_params, &deck, &shuffle_deck, &proof).unwrap();
        // }

        drop(params);

        let mut bytes = vec![];
        for c in shuffle_deck.iter() {
            let mut e1 = vec![];
            c.e1.serialize_compressed(&mut e1).unwrap();
            bytes.extend(e1);
            let mut e2 = vec![];
            c.e2.serialize_compressed(&mut e2).unwrap();
            bytes.extend(e2);
        }

        println!("Finish Handler Accept: {}", bytes.len());

        bytes
    }

    async fn create(
        peers: &[(Address, PeerId, [u8; 32])],
        shuffle_decks: Vec<u8>,
        room_id: RoomId,
    ) -> (Self, Tasks<Self>) {
        println!("Begin Handler Create :{}",room_id);

        assert_eq!(peers.len(), N_PLAYS);

        let mut players_order = vec![];
        let accounts = peers
            .iter()
            .map(|(_, pid, pk)| {
                let pk = EdwardsAffine::deserialize_with_mode(
                    pk.as_slice(),
                    Compress::Yes,
                    Validate::Yes,
                )
                .unwrap();

                players_order.push(*pid);

                (*pid, PublicKey(pk))
            })
            .collect();

        let mut deck = vec![];
        for x in shuffle_decks.chunks(64) {
            let e1 = EdwardsProjective::deserialize_compressed(&x[..32]).unwrap();
            let e2 = EdwardsProjective::deserialize_compressed(&x[32..]).unwrap();

            deck.push(Ciphertext::new(e1, e2));
        }
        assert_eq!(deck.len(), N_CARDS);

        let player0_hand = deck[..HAND_NUM]
            .iter()
            .map(|x| CryptoCard(CiphertextAffine::from(*x)))
            .collect();
        let player1_hand = deck[HAND_NUM..2 * HAND_NUM]
            .iter()
            .map(|x| CryptoCard(CiphertextAffine::from(*x)))
            .collect();
        let player2_hand = deck[2 * HAND_NUM..]
            .iter()
            .map(|x| CryptoCard(CiphertextAffine::from(*x)))
            .collect();

        let mut players_hand = HashMap::new();
        players_hand.insert(peers[0].1, player0_hand);
        players_hand.insert(peers[1].1, player1_hand);
        players_hand.insert(peers[2].1, player2_hand);


        println!("Fininsh Handler Create");

        (
            Self {
                room_id,
                accounts,
                first_player: N_PLAYS,
                players_hand,
                players_order,
                players_envs: HashMap::new(),
            },
            Default::default(),
        )
    }

    /// when player connected to server, will send remain cards
    async fn online(&mut self, _player: PeerId) -> Result<HandleResult<Self::Param>> {
        // check the remain cards
        // send remain cards to player
        // broadcast the player online
        Ok(HandleResult::default())
    }

    /// when player offline, tell other players, then do some change in game UI
    async fn offline(&mut self, _player: PeerId) -> Result<HandleResult<Self::Param>> {
        // broadcast the player offline
        Ok(HandleResult::default())
    }

    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        params: DefaultParams,
    ) -> Result<HandleResult<Self::Param>> {
        println!(" Handler interface :{}", method);

        let public_key = self.accounts.get(&player).ok_or(Error::NoPlayer)?;
        let params = params.0;

        let mut results = HandleResult::default();

        match method {
            "play" => {

                println!(" Handler play");

                assert_eq!(params.len(), 1);
                let btyes = params[0].as_str().unwrap();
                let mut play_env: PlayerEnv =
                    serde_json::from_str(btyes).map_err(|_| Error::Params)?;
                assert_eq!(play_env.action, PlayAction::PLAY);
                assert!(play_env.verify_sign(public_key).is_ok());

                let ordered_key = self
                    .players_order
                    .iter()
                    .map(|x| self.accounts.get(x).unwrap().clone())
                    .collect::<Vec<_>>();
                play_env.sync_reveal_order(&ordered_key);

                let current_round = self.players_envs.len() as u8;

                match play_env.round_id.cmp(&current_round) {
                    std::cmp::Ordering::Less => {
                        process_error_response(
                            &mut results,
                            player,
                            &NodeError::PlayRoundError.to_string(),
                        );
                        return Ok(results);
                    }
                    std::cmp::Ordering::Equal => {
                        let round_info = self
                            .players_envs
                            .entry(play_env.round_id)
                            .or_insert(HashMap::new());
                        let mut sorted_keys: Vec<_> = round_info.keys().cloned().collect();
                        sorted_keys.sort_by(|a, b| b.cmp(a));

                        let round_over = (!round_info.is_empty())
                            && sorted_keys
                                .iter()
                                .take(N_PLAYS - 1)
                                .all(|&k| round_info[&k].action != PlayAction::PLAY)
                            && round_info.iter().any(|x| x.1.action == PlayAction::PLAY);

                        if round_over {
                            process_error_response(
                                &mut results,
                                player,
                                &NodeError::RoundOverError.to_string(),
                            );
                            return Ok(results);
                        }
                    }
                    std::cmp::Ordering::Greater => {
                        let previous_round = self.players_envs.get(&current_round).unwrap();
                        let mut sorted_keys: Vec<_> = previous_round.keys().cloned().collect();
                        sorted_keys.sort_by(|a, b| b.cmp(a));

                        let round_over = (!previous_round.is_empty())
                            && previous_round
                                .iter()
                                .any(|x| x.1.action == PlayAction::PLAY);

                        let round_info = self
                            .players_envs
                            .entry(play_env.round_id)
                            .or_insert(HashMap::new());

                        let round_over = round_over
                            && sorted_keys
                                .iter()
                                .take(N_PLAYS - 1)
                                .all(|&k| round_info[&k].action != PlayAction::PLAY);

                        if !round_over || !round_info.is_empty() {
                            process_error_response(
                                &mut results,
                                player,
                                &NodeError::PlayRoundError.to_string(),
                            );
                            return Ok(results);
                        }
                    }
                }

                let classic = play_env.play_classic_cards.clone().unwrap();
                assert!(classic.check_format());

                if self.first_player == N_PLAYS {
                    if !classic.contains(&ClassicCard::new(Value::Three, Suite::Heart)) {
                        process_error_response(
                            &mut results,
                            player,
                            &NodeError::PlayFirstError.to_string(),
                        );
                        return Ok(results);
                    }
                }

                let play_crypto_cards = play_env.play_crypto_cards.clone().unwrap().to_vec();
                let hand = self.players_hand.get_mut(&player).unwrap();
                let hand_len = hand.len();
                let play_len = play_crypto_cards.len();
                for element in play_crypto_cards {
                    if let Some(index) = hand.iter().position(|&x| x == element) {
                        hand.remove(index);
                    }
                }
                let remainder_len = hand.len();
                assert_eq!(hand_len - play_len, remainder_len);

                let round_info = self
                    .players_envs
                    .entry(play_env.round_id)
                    .or_insert(HashMap::new());
                round_info
                    .entry(play_env.turn_id)
                    .or_insert(play_env.clone());

                process_play_response(&mut results, player, classic.to_bytes());

                println!("Finish Handler play");
            }

            "pass" => {
                println!(" Handler pass");

                assert_eq!(params.len(), 1);
                let btyes = params[0].as_str().unwrap();
                let play_env: PlayerEnv = serde_json::from_str(btyes).map_err(|_| Error::Params)?;
                assert_eq!(play_env.action, PlayAction::PAAS);
                assert!(play_env.verify_sign(public_key).is_ok());

                let round_info = self
                    .players_envs
                    .entry(play_env.round_id)
                    .or_insert(HashMap::new());
                round_info.entry(play_env.turn_id).or_insert(play_env);

                process_pass_response(&mut results, player);

                println!("Finish Handler pass");
            }

            "revealRequest" => {
                println!("Handler revealRequest:{:?}",params);

                assert_eq!(params.len(), 4);
                process_reveal_request(&mut results, player, params);

                println!("Finish Handler revealRequest ");
            }

            "revealResponse" => {
                println!("Handler revealResponse :{:?}",params);

                // vec![peerId, crypto_card, reveal_card, reveal_proof, public_key]
                assert_eq!(params.len(), 5);
                let peer_id = params[0].as_array().unwrap();
                let peer_id: Vec<u8> = peer_id.iter().map(|x| x.as_u64().unwrap() as u8).collect();
                let peer_id = PeerId(peer_id.try_into().unwrap());
                process_reveal_response(&mut results, peer_id, &params[1..]);

                println!("Finish Handler revealResponse ");
            }

            _ => unimplemented!(),
        }

        Ok(results)
    }
}

fn process_play_response(
    results: &mut HandleResult<DefaultParams>,
    pid: PeerId,
    play_cards: Vec<u8>,
) {
    results.add_all(
        "play",
        DefaultParams(vec![pid.0.to_vec().into(), play_cards.into()]),
    );
}

fn process_pass_response(results: &mut HandleResult<DefaultParams>, pid: PeerId) {
    results.add_all("pass", DefaultParams(vec![pid.0.to_vec().into()]));
}

fn process_reveal_request(
    results: &mut HandleResult<DefaultParams>,
    pid: PeerId,
    reveal_card: Vec<serde_json::Value>,
) {
    results.add_all(
        "revealRequest",
        DefaultParams(vec![pid.0.to_vec().into(), reveal_card.into()]),
    );
}

fn process_reveal_response(
    results: &mut HandleResult<DefaultParams>,
    pid: PeerId,
    reveal_proof: &[serde_json::Value],
) {
    results.add_one(pid, "revealResponse", DefaultParams(reveal_proof.to_vec()));
}

fn process_error_response(results: &mut HandleResult<DefaultParams>, pid: PeerId, error_msg: &str) {
    results.add_one(pid, "error", DefaultParams(vec![error_msg.into()]));
}

#[cfg(test)]
mod test {
    use ark_ed_on_bn254::EdwardsProjective;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use poker_core::{
        cards::{reveal0, unmask, verify_reveal0, ENCODING_CARDS_MAPPING},
        schnorr::KeyPair,
    };
    use poker_snark::build_cs::N_CARDS;
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
    use z4_engine::{json, Address, DefaultParams, Handler, PeerId};
    use zshuffle::Ciphertext;

    use super::{init_prover_key, PokerHandler};

    #[test]
    fn t() {
    }

    #[tokio::test]
    async fn test_accept() {
        let mut rng = ChaChaRng::from_seed([0u8; 32]);

        let keypair_a = KeyPair::sample(&mut rng);
        let keypair_b = KeyPair::sample(&mut rng);
        let keypair_c = KeyPair::sample(&mut rng);

        let mut pk_a_vec = vec![];
        let mut pk_b_vec = vec![];
        let mut pk_c_vec = vec![];
        keypair_a
            .get_public_key()
            .0
            .serialize_compressed(&mut pk_a_vec)
            .unwrap();
        keypair_b
            .get_public_key()
            .0
            .serialize_compressed(&mut pk_b_vec)
            .unwrap();
        keypair_c
            .get_public_key()
            .0
            .serialize_compressed(&mut pk_c_vec)
            .unwrap();

        let peers = vec![
            (
                Address::default(),
                PeerId::default(),
                pk_a_vec.try_into().unwrap(),
            ),
            (
                Address::default(),
                PeerId::default(),
                pk_b_vec.try_into().unwrap(),
            ),
            (
                Address::default(),
                PeerId::default(),
                pk_c_vec.try_into().unwrap(),
            ),
        ];

        init_prover_key(N_CARDS);
        let deck = PokerHandler::accept(&peers).await;

        let mut last_deck = vec![];
        for x in deck.chunks(64) {
            let e1 = EdwardsProjective::deserialize_compressed(&x[..32]).unwrap();
            let e2 = EdwardsProjective::deserialize_compressed(&x[32..]).unwrap();
            let card = Ciphertext::new(e1, e2);
            last_deck.push(card.into());
        }

        for card in last_deck.iter() {
            let (reveal_card_a, reveal_proof_a) = reveal0(&mut rng, &keypair_a, card).unwrap();
            let (reveal_card_b, reveal_proof_b) = reveal0(&mut rng, &keypair_b, card).unwrap();
            let (reveal_card_c, reveal_proof_c) = reveal0(&mut rng, &keypair_c, card).unwrap();

            verify_reveal0(
                &keypair_a.get_public_key(),
                &card,
                &reveal_card_a,
                &reveal_proof_a,
            )
            .unwrap();
            verify_reveal0(
                &keypair_b.get_public_key(),
                &card,
                &reveal_card_b,
                &reveal_proof_b,
            )
            .unwrap();
            verify_reveal0(
                &keypair_c.get_public_key(),
                &card,
                &reveal_card_c,
                &reveal_proof_c,
            )
            .unwrap();

            let reveals = vec![reveal_card_a, reveal_card_b, reveal_card_c];
            let unmask = unmask(&card, &reveals);
            let classic = ENCODING_CARDS_MAPPING.get(&unmask.0).unwrap();
            println!("{:?}", classic);
        }
    }
}
