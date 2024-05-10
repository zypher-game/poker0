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
use std::{collections::{HashMap, HashSet}, sync::Mutex};
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
    pub connected_players : HashSet<PeerId>,
    pub has_reveal :bool,
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
                connected_players : HashSet::new(),
                has_reveal:false,
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
            let vs: Vec<serde_json::Value> = v
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    let e1 = point_to_hex(&x.0.e1);
                    let e2 = point_to_hex(&x.0.e1);
                    let r = vec![e1.0, e1.1, e2.0, e2.1];

                    let rk: String = r.iter().flat_map(|x| x.chars()).collect();
                    match self.reveal_info.get(&rk) {
                        Some(info) => reveal_info.insert(i.to_string(), info.to_vec().into()),
                        None => reveal_info.insert(i.to_string(), serde_json::Value::Null),
                    };

                    r.into()
                })
                .collect();
            hand.insert(ks, vs.into());
        }

        let player_order: Vec<_> = self.players_order.iter().map(|x| x.to_hex()).collect();
        let mut game_info: Map<String, serde_json::Value> = Map::new();
        game_info.insert("player_order".to_string(), player_order.into());
        game_info.insert("room_id".to_string(), self.room_id.into());
        game_info.insert("round_id".to_string(), self.round_id.into());
        game_info.insert("turn_id".to_string(), self.turn_id.into());
        game_info.insert("first_player".to_string(), self.first_player.into());
        game_info.insert("online_player".to_string(), player.0.to_vec().into());

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
        if self.connected_players.len() == N_PLAYS  && !self.has_reveal{
            for k in self.players_order.iter() {
                let ks = k.to_hex();
                let v = hand.get(&ks).unwrap();
                process_reveal_request(&mut results, *k,v.clone());
            }

            self.has_reveal =true;
        }

        Ok(results)
    }

    /// when player offline, tell other players, then do some change in game UI
    async fn offline(&mut self, player: PeerId) -> Result<HandleResult<Self::Param>> {
        let mut results = HandleResult::default();
        results.add_all("offline", DefaultParams(vec![player.0.to_vec().into()]));
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
                    self.first_player = self.players_order.iter()
                    .position(|x| *x == player).unwrap();
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
                trace.insert("cards".to_string(), classic_index.into());
                trace.insert("player".to_string(), player.0.to_vec().into());
                self.traces.push(trace);

                println!("Finish Handler play");

                process_play_response(&mut results, player, classic.to_bytes());

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
                trace.insert("player".to_string(), player.0.to_vec().into());
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
                println!("Handler revealResponse :{:?}", params);

                // vec![peerId, vec![crypto_card, reveal_card, reveal_proof, public_key]]
                assert!(params.len() == 2);
                println!("Handler revealResponse");

                let peer_id = params[0].as_array().unwrap();
                let peer_id: Vec<String> = peer_id.iter().map(|x| x.as_str().unwrap().to_string()).collect();
                let peer_id = peer_id.iter().map(|x| PeerId::from_hex(x).unwrap() ).collect::<Vec<_>>();

                let reveal_info = params[1].as_array().unwrap();
                for (v,id) in reveal_info.iter().zip(peer_id.iter()) {
                    process_reveal_response(&mut results, *id, v);
                }

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
    play_cards: Vec<u8>,
) {
    results.add_all(
        "play",
        DefaultParams(vec![pid.to_hex().into(), play_cards.into()]),
    );
}

fn process_pass_response(results: &mut HandleResult<DefaultParams>, pid: PeerId) {
    results.add_all("pass", DefaultParams(vec![pid.0.to_vec().into()]));
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
    results.add_one(pid, "revealResponse", DefaultParams(vec![reveal_proof.clone()]));
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
    use z4_engine::{Address, Handler, PeerId};
    use zshuffle::Ciphertext;

    use super::{init_prover_key, PokerHandler};

    // #[test]
    #[tokio::test]
    async fn t() {
       
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
}
