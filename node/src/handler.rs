use ark_ec::AffineRepr;
use ark_ed_on_bn254::{EdwardsAffine, EdwardsProjective, Fr};
use ark_ff::{BigInteger, One, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use once_cell::sync::Lazy;
use poker_bonsai::{snark::stark_to_snark, stark::prove_bonsai};
use poker_core::{
    cards::{ClassicCard, CryptoCard, Suite, Value, DECK, ENCODING_CARDS_MAPPING},
    left_rotate,
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
use serde_json::Map;
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};
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

    pub round_id: u8,
    pub turn_id: u8,
    pub reveal_info: HashMap<String, Vec<serde_json::Value>>,
    pub traces: Vec<Map<String, serde_json::Value>>,
    pub connected_players: HashSet<PeerId>,
    pub has_reveal: bool,
    pub revealed: HashSet<PeerId>,
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
            // let task0 = task.convert0();
            // let (_receipt, session_id) = prove_bonsai(&task0).unwrap();

            // let _snark_proof = stark_to_snark(session_id).unwrap();
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
        println!("Begin Handler Create :{}", room_id);

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

        let mut players_hand = HashMap::new();
        for (i, x) in deck.chunks_exact(HAND_NUM).enumerate() {
            let hand = x
                .iter()
                .map(|x| CryptoCard(CiphertextAffine::from(*x)))
                .collect::<Vec<_>>();

            players_hand.insert(peers[i].1, hand);
        }

        println!("Fininsh Handler Create");

        (
            Self {
                room_id,
                accounts,
                first_player: N_PLAYS,
                players_hand,
                players_order,
                players_envs: HashMap::new(),
                round_id: 0,
                turn_id: 0,
                reveal_info: HashMap::new(),
                traces: vec![],
                connected_players: HashSet::new(),
                has_reveal: false,
                revealed: HashSet::new(),
            },
            Default::default(),
        )
    }

    /// when player connected to server, will send remain cards
    async fn online(&mut self, player: PeerId) -> Result<HandleResult<Self::Param>> {
        let mut hand: Map<String, serde_json::Value> = Map::new();
        let mut reveal_info: Map<String, serde_json::Value> = Map::new();
        for (k, v) in self.players_hand.iter() {
            let ks = k.to_hex();
            let mut tmp_reveal = vec![];

            let vs: Vec<serde_json::Value> = v
                .iter()
                .map(|x| {
                    let e1 = point_to_hex(&x.0.e1);
                    let e2 = point_to_hex(&x.0.e2);
                    let r = vec![e1.0, e1.1, e2.0, e2.1];

                    let rk: String = r.iter().flat_map(|x| x.chars()).collect();
                    match self.reveal_info.get(&rk) {
                        Some(info) => tmp_reveal.push(info.to_vec().into()),
                        None => tmp_reveal.push(serde_json::Value::Null),
                    };

                    r.into()
                })
                .collect();
            hand.insert(ks.clone(), vs.into());
            reveal_info.insert(ks, tmp_reveal.into());
        }

        let player_order: Vec<_> = self.players_order.iter().map(|x| x.to_hex()).collect();
        let mut game_info: Map<String, serde_json::Value> = Map::new();
        game_info.insert("player_order".to_string(), player_order.into());
        game_info.insert("room_id".to_string(), self.room_id.into());
        game_info.insert("round_id".to_string(), self.round_id.into());
        game_info.insert("turn_id".to_string(), self.turn_id.into());
        game_info.insert("first_player".to_string(), self.first_player.into());
        game_info.insert("online_player".to_string(), player.to_hex().into());

         println!("reveal_info:{:?}", reveal_info.clone());


        let mut results = HandleResult::default();
        results.add_all(
            "online",
            DefaultParams(vec![
                hand.clone().into(),
                game_info.into(),
                reveal_info.into(),
                self.traces.clone().into(),
            ]),
        );

        self.connected_players.insert(player);
        if self.connected_players.len() == N_PLAYS && !self.has_reveal {
            for k in self.players_order.iter() {
                let ks = k.to_hex();
                let v = hand.get(&ks).unwrap();

                process_reveal_request(&mut results, *k, v.clone());
            }

            self.has_reveal = true;
        }

        Ok(results)
    }

    /// when player offline, tell other players, then do some change in game UI
    async fn offline(&mut self, player: PeerId) -> Result<HandleResult<Self::Param>> {
        let mut results = HandleResult::default();
        results.add_all("offline", DefaultParams(vec![player.to_hex().into()]));
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
              //  assert!(play_env.verify_sign(public_key).is_ok());

                // let current_round =  if self.players_envs.len() ==0  {
                //     0
                // }else {
                //     self.players_envs.keys().max().unwrap().to_owned()
                // };

                // match play_env.round_id.cmp(&current_round) {
                //     std::cmp::Ordering::Less => {
                //         println!("=l= {},{}",play_env.round_id, current_round);
                //         process_error_response(
                //             &mut results,
                //             player,
                //             &NodeError::PlayRoundError.to_string(),
                //         );
                //         return Ok(results);
                //     }
                //     std::cmp::Ordering::Equal => {
                //         println!("=e= {},{}",play_env.round_id, current_round);

                //         let round_info = self
                //             .players_envs
                //             .entry(play_env.round_id)
                //             .or_insert(HashMap::new());
                //         let mut sorted_keys: Vec<_> = round_info.keys().cloned().collect();
                //         sorted_keys.sort_by(|a, b| b.cmp(a));

                //         let round_over = (!round_info.is_empty())
                //             && sorted_keys
                //                 .iter()
                //                 .take(N_PLAYS - 1)
                //                 .all(|&k| round_info[&k].action != PlayAction::PLAY)
                //             && round_info.iter().any(|x| x.1.action == PlayAction::PLAY);

                //         if round_over {
                //             process_error_response(
                //                 &mut results,
                //                 player,
                //                 &NodeError::RoundOverError.to_string(),
                //             );
                //             return Ok(results);
                //         }
                //     }
                //     std::cmp::Ordering::Greater => {
                //         println!("=g= {},{}",play_env.round_id, current_round);

                //         let previous_round = self.players_envs.get(&current_round).unwrap();
                //         let mut sorted_keys: Vec<_> = previous_round.keys().cloned().collect();
                //         sorted_keys.sort_by(|a, b| b.cmp(a));

                //         let round_over = (!previous_round.is_empty())
                //             && previous_round
                //                 .iter()
                //                 .any(|x| x.1.action == PlayAction::PLAY);

                //          println!("over1 {},{:?}",round_over,previous_round);

                //         let current = self
                //             .players_envs
                //             .get(&play_env.round_id);

                //         let round_over = round_over
                //             && sorted_keys
                //                 .iter()
                //                 .take(N_PLAYS - 1)
                //                 .all(|&k| previous_round[&k].action != PlayAction::PLAY);

                //         if !round_over || current.is_some() {
                //             println!("great return {},{},",round_over,current.is_some());
                //             process_error_response(
                //                 &mut results,
                //                 player,
                //                 &NodeError::PlayRoundError.to_string(),
                //             );
                //             return Ok(results);
                //         }
                //     }
                // }

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
                    self.first_player = self
                        .players_order
                        .iter()
                        .position(|x| *x == player)
                        .unwrap();
                }

                let ordered_key = self
                    .players_order
                    .iter()
                    .map(|x| self.accounts.get(x).unwrap().clone())
                    .collect::<Vec<_>>();
                play_env.sync_reveal_order(&left_rotate(&ordered_key, self.first_player));

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

                let round_id = play_env.round_id;
                let turn_id = play_env.turn_id;

                let round_info = self
                    .players_envs
                    .entry(play_env.round_id)
                    .or_insert(HashMap::new());
                round_info.entry(turn_id).or_insert(play_env);

                self.round_id = round_id;
                self.turn_id = turn_id;

                let classic_index = round_info
                    .get(&turn_id)
                    .cloned()
                    .unwrap()
                    .play_classic_cards
                    .unwrap()
                    .to_vec()
                    .iter()
                    .map(|x| DECK.iter().position(|y| x == y).unwrap())
                    .collect::<Vec<_>>();

                let mut trace = Map::new();
                trace.insert("action".to_string(), "play".into());
                trace.insert("cards".to_string(), classic_index.clone().into());
                trace.insert("player".to_string(), player.to_hex().into());
                self.traces.push(trace);

                println!("Finish Handler play");

                process_play_response(&mut results, player, classic_index);

                println!("remainder_len:{}", remainder_len);
                if remainder_len == 0 {
                    println!("game over, beign prove");
                    self.prove();
                    println!("finish prove");
                }

                println!("Finish Handler play");
            }

            "pass" => {
                println!(" Handler paas");

                assert_eq!(params.len(), 1);
                let btyes = params[0].as_str().unwrap();
                let play_env: PlayerEnv = serde_json::from_str(btyes).map_err(|_| Error::Params)?;
                assert_eq!(play_env.action, PlayAction::PAAS);
                assert!(play_env.verify_sign(public_key).is_ok());

                self.round_id = play_env.round_id;

                let round_info = self
                    .players_envs
                    .entry(play_env.round_id)
                    .or_insert(HashMap::new());
                round_info.entry(play_env.turn_id).or_insert(play_env);

                self.turn_id = round_info.len() as u8;

                let mut trace = Map::new();
                trace.insert("action".to_string(), "paas".into());
                trace.insert("cards".to_string(), serde_json::Value::Null);
                trace.insert("player".to_string(), player.to_hex().into());
                self.traces.push(trace);

                process_pass_response(&mut results, player);

                println!("Finish Handler pass");
            }

            "revealRequest" => {
                println!("Handler revealRequest:{:?}", params);

                assert!(params.len() <= N_CARDS);
                process_reveal_request(&mut results, player, params.into());

                println!("Finish Handler revealRequest ");
            }

            "revealResponse" => {
                // vec![peerId, vec![crypto_card, reveal_card, reveal_proof, public_key]]
                assert!(params.len() == 2);
                println!("Handler revealResponse");

                let peer_id = params[0].as_array().unwrap();
                let peer_id: Vec<String> = peer_id
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect();

                println!(
                    "------------------------------------------------------------------{:?}",
                    peer_id
                );

                let peer_id = peer_id
                    .iter()
                    .map(|x| PeerId::from_hex(x).unwrap())
                    .collect::<Vec<_>>();

                let reveal_info = params[1].as_array().unwrap();

                for (v, id) in reveal_info.iter().zip(peer_id.iter()) {
                    {
                        let rs = v.as_array().unwrap();
                        let hands = self.players_hand.get(id).unwrap();
                        if rs.len() == HAND_NUM
                            && hands.len() == HAND_NUM
                            && !self.revealed.contains(&player)
                        {
                            println!("-------------{},{}", rs.len(), hands.len());

                            for (r, c) in rs.iter().zip(hands.iter()) {
                                let map = r.as_object().unwrap();
                                let card = map
                                    .get("card")
                                    .unwrap()
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|x| x.as_str().unwrap().to_owned())
                                    .collect::<Vec<_>>();
                                let proof = map.get("proof").unwrap().as_str().unwrap().to_owned();
                                let pk =
                                    map.get("public_key").unwrap().as_str().unwrap().to_owned();

                                let e1 = point_to_hex(&c.0.e1);
                                let e2 = point_to_hex(&c.0.e2);
                                let r = vec![e1.0, e1.1, e2.0, e2.1];

                                let rk: String = r.iter().flat_map(|x| x.chars()).collect();
                                let reveal_info = self.reveal_info.entry(rk).or_insert(vec![]);

                                let info: Vec<serde_json::Value> =
                                    vec![card.into(), proof.into(), pk.into()];
                                reveal_info.push(info.into());
                            }
                        }
                    }
                    process_reveal_response(&mut results, *id, v);
                }

                self.revealed.insert(player);

                println!("Finish Handler revealResponse");
            }

            _ => unimplemented!(),
        }

        Ok(results)
    }
}

fn process_play_response(
    results: &mut HandleResult<DefaultParams>,
    pid: PeerId,
    play_cards: Vec<usize>,
) {
    results.add_all(
        "play",
        DefaultParams(vec![pid.to_hex().into(), play_cards.into()]),
    );
}

fn process_pass_response(results: &mut HandleResult<DefaultParams>, pid: PeerId) {
    results.add_all("pass", DefaultParams(vec![pid.to_hex().into()]));
}

fn process_reveal_request(
    results: &mut HandleResult<DefaultParams>,
    pid: PeerId,
    reveal_card: serde_json::Value,
) {
    results.add_all(
        "revealRequest",
        DefaultParams(vec![pid.to_hex().into(), reveal_card]),
    );
}

fn process_reveal_response(
    results: &mut HandleResult<DefaultParams>,
    pid: PeerId,
    reveal_proof: &serde_json::Value,
) {
    results.add_one(
        pid,
        "revealResponse",
        DefaultParams(vec![reveal_proof.clone()]),
    );
}

fn process_error_response(results: &mut HandleResult<DefaultParams>, pid: PeerId, error_msg: &str) {
    results.add_one(pid, "error", DefaultParams(vec![error_msg.into()]));
}

pub fn point_to_hex(point: &EdwardsAffine) -> (String, String) {
    let (x, y) = point.xy().unwrap();
    let x_bytes = x.into_bigint().to_bytes_be();
    let y_bytes = y.into_bigint().to_bytes_be();
    let x = hex::encode(&x_bytes);
    let y = hex::encode(&y_bytes);
    (format!("0x{}", x), format!("0x{}", y))
}

#[cfg(test)]
mod test {
    use ark_ed_on_bn254::EdwardsProjective;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use poker_core::{
        cards::{reveal0, unmask, verify_reveal0, ENCODING_CARDS_MAPPING},
        mock_data::task::mock_task,
        schnorr::KeyPair,
    };
    use poker_snark::build_cs::N_CARDS;
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
    use z4_engine::{json, Address, Handler, PeerId};
    use zshuffle::Ciphertext;

    use super::{init_prover_key, PokerHandler};

    #[test]
    fn t() {
        let x = json!([["hhe", "uuu"], ["yyy"]]);

        println!("{:?}", x)
    }

    #[tokio::test]
    async fn test_accept_and_create() {
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
                PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06d").unwrap(),
                pk_a_vec.try_into().unwrap(),
            ),
            (
                Address::default(),
                PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06e").unwrap(),
                pk_b_vec.try_into().unwrap(),
            ),
            (
                Address::default(),
                PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06f").unwrap(),
                pk_c_vec.try_into().unwrap(),
            ),
        ];

        init_prover_key(N_CARDS);
        let deck = PokerHandler::accept(&peers).await;

        let (handler, _) = PokerHandler::create(&peers, deck.clone(), 0).await;
        let handler_deck = handler
            .players_order
            .iter()
            .flat_map(|x| handler.players_hand.get(x).cloned().unwrap())
            .map(|x| x.0)
            .collect::<Vec<_>>();

        let mut last_deck = vec![];
        for x in deck.chunks(64) {
            let e1 = EdwardsProjective::deserialize_compressed(&x[..32]).unwrap();
            let e2 = EdwardsProjective::deserialize_compressed(&x[32..]).unwrap();
            let card = Ciphertext::new(e1, e2);
            last_deck.push(card.into());
        }

        assert_eq!(handler_deck, last_deck);

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

    #[tokio::test]
    #[cfg(not(all(feature = "serialize0", feature = "deserialize0")))]
    async fn test_handle() {
        println!("--1");
        use std::collections::HashMap;
        use z4_engine::DefaultParams;

        let task = mock_task();

        let mut pk_bytes = vec![];
        for x in task.players_key.iter() {
            let mut bytes = vec![];
            x.0.serialize_compressed(&mut bytes).unwrap();
            pk_bytes.push(bytes)
        }

        let peers = vec![
            (
                Address::default(),
                PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06d").unwrap(),
                pk_bytes[0].clone().try_into().unwrap(),
            ),
            (
                Address::default(),
                PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06e").unwrap(),
                pk_bytes[1].clone().try_into().unwrap(),
            ),
            (
                Address::default(),
                PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06f").unwrap(),
                pk_bytes[2].clone().try_into().unwrap(),
            ),
        ];

        let mut keys = HashMap::new();
        keys.insert(
            pk_bytes[0].clone(),
            PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06d").unwrap(),
        );
        keys.insert(
            pk_bytes[1].clone(),
            PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06e").unwrap(),
        );
        keys.insert(
            pk_bytes[2].clone(),
            PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06f").unwrap(),
        );

        let deck = task.players_hand.into_iter().flatten().collect::<Vec<_>>();
        let deck_inner = deck.iter().map(|x| x.0).collect::<Vec<_>>();
        let mut bytes = vec![];
        for c in deck.iter() {
            let mut e1 = vec![];
            c.0.e1.serialize_compressed(&mut e1).unwrap();
            bytes.extend(e1);
            let mut e2 = vec![];
            c.0.e2.serialize_compressed(&mut e2).unwrap();
            bytes.extend(e2);
        }

        let (mut handler, _) = PokerHandler::create(&peers, bytes, 1).await;
        let handler_deck = handler
            .players_order
            .iter()
            .flat_map(|x| handler.players_hand.get(x).cloned().unwrap())
            .map(|x| x.0)
            .collect::<Vec<_>>();
        assert_eq!(handler_deck, deck_inner);

        let mut i = task.first_player;
        for envs in task.players_env.iter() {
            for env in envs.iter() {
                let peer = peers[i % 3].1;
                let params = serde_json::to_string(env).unwrap();
                let action = match env.action {
                    poker_core::play::PlayAction::PAAS => "pass",
                    poker_core::play::PlayAction::PLAY => "play",
                    poker_core::play::PlayAction::OFFLINE => unimplemented!(),
                };

                let _ = handler
                    .handle(peer, action, DefaultParams(vec![params.into()]))
                    .await
                    .unwrap();

                i = i + 1;
            }
        }
    }

    #[tokio::test]
    #[cfg(not(all(feature = "serialize0", feature = "deserialize0")))]
    async fn test_reveal_response() {
        use std::{collections::HashMap, str::FromStr};
        use z4_engine::DefaultParams;

        let task = mock_task();

        let mut pk_bytes = vec![];
        for x in task.players_key.iter() {
            let mut bytes = vec![];
            x.0.serialize_compressed(&mut bytes).unwrap();
            pk_bytes.push(bytes)
        }

        let peers = vec![
            (
                Address::default(),
                PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06d").unwrap(),
                pk_bytes[0].clone().try_into().unwrap(),
            ),
            (
                Address::default(),
                PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06e").unwrap(),
                pk_bytes[1].clone().try_into().unwrap(),
            ),
            (
                Address::default(),
                PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06f").unwrap(),
                pk_bytes[2].clone().try_into().unwrap(),
            ),
        ];

        let mut keys = HashMap::new();
        keys.insert(
            pk_bytes[0].clone(),
            PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06d").unwrap(),
        );
        keys.insert(
            pk_bytes[1].clone(),
            PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06e").unwrap(),
        );
        keys.insert(
            pk_bytes[2].clone(),
            PeerId::from_hex("0x54f387596caeabf85c19c27162cb0ae9fab8f06f").unwrap(),
        );

        let deck = task.players_hand.into_iter().flatten().collect::<Vec<_>>();
        let mut bytes = vec![];
        for c in deck.iter() {
            let mut e1 = vec![];
            c.0.e1.serialize_compressed(&mut e1).unwrap();
            bytes.extend(e1);
            let mut e2 = vec![];
            c.0.e2.serialize_compressed(&mut e2).unwrap();
            bytes.extend(e2);
        }

        let (mut handler, _) = PokerHandler::create(&peers, bytes, 1).await;

        let s2 = r##"
		[{
				"card": ["0x2b9ff8125b3eabdb53a82b9a05f8934ec43bd4965bdcd95eb39461c03548364f", "0x08730cdb205392cb6982a6e1505c50c415094a6334a64e972b5abe96586ceb8a"],
				"proof": "0x0f77e4ccb8ab782639b1f14545ac929185bbe1610c183e7295284c13718a5a8f1304b6a2d54367b3ce20689822472b12cfd3613479802f87d8102ca9293fb7df25343299a8980db9ae9046c772a1d898a0b5fef5d0b22d85ccc94b0ec43f36e90452b89bd8b53647ed4deb884818a84aed50c0a42cad38c085a38d10a82e98980419861391794344beb2fa2082a6a39bf417337f0ac2bf2b87e6f8b42771cbca",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x1f92089db8e7b0f1dec89f99ce7244d7ebae246783fd3ff91c8b8ef27fcec6ed", "0x0c87b8716c30c41c34c92aa2c5b25cf206643368c83322039ab47f1c217dbf06"],
				"proof": "0x2d50e3aa1068967f029463e522e17a07dc4538ce54d725654edf2396811086c2228f1ff6230f7a6cc5ff2a5800d9b9178335809b0d1598e1a18a3082f625d67c1cd215fb2632a7bfc58426a290ab09965d7633d3eae3e33757c1882c2549d75f2905b61480555beccb00efc6800a7aa13c0b2788c86e05ac05ae9eb021c9f08b02016f0a7cba19ec82cadb3233fd6232d992cbacf0c01476cf90ab3fc9d6da12",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x140740d95a9a79c0a52df4a63734815326ff275ab24b6eafbc01e656d4c593ca", "0x1bef7f903294071571a32790d40670bb55ce18a9c999140f132623ddb06bf477"],
				"proof": "0x002279dc71e96ec76c31c9c53aefb09f3509cbe8495bac76f910e6ebc924a03d00221ec0c7741c62a767d7f6d5836d2c47da0354b5ede5a91c6e9e29f3cf61f80766ba031f97647036c818d658dbdba2070325c8a6789063297fbb28168fc257192c2ec919b27868c528bde5debf3a4a23dcbe58b067a1747d619dd72c51f55801e0f9528571022cf99d9b6a82c3245d480b33750b9ff158fe92498b89f547ec",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x2185b1e58e938e268577e4c1ca1a60a011b1ac084cbcc6a573fc53be39cc815f", "0x09ff44576534262190f1ef4d78aa94d6c8ebd8ccb73d8e06049a4656a5ee96cb"],
				"proof": "0x2861a26b266fd61b6cb6518f0a873e0eae9e8ca8a894346c6c3b7e1aa5138352049559ddc6c1f4773f657122fa85ef29c051c044abeb2e105af5ea13b36593500a07cf999076fa115eae59410ba861b9f7a76b595f2cec4e34621fbe0e7c4dc3294b4083b153c01f2df54232284f65a360415579cae160da42673b618414fca602e576b1842dcb972b4754b96e06a045f2364cf247b085d45900decf4160c93d",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x0567db0b3f91e3ba1ead36c88ff1edada8362d7eb5895ee024685522f7315bfe", "0x172a77a54b977bed298a170df7aab7f442a1962da737f4a7c5f888a5bd76b5b6"],
				"proof": "0x0d898177a8b6a394532afaf303137fc50d0580af8508af7811e4a495aa2de47c231167c3c42f12ab605803d4254c51ae9edf59cd9e65bd98e2980caf5a0d94b21b5314780e0bc8c2e82fc3052db1b2f007806e17bbe5468d07efd8d066a07e98049192cac62ec0553fe87849bf1b83908829a654d2ca033b4899a3016096a94305f5d42690ad46d76c2917117e7caba26e52369702f050229feb5e530d246b79",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x0a74cce1bd75370302a3fe83a937790932cece05bf9b02e471aa927905624b30", "0x0e7d015cbbada4be54136a5daf10721a5c660938cccaa308e8a51f5dbdaa9502"],
				"proof": "0x0280353e5d68ea99e3e120027484e741e04d7b5972f3908f1f465645cb3a0fa62b93be878e820890e11ef1a2bf851d6b75e6f6cd0b0f2ca3fd26c9d152270c8b20febad4502e1b203023ff02a27b56c282777e73a2d9119ef1d3e9ad3ae4f5a92b71b158e44c9eed39504207f1965145489df6b90ce91f53192f7e23e6483f160071f3e0c032a6a8b0eaf08ec659dc0ec1e1412d5cddfe4d8cab815a41299a0d",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x260639d5413c5a310b0d1b96cdf282474e741d72f19b268cf6fa5f04895c3088", "0x173d9946bf7da9cd2bad113a3d23fa9523af78a1af8b87602a62807f3822978b"],
				"proof": "0x2be1dfbf9f667f1dac79e14ac166fa94353e9829121c3d96b79bb02e33c505c705e79c1fb25dfa28a6f96faebe2d14671d715d80b8d8d8907b81ea9e204ef7d0071647d45b2f227553f2b17a3a776c566bd548f7e5c547bd15c34ee24f0f08912067ab98dfd97d5339f87875062da03323aa2d11413764f0e031ad3cc650292a0388202fc6d212624bee4e7710fe4f0a652df42a8721144345cff9374de0db4d",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x24edaf07099912ef2d27894faf489b4f066e75b813d80843f9512fd56f037ef3", "0x23dd2bc7ecef446e482d357356f111bed7818d10842897741e976f4f33c62def"],
				"proof": "0x067e148c54cb6061668a41745ba96e4eb5fa500399fac590f48782e0d20998ec1771f5d891f48442e36897b0af7328e3649b86f3b80effba66a74b2c5e5f0b6c168d23996abd32bde4386d58eb827bd7f1bc0e77abc03f197521ef3167c4da510a5e93417789f700106f8d95e363ff31333c9acd2c3181c5e3eee22956439939019312474629d381b08e8884a27155a5f238b776e66daa35957c6a1844366617",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x2da03d97d7def1cf0e1e05910b13068edd4812989453408a490a5fde35a6f5d1", "0x1e8f985177a88e219a3b9481270c3579634543e34f7b05e7b5d064b7b8a2bff1"],
				"proof": "0x2f83ddb37c949a7e0d573db04ddd4469035733b2a208fae7dc5b73cdf972bb970f77e7c471bbc9d0f7c31a56e0eb50833466959cd423101491e8d4e67b35015313e7caf7428d20acb87866b8cf37e0b0d144a2a9c0ec96cce371b122a1bbc1ea137e0b07241e174e07f0d42f139412469980979ad55af86444f0cdd9bb95dc9b00e0f7e73f360e11ef81a52eb7a65d4fa6dc3766350ee212aefae8b35a2fab7f",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x09a104a08991267b6ade624ee1a290fe6b5098f1f0966268dee548df83c6e485", "0x2b173b4386a012bd1920af49ddffd6eddfee87d977e5679eae20c174296c1058"],
				"proof": "0x1e2336a25be5da67b47830741e8a56ee9177c8fae67d78cf91a52105f3252bb005931db77904345d7902de724f348533d303d7f5166f8575c3fb166ca851871b0012d5ce64021ae919ed4b65402c1546e4e312be972f93988070e24449f6e2650a24d00ed028a4294b545c316051cf9a7547b58044a0977a97802739aee27ba801e5f05c36adf7e7ea1a1b046dfbe6d7fd5131fe593fb785288d40e738672b6a",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x0f2e1e802e5a3e0dc6793f340f04613bf3fa8032050249e8a4aef6d39b736bd3", "0x260c61e67c5069347de7e2b17b508ec4ac9a1341d7e740d8d07e7f8bba843529"],
				"proof": "0x0b93ebf4bab0183fb37bdbff4a1235c52edb235d2d63fcd1f82e8a779fbe42a8079bd898f97ac668cc6c9aa84cf44212c48817212671c79ccd7be087225660740de698855f913bd396315eb57588fcb89ec3902475cfcb77b7edf3a2f39e0517178101b2c3ac6a68ac613537c9a9bb22ddc1a795f1e85186b4acf9bb4e2e946e002d44f4b27adc9b5720467e126bf82f6375e58605c5e2357d09aa5de37ac24b",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x0ec703cd90c9ee1325db46e3129109c6f27c63d156a15a8a0800349e6e009890", "0x2d660d9ba402ca3a96751a3513d441f5ba09549dc04072f84cf7202da6e94f4a"],
				"proof": "0x1f0f1ccf021828e34787da3eee6d1526438b184a5f59ad7a69a06b03a82605ac0a1970f6951a38a3967bd08810c24fe156d015ac3e8f2fc4421394f1214014080c0ea8a0f2516c3efd6345f565722b80d7755f2a72d2e0c2bfd8e1356dc6341407abbd2d04e2b78c16456afcec220d0016f7bb765e7f15718a3a2a8f4365bddb027e47931647053916a0167fc512a40acf7b84ca6f7f50228ef015deb1108463",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x174660ffef1398180488aee42ec4a29d933db225d36001ed58e6af3c0b479b51", "0x02f495a6209a9f9e9b90d2366a339a1454f93ef732bf0858ca6463a4926f1072"],
				"proof": "0x2af0d9dd0b9190429e7b91da001b1112280a09cfe18215cf45c02f27744839c119b5da1ba70d4c16383c53ba906dfb292684b0de007962c282d14419ef1e1df403bf58a16441cbae8170460304868b566ade3701d8703ac05d4cc5e3ba70897f1d2a513694c361525413932a4d504ac069742fd6241517ff34cf68200314080001193223924dd979b03886f4f6c8ec606108d8a058bafd9bbaa530d413c9fbb0",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x0605245cc44474dcf221359c065a4c7a525075a503198d62a959199ba8739997", "0x20c92c719993e62bbed740698f0586014654135211c80233e484581f0cde9696"],
				"proof": "0x05ef964ab0eb90f8e70997a467bd52d599bb7c7b89202ef985ee5693dc96080816d47a0ff994ea742cf8037a2afc61ad2aa967d2c9589aab79045b32a810c62723aad9cd12b4dff2fad5f9032841fd1fda574056b2baa5ca68127080b3135bc22a365e4b221eb0dfe387201261819a4dd3b3a4844786e93b8b7659e03c8e52a703662c758be189503ea8c702cf2de144abe84eb92edf1e08d3b0c45679c7983e",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x195c92d119ede1f012de2a74e766c1e3567cac91ef7852074b5983659f61f0d4", "0x14b23f2cb099bcc4b8849ed17b76e03644e11e427d98ccfc40ab361c49deabad"],
				"proof": "0x2e8635be0e045f287ca938f1e0b32c4e1e10e8e06f2877d9484ef4bea5a9b9822267dd677143e323ac424438f24c6288e690c25c13fad3acf7331ca83f747bfe08bad59e92572a9c7cc3591a9520dae96d79efd64d21f1d11f8429dfd6baaa2d2259de3a589fc61080b911a54e349dac41b5cd70b202dbddd726f51bbc66675001109f1ae99ee21b0a643baff6045e1ad5fa26fde899c427c5f38cc71d272adc",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			},
			{
				"card": ["0x2651a79169ed2334b132dec3b05a780034318e4817d85f362f1cef2c7839e96e", "0x24f58f8fd3edf622f8fe895bb5ee77293a076e4bcbd80bb486e6c142ab48b426"],
				"proof": "0x22db5c37e3455112e3182c2478f12b5a510a80937fea1e2547119f8e72c3072d04bb7fcc06bac8bdbb568e1fa624279351b01d0576f09f8eb0e6a63af37f1e8d1715d86d8257feffa3cbc406c813ad9449fff8510fe1f59e67f5a14126278003003fc7ef536394f171f42ddc0049a52e99e0eea06785109e3f5f94af3994c23a001d17287f2fb3433582fcab9d89acd38f0504dfc60a1442207286464e895e2b",
				"public_key": "0x61084c5410e02d9019817eba8bb41b70137f6c519c65f59e9d02bcfcdde95629"
			}
		]
        "##;

        let s1_1 = json!(vec![
            "0x54f387596caeabf85c19c27162cb0ae9fab8f06e",
            "0x54f387596caeabf85c19c27162cb0ae9fab8f06f"
        ]);
        let s1_2 = json!(vec![
            "0x54f387596caeabf85c19c27162cb0ae9fab8f06d",
            "0x54f387596caeabf85c19c27162cb0ae9fab8f06f"
        ]);
        let s1_3 = json!(vec![
            "0x54f387596caeabf85c19c27162cb0ae9fab8f06d",
            "0x54f387596caeabf85c19c27162cb0ae9fab8f06e"
        ]);
        let s2 = serde_json::Value::from_str(&s2).unwrap();

        handler
            .handle(
                peers[0].1,
                "revealResponse",
                DefaultParams(vec![s1_1, vec![s2.clone(), s2.clone()].into()]),
            )
            .await
            .unwrap();

        handler
            .handle(
                peers[1].1,
                "revealResponse",
                DefaultParams(vec![s1_2, vec![s2.clone(), s2.clone()].into()]),
            )
            .await
            .unwrap();

        handler
            .handle(
                peers[2].1,
                "revealResponse",
                DefaultParams(vec![s1_3, vec![s2.clone(), s2.clone()].into()]),
            )
            .await
            .unwrap();

        let _ = handler.online(peers[0].1).await;
    }
}
