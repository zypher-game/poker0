use ark_ed_on_bn254::{EdwardsAffine, EdwardsProjective, Fr};
use ark_ff::{BigInteger, One, PrimeField, UniformRand};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use poker_bonsai::{snark::stark_to_snark, stark::prove_bonsai};
use poker_core::{
    cards::{CryptoCard, ENCODING_CARDS_MAPPING},
    play::{PlayAction, PlayerEnv},
    schnorr::PublicKey,
    task::Task as CoreTask,
    CiphertextAffine,
};
use poker_gadgets::{
    build_cs::{prove_outsource, verify_outsource, N_CARDS, N_PLAYS},
    create_and_rescale_outsource,
    gen_params::PROVER_PARAMS,
};
use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
use std::collections::HashMap;
use z4_engine::{
    Address, DefaultParams, Error, HandleResult, Handler, PeerId, Result, RoomId, Task, Tasks,
};
use zshuffle::{
    build_cs::{prove_shuffle, verify_shuffle},
    gen_params::{
        gen_shuffle_prover_params, get_shuffle_verifier_params, refresh_prover_params_public_key,
    },
    keygen::aggregate_keys,
    mask::*,
    Ciphertext,
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

        let task = CoreTask {
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

    async fn accept(peers: &[(Address, PeerId, [u8; 32])]) -> Vec<u8> {
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
            let (masked_card, masked_proof) = mask(&mut rng, &joint_pk, &card, &Fr::one()).unwrap();
            verify_mask(&joint_pk, &card, &masked_card, &masked_proof).unwrap();

            deck.push(masked_card)
        }

        assert_eq!(deck.len(), N_CARDS);

        let mut prover_params = gen_shuffle_prover_params(N_CARDS).unwrap();
        refresh_prover_params_public_key(&mut prover_params, &joint_pk).unwrap();

        let (proof, shuffle_deck) =
            prove_shuffle(&mut rng, &joint_pk, &deck, &prover_params).unwrap();

        {
            let mut verifier_params = get_shuffle_verifier_params(N_CARDS).unwrap();
            verifier_params.verifier_params = prover_params.prover_params.verifier_params.clone();
            verify_shuffle(&verifier_params, &deck, &shuffle_deck, &proof).unwrap();
        }

        let mut bytes = vec![];
        for c in shuffle_deck.iter() {
            let mut e1 = vec![];
            c.e1.serialize_compressed(&mut e1).unwrap();
            bytes.extend(e1);
            let mut e2 = vec![];
            c.e2.serialize_compressed(&mut e2).unwrap();
            bytes.extend(e2);
        }

        bytes
    }

    async fn create(
        peers: &[(Address, PeerId, [u8; 32])],
        shuffle_decks: Vec<u8>,
        room_id: RoomId,
    ) -> (Self, Tasks<Self>) {
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

        let player0_hand = deck[0..17]
            .iter()
            .map(|x| CryptoCard(CiphertextAffine::from(*x)))
            .collect();
        let player1_hand = deck[17..34]
            .iter()
            .map(|x| CryptoCard(CiphertextAffine::from(*x)))
            .collect();
        let player2_hand = deck[34..]
            .iter()
            .map(|x| CryptoCard(CiphertextAffine::from(*x)))
            .collect();

        let mut players_hand = HashMap::new();
        players_hand.insert(peers[0].1, player0_hand);
        players_hand.insert(peers[1].1, player1_hand);
        players_hand.insert(peers[2].1, player2_hand);

        (
            Self {
                room_id,
                accounts,
                players_hand,
                players_order,
                players_envs: HashMap::new(),
            },
            Default::default(),
        )
    }

    async fn handle(
        &mut self,
        player: PeerId,
        method: &str,
        params: DefaultParams,
    ) -> Result<HandleResult<Self::Param>> {
        let public_key = self.accounts.get(&player).ok_or(Error::NoPlayer)?;
        let params = params.0;

        let mut results = HandleResult::default();

        match method {
            "play" => {
                assert_eq!(params.len(), 1);
                let btyes = params[0].as_str().unwrap();
                let play_env: PlayerEnv = serde_json::from_str(btyes).map_err(|_| Error::Params)?;
                assert!(play_env.verify_sign(public_key).is_ok());

                let round_info = self
                    .players_envs
                    .entry(play_env.round_id)
                    .or_insert(HashMap::new());
                round_info
                    .entry(play_env.turn_id)
                    .or_insert(play_env.clone());

                assert_eq!(play_env.action, PlayAction::PLAY);
                let play_crypto_cards = play_env.play_crypto_cards.unwrap().to_vec();
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

                let classic = play_env.play_classic_cards.unwrap().to_bytes();
                process_play_response(&mut results, player, classic);
            }

            "pass" => {
                assert_eq!(params.len(), 1);
                let btyes = params[0].as_str().unwrap();
                let play_env: PlayerEnv = serde_json::from_str(btyes).map_err(|_| Error::Params)?;
                assert!(play_env.verify_sign(public_key).is_ok());

                let round_info = self
                    .players_envs
                    .entry(play_env.round_id)
                    .or_insert(HashMap::new());
                round_info
                    .entry(play_env.turn_id)
                    .or_insert(play_env.clone());

                assert_eq!(play_env.action, PlayAction::PAAS);

                process_pass_response(&mut results, pid);
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

#[cfg(test)]
mod test {
    use ark_ec::CurveGroup;
    use ark_ed_on_bn254::EdwardsProjective;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use poker_core::{cards::ENCODING_CARDS_MAPPING, schnorr::KeyPair};
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
    use z4_engine::{Address, Handler, PeerId};
    use zshuffle::{
        reveal::{reveal, unmask, verify_reveal},
        Ciphertext,
    };

    use super::PokerHandler;

    #[test]
    fn t() {}

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

        let deck = PokerHandler::accept(&peers).await;

        let mut last_deck = vec![];
        for x in deck.chunks(64) {
            let e1 = EdwardsProjective::deserialize_compressed(&x[..32]).unwrap();
            let e2 = EdwardsProjective::deserialize_compressed(&x[32..]).unwrap();

            last_deck.push(Ciphertext::new(e1, e2));
        }

        let alice_z: zshuffle::keygen::Keypair = keypair_a.clone().into();
        let bob_z: zshuffle::keygen::Keypair = keypair_b.clone().into();
        let charlie_z: zshuffle::keygen::Keypair = keypair_c.clone().into();

        for card in last_deck.iter() {
            let (reveal_card_a, reveal_proof_a) = reveal(&mut rng, &alice_z, card).unwrap();
            let (reveal_card_b, reveal_proof_b) = reveal(&mut rng, &bob_z, card).unwrap();
            let (reveal_card_c, reveal_proof_c) = reveal(&mut rng, &charlie_z, card).unwrap();

            verify_reveal(&alice_z.public, &card, &reveal_card_a, &reveal_proof_a).unwrap();
            verify_reveal(&bob_z.public, &card, &reveal_card_b, &reveal_proof_b).unwrap();
            verify_reveal(&charlie_z.public, &card, &reveal_card_c, &reveal_proof_c).unwrap();

            let reveals = vec![reveal_card_a, reveal_card_b, reveal_card_c];
            let unmask = unmask(&card, &reveals).unwrap();
            let classic = ENCODING_CARDS_MAPPING.get(&unmask.into_affine()).unwrap();
            println!("{:?}", classic);
        }
    }
}
