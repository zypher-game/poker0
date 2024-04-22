use crate::cards::{ClassicCard, EncodingCard, Suite, Value};
use crate::combination::ClassicCardCombination;
use crate::play::{PlayAction, PlayerEnv, PlayerEnv0};
use crate::{cards::CryptoCard, schnorr::PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub room_id: usize,
    pub first_player: usize,
    pub players_key: Vec<PublicKey>,
    pub players_env: Vec<Vec<PlayerEnv>>,
    pub players_hand: Vec<Vec<CryptoCard>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskCommit {
    pub room_id: usize,
    pub players_hand: Vec<Vec<CryptoCard>>,
    pub winner: usize,
    pub crypto_cards: Vec<CryptoCard>,
    pub unmasked_cards: Vec<EncodingCard>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task0 {
    pub room_id: usize,
    pub players_env: Vec<Vec<PlayerEnv0>>,
    pub players_hand: Vec<Vec<CryptoCard>>,
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
            players_env,
            players_hand: self.players_hand.clone(),
        }
    }

    pub fn verify(&self) -> TaskCommit {
        let mut input_hand = self.players_hand.clone();

        let Task {
            room_id,
            first_player,
            players_key,
            players_env,
            players_hand,
        } = self.clone();

        let n_player = players_key.len();
        let n_round: usize = players_env.len();
        let mut round_first_player = first_player;
        let mut check_first_play = true;
        let mut crypto_cards = vec![];
        let mut unmasked_cards = vec![];
        let mut winner = 0;

        for (round_id, round_env) in players_env.iter().enumerate() {
            let mut round_max_cards = ClassicCardCombination::default();
            let mut current_first_player = 0;

            if n_round - 1 != round_id {
                assert!(round_env
                    .iter()
                    .rev()
                    .take(n_player - 1)
                    .all(|x| x.action == PlayAction::PAAS));
            }

            for (i, player) in round_env.iter().enumerate() {
                let current = (round_first_player + i) % n_player;

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

                    // Check if Heart 3 is played first?
                    if check_first_play {
                        assert!(classic.contains(&ClassicCard::new(Value::Three, Suite::Heart)));
                        check_first_play = false;
                    }

                    let play_cards = player.play_crypto_cards.clone().unwrap().to_vec();
                    crypto_cards.extend(play_cards.clone());
                    let hand = input_hand.get_mut(current).unwrap();
                    assert!(play_cards.iter().all(|x| hand.contains(x)));
                    hand.retain(|x| !play_cards.contains(x));

                    if hand.len() == 0 && winner == 0 {
                        winner = current + 1
                    }

                    round_max_cards = classic;
                    current_first_player = current;
                }
            }

            round_first_player = current_first_player;
        }

        TaskCommit {
            room_id,
            players_hand,
            winner,
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
        assert_eq!(commit.winner, 3);
    }
}
