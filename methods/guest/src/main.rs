use poker_core::{
    combination::ClassicCardCombination,
    play::PlayAction,
    task::{Task0, TaskCommit},
};
use risc0_zkvm::guest::env;

pub fn main() {
    let cycle_0 = env::cycle_count();
    let task: Task0 = env::read();
    println!("read cycle:{}", env::cycle_count() - cycle_0);

    let Task0 {
        room_id,
        num_round,
        players_env,
        players_hand,
    } = task;

    let mut input_hand = players_hand.clone();

    assert_eq!(num_round, players_env.len());

    let n = players_hand.len();
    let mut first_player_id = 0;
    let mut round_max_cards = ClassicCardCombination::default();
    let mut packs = vec![];
    let mut crypto_cards = vec![];
    let mut unmasked_cards = vec![];
    let mut winner = 0;

    for (round_id, round_env) in players_env.iter().enumerate() {
        let mut round_player_id = 0;

        println!("---------round {}:{}", round_id, env::cycle_count());

        assert!(round_env
            .iter()
            .rev()
            .take(n - 1)
            .all(|x| x.action == PlayAction::PAAS)); // todo the last

        for (i, player) in round_env.iter().enumerate() {
            let turn_id = i / n;
            let current = (first_player_id + i) % n;

            let action: u8 = player.action.into();
            let pack = (action as u128)
                + ((round_id as u128) << 8)
                + ((turn_id as u128) << 16)
                + ((room_id as u128) << 24);
            packs.push(pack);

            if i == 0 {
                let cycle_1 = env::cycle_count();

                assert_eq!(player.action, PlayAction::PLAY);

                let play_crypto_cards = player.play_crypto_cards.clone().unwrap().to_vec();
                crypto_cards.extend(play_crypto_cards.clone());
                let play_unmasked_cards = player.play_unmasked_cards.clone().unwrap();
                let play_unmasked_cards_vec = play_unmasked_cards.to_vec();
                unmasked_cards.extend(play_unmasked_cards_vec);
                let classic = play_unmasked_cards.morph_to_classic().unwrap();
                assert!(classic.sanity_check());

                let hand = input_hand.get_mut(current).unwrap();
                let hand_len = hand.len();
                let play_len = classic.len();
                for element in play_crypto_cards {
                    if let Some(index) = hand.iter().position(|&x| x == element) {
                        hand.remove(index);
                    }
                }
                let remainder_len = hand.len();
                assert_eq!(hand_len - play_len, remainder_len);

                if hand.len() == 0 && winner == 0 {
                    winner = current + 1
                }

                println!("inner i:{} cycle:{}", i, env::cycle_count() - cycle_1);

                round_max_cards = classic;
                round_player_id = current;
            } else {
                let cycle_1 = env::cycle_count();

                if let PlayAction::PLAY = player.action {
                    let play_crypto_cards = player.play_crypto_cards.clone().unwrap().to_vec();
                    crypto_cards.extend(play_crypto_cards.clone());
                    let play_unmasked_cards = player.play_unmasked_cards.clone().unwrap();
                    let play_unmasked_cards_vec = play_unmasked_cards.to_vec();
                    unmasked_cards.extend(play_unmasked_cards_vec);
                    let classic = play_unmasked_cards.morph_to_classic().unwrap();
                    assert!(classic.sanity_check());
                    assert!(classic > round_max_cards);

                    let hand = input_hand.get_mut(current).unwrap();
                    let hand_len = hand.len();
                    let play_len = classic.len();
                    for element in play_crypto_cards {
                        if let Some(index) = hand.iter().position(|&x| x == element) {
                            hand.remove(index);
                        }
                    }
                    let remainder_len = hand.len();
                    assert_eq!(hand_len - play_len, remainder_len);

                    if hand.len() == 0 && winner == 0 {
                        winner = current + 1
                    }

                    round_max_cards = classic;
                    round_player_id = current;
                }

                println!("inner i:{} cycle:{}", i, env::cycle_count() - cycle_1);
            }
        }

        first_player_id = round_player_id;
    }

    println!("total cycle:{}", env::cycle_count());

    env::commit(&TaskCommit {
        room_id,
        players_hand,
        winner,
        crypto_cards,
        unmasked_cards
    });
}
