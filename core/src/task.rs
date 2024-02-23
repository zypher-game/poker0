use crate::cards::EncodingCard;
use crate::combination::ClassicCardCombination;
use crate::play::{PlayAction, PlayerEnv, PlayerEnv0};
use crate::{cards::CryptoCard, schnorr::PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub room_id: usize,
    pub num_round: usize,
    pub players_keys: Vec<PublicKey>,
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
    pub num_round: usize,
    pub players_env: Vec<Vec<PlayerEnv0>>,
    pub players_hand: Vec<Vec<CryptoCard>>,
}

impl Task {
    pub fn convert_to_task0(&self) -> Task0 {
        let players_env = self
            .players_env
            .iter()
            .map(|x| x.iter().map(|y| y.convert_to_env0()).collect())
            .collect();

        Task0 {
            room_id: self.room_id,
            num_round: self.num_round,
            players_env,
            players_hand: self.players_hand.clone(),
        }
    }

    pub fn verify(&self) -> TaskCommit {
        let mut input_hand = self.players_hand.clone();

        let Task {
            room_id,
            num_round,
            players_keys,
            players_env,
            players_hand,
        } = self.clone();

        assert_eq!(num_round, players_env.len());

        let n = players_keys.len();
        let mut first_player_id = 0;
        let mut round_max_cards = ClassicCardCombination::default();
        let mut crypto_cards = vec![];
        let mut unmasked_cards = vec![];
        let mut winner = 0;

        for (round_id, round_env) in players_env.iter().enumerate() {
            let mut round_first_player_id = 0;

            assert!(round_env
                .iter()
                .rev()
                .take(n - 1)
                .all(|x| x.action == PlayAction::PAAS));

            for (i, player) in round_env.iter().enumerate() {
                let turn_id = i / n;
                let current = (first_player_id + i) % n;
                let pk = &players_keys[current];

                assert!(player
                    .verify_sign_with_params(&pk, room_id, round_id as u8, turn_id as u8)
                    .is_ok());

                if i == 0 {
                    assert_eq!(player.action, PlayAction::PLAY);
                    let reveals = player.verify_and_get_reveals().unwrap();
                    let encoding = player
                        .play_cards
                        .as_ref()
                        .and_then(|x| Some(x.morph_to_encoding(&reveals)))
                        .unwrap();
                    unmasked_cards.extend(encoding.to_vec());
                    let classic = encoding.morph_to_classic().unwrap();
                    assert!(classic.validate_rules());

                    let play_cards = player.play_cards.clone().unwrap().to_vec();
                    crypto_cards.extend(play_cards.clone());
                    let hand = input_hand.get_mut(current).unwrap();
                    assert!(play_cards.iter().all(|x| hand.contains(x)));
                    hand.retain(|x| !play_cards.contains(x));

                    if hand.len() == 0 && winner == 0 {
                        winner = current + 1
                    }

                    round_max_cards = classic;
                    round_first_player_id = current;
                } else {
                    if let PlayAction::PLAY = player.action {
                        let reveals = player.verify_and_get_reveals().unwrap();
                        let encoding = player
                            .play_cards
                            .as_ref()
                            .and_then(|x| Some(x.morph_to_encoding(&reveals)))
                            .unwrap();
                        unmasked_cards.extend(encoding.to_vec());
                        let classic = encoding.morph_to_classic().unwrap();
                        assert!(classic.validate_rules());
                        assert!(classic > round_max_cards);

                        let play_cards = player.play_cards.clone().unwrap().to_vec();
                        crypto_cards.extend(play_cards.clone());
                        let hand = input_hand.get_mut(current).unwrap();
                        assert!(play_cards.iter().all(|x| hand.contains(x)));
                        hand.retain(|x| !play_cards.contains(x));

                        if hand.len() == 0 && winner == 0 {
                            winner = current + 1
                        }

                        round_max_cards = classic;
                        round_first_player_id = current;
                    }
                }
            }

            first_player_id = round_first_player_id;
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
    use crate::mock_data::mock_task;

    #[test]
    fn test_verify_task() {
        let commit = mock_task().verify();
        assert_eq!(commit.winner, 2);
    }
}
