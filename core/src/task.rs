use crate::cards::{ClassicCard, EncodingCard, Suite, Value};
use crate::combination::ClassicCardCombination;
use crate::play::{PlayAction, PlayerEnv, PlayerEnv0};
use crate::{cards::CryptoCard, schnorr::PublicKey};
use serde::{Deserialize, Serialize};

pub const N_PLAYERS: usize = 3;
pub const HAND_NUM: usize = 16;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub room_id: usize,
    // The first one to play.
    pub first_player: usize,
    pub players_key: Vec<PublicKey>,
    pub players_env: Vec<Vec<PlayerEnv>>,
    pub players_hand: Vec<Vec<CryptoCard>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskCommit {
    pub room_id: usize,
    pub first_player: usize,
    pub remaining_hand: Vec<usize>,
    pub players_hand: Vec<Vec<CryptoCard>>,
    // The indices stored in `crypto_cards` refer to the indices of the `players_hand` array.
    pub crypto_cards: Vec<usize>,
    pub unmasked_cards: Vec<EncodingCard>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task0 {
    pub room_id: usize,
    // The first one to play.
    pub first_player: usize,
    pub players_env: Vec<Vec<PlayerEnv0>>,
    pub players_hand: Vec<Vec<CryptoCard>>,
}

impl Task {
    pub fn convert0(&self) -> Task0 {
        let hand = self
            .players_hand
            .clone()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        let players_env = self
            .players_env
            .iter()
            .map(|x| x.iter().map(|y| y.convert0(&hand)).collect())
            .collect();

        Task0 {
            room_id: self.room_id,
            first_player: self.first_player,
            players_env,
            players_hand: self.players_hand.clone(),
        }
    }

    pub fn verify(&self) -> TaskCommit {
        let mut input_hand = self.players_hand.clone();
        let first_player_copy = self.first_player;

        let Task {
            room_id,
            mut first_player,
            players_key,
            mut players_env,
            players_hand,
        } = self.clone();

        let n_rounds: usize = players_env.len();
        let mut is_first_play = true;
        let mut crypto_cards = vec![];
        let mut unmasked_cards = vec![];

        'outer: for (round_id, round_env) in players_env.iter_mut().enumerate() {
            let mut round_max_cards = ClassicCardCombination::default();
            let mut round_first_player = 0;

            if n_rounds - 1 != round_id {
                assert!(round_env
                    .iter()
                    .rev()
                    .take(N_PLAYERS - 1)
                    .all(|x| x.action == PlayAction::PAAS));
            }

            for (i, player) in round_env.iter_mut().enumerate() {
                let current = (first_player + i) % N_PLAYERS;

                assert!(player
                    .verify_sign_with_params(
                        &players_key[current],
                        room_id,
                        round_id as u8,
                        i as u8
                    )
                    .is_ok());

                if player.action == PlayAction::PLAY {
                    let reveals = player.verify_and_get_reveals().unwrap();
                    let encoding = player
                        .play_crypto_cards
                        .as_ref()
                        .and_then(|x| Some(x.morph_to_encoding(&reveals)))
                        .unwrap();
                    unmasked_cards.extend(encoding.to_vec());
                    let classic = encoding.morph_to_classic().unwrap();
                    assert!(classic.check_format());
                    assert!(classic > round_max_cards);

                    // Check if Heart 3 is played first
                    if is_first_play {
                        assert!(classic.contains(&ClassicCard::new(Value::Three, Suite::Heart)));
                        is_first_play = false;
                    }

                    let play_cards = player.play_crypto_cards.take().unwrap().to_vec();
                    let hand = input_hand.get_mut(current).unwrap();
                    assert!(play_cards.iter().all(|x| hand.contains(x)));
                    hand.retain(|x| !play_cards.contains(x));

                    crypto_cards.extend(play_cards.iter().map(|x| {
                        players_hand[current].iter().position(|y| x == y).unwrap()
                            + HAND_NUM * current
                    }));

                    if hand.len() == 0 {
                        break 'outer;
                    }

                    round_max_cards = classic;
                    round_first_player = current;
                }
            }

            first_player = round_first_player;
        }

        let remaining_hand: Vec<_> = input_hand.iter().map(|x| x.len()).collect();

        TaskCommit {
            room_id,
            first_player: first_player_copy,
            remaining_hand,
            players_hand,
            crypto_cards,
            unmasked_cards,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::mock_data::task::mock_task;

    #[test]
    fn test_verify_task() {
        let commit = mock_task().verify();
        assert_eq!(commit.remaining_hand, vec![2, 13, 0]);
        assert_eq!(
            commit.crypto_cards,
            vec![
                16, 22, 39, 40, 11, 14, 4, 5, 1, 13, 10, 8, 3, 7, 15, 9, 32, 37, 46, 41, 44, 45,
                47, 34, 38, 0, 23, 35, 36, 2, 42, 33, 43
            ]
        );
        assert_eq!(commit.first_player, 1);
    }
}
