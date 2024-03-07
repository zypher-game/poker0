use std::{
    collections::{hash_map, HashMap},
    io::Bytes,
};
use rand_chacha::{ChaChaRng,rand_core::SeedableRng};
use poker_bonsai::{stark::prove_bonsai,snark::stark_to_snark};
use poker_core::{
    cards::CryptoCard,
    play::{self, PlayAction, PlayerEnv},
    schnorr::PublicKey,
    task::Task,
};
use poker_gadgets::{
    build_cs::{prove_outsource, verify_outsource, N_CARDS, N_PLAYS},
    create_and_rescale_outsource,
    gen_params::PROVER_PARAMS,
};
use z4_engine::{
    Address, DefaultParams, Error, HandleResult, Handler, PeerId, Result, RoomId, Tasks,
};

pub struct PokerHandler {
    room_id: RoomId,
    accounts: HashMap<PeerId, PublicKey>,
    players_hand: HashMap<PeerId, Vec<CryptoCard>>,
    players_order: Vec<PeerId>,
    // round_id => (turn_id => PlayerEnv)
    players_envs: HashMap<u8, HashMap<u8, PlayerEnv>>,
}

// todo
// remove num_round in task
// rename playaction in wasm
// task's romm_id type
// task turn_id order

impl PokerHandler {
    fn prove(&self) {
        let players_keys = self
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

        let task = Task {
            room_id: self.room_id as usize,
            num_round: 1, // todo 
            players_keys,
            players_env,
            players_hand,
        };

        {
            let task0 = task.convert_to_task0();
            let (receipt, session_id) = prove_bonsai(&task0).unwrap();

            let snark_proof = stark_to_snark(session_id).unwrap();
        }

        {
            let mut rng = ChaChaRng::from_entropy(); 
            let (players_keys, reveal_outsources, unmask_outsources) =
                create_and_rescale_outsource(&task, N_PLAYS, N_CARDS);

            let proof = prove_outsource(
                &mut rng,
                &players_keys,
                &reveal_outsources,
                &unmask_outsources,
                &PROVER_PARAMS,
            )
            .unwrap();
        }
    }
}

#[async_trait::async_trait]
impl Handler for PokerHandler {
    type Param = DefaultParams;

    async fn create(peers: &[(Address, PeerId)]) -> (Self, Tasks<Self>) {
        let accounts = peers
            .iter()
            .map(|(_, pid)| (*pid, PublicKey::default()))
            .collect();

        (
            Self {
                room_id: 1, // todo 
                accounts,
                players_hand: HashMap::default(),
                players_order: vec![],
                players_envs: HashMap::new(),
            },
            Default::default(),
        )
    }

    // async fn online(&mut self, peer: PeerId) -> Result<HandleResult<Self::Param>> {
    //     todo!()
    // }

    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        params: DefaultParams,
    ) -> Result<HandleResult<Self::Param>> {
        let public_key = self.accounts.get(&player).ok_or(Error::NoPlayer)?;
        let mut params = params.0;

        if let Some(velue) = params.pop() {
            let btyes = velue.as_str().unwrap();
            let play_env: PlayerEnv = serde_json::from_str(btyes).map_err(|_| Error::Params)?;
            assert!(play_env.verify_sign(public_key).is_ok());

            let round_info = self
                .players_envs
                .entry(play_env.round_id)
                .or_insert(HashMap::new());
            round_info.insert(play_env.turn_id, play_env.clone());

            match method {
                "play" => {
                    assert_eq!(play_env.action, PlayAction::PLAY);
                    let play_crypto_cards = play_env.play_cards.unwrap().to_vec();
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

                    // todo check card rule
                }

                "pass" => {
                    assert_eq!(play_env.action, PlayAction::PAAS);
                }

                _ => unimplemented!(),
            }
        }

        Err(Error::Params)
    }
}

#[cfg(test)]
mod t {
    #[test]
    fn t() {}
}
