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
    // The indices stored in `play_cards` refer to the indices of the `crypto_cards` array.
    pub play_cards: Vec<Vec<usize>>,
    pub crypto_cards: Vec<CryptoCard>,
    pub unmasked_cards: Vec<EncodingCard>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task0 {
    pub room_id: usize,
    // The first one to play.
    pub first_player: usize,
    pub players_env: Vec<Vec<PlayerEnv0>>,
}

impl Task {
    pub fn convert0(&self) -> Task0 {
        let players_env = self
            .players_env
            .iter()
            .map(|x| x.iter().map(|y| y.convert0()).collect())
            .collect();

        Task0 {
            room_id: self.room_id,
            first_player: self.first_player,
            players_env,
        }
    }

    pub fn verify(&self) -> TaskCommit {
        let Task {
            room_id,
            mut first_player,
            players_key,
            mut players_env,
            ..
        } = self.clone();

        let n_rounds: usize = players_env.len();
        let mut is_first_play = true;
        let mut crypto_cards = vec![];
        let mut unmasked_cards = vec![];
        let mut play_cards = vec![vec![]; N_PLAYERS];

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
                    crypto_cards.extend(player.play_crypto_cards.take().unwrap().to_vec());
                    unmasked_cards.extend(encoding.to_vec());
                    let classic = encoding.morph_to_classic().unwrap();
                    assert!(classic.check_format());
                    assert!(classic > round_max_cards);

                    // Check if Heart 3 is played first
                    if is_first_play {
                        assert!(classic.contains(&ClassicCard::new(Value::Three, Suite::Heart)));
                        is_first_play = false;
                    }

                    let has_played = play_cards.get_mut(current).unwrap();
                    has_played.extend(crypto_cards.len() - classic.len()..crypto_cards.len());

                    if has_played.len() == HAND_NUM {
                        break 'outer;
                    }

                    round_max_cards = classic;
                    round_first_player = current;
                }
            }

            first_player = round_first_player;
        }

        TaskCommit {
            room_id,
            play_cards,
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
        assert_eq!(
            commit
                .play_cards
                .iter()
                .map(|x| x.len())
                .collect::<Vec<_>>(),
            vec![14, 3, 16]
        );
    }
}
