use poker_core::{
    combination::ClassicCardCombination,
    play::PlayAction,
    task::{Task0, TaskCommit},
};
use risc0_zkvm::guest::env;

pub fn main() {
    let cycle_0 = env::cycle_count();
    // todo！
    // Utilizing array indexing to represent cards will reduce card serialization and minimize cycles.
    let task: Task0 = env::read();
    println!("read cycle:{}", env::cycle_count() - cycle_0);

    let Task0 {
        room_id,
        players_env,
        players_hand,
    } = task;

    let mut input_hand = players_hand.clone();

    let n_players = players_hand.len();
    let n_round = players_env.len();
    let mut packs = vec![];
    let mut crypto_cards = vec![];
    let mut unmasked_cards = vec![];
    let mut final_winner = 0;
    let mut round_winner = 0;

    for (round_id, round_env) in players_env.iter().enumerate() {
        let mut round_max_cards = ClassicCardCombination::default();
        let mut current_winner = 0;

        if n_round - 1 != round_id {
            assert!(round_env
                .iter()
                .rev()
                .take(n_players - 1)
                .all(|x| x.action == PlayAction::PAAS));
        }

        for (i, player) in round_env.iter().enumerate() {
            let current = (round_winner + i) % n_players;

            let action: u8 = player.action.into();
            let pack = (action as u128)
                + ((round_id as u128) << 8)
                + ((i as u128) << 16)  // turn_id
                + ((room_id as u128) << 24);
            packs.push(pack);

            if player.action == PlayAction::PLAY {
                let play_crypto_cards = player.play_crypto_cards.clone().unwrap().to_vec();
                crypto_cards.extend(play_crypto_cards.clone());
                let play_unmasked_cards = player.play_unmasked_cards.clone().unwrap();
                let play_unmasked_cards_vec = play_unmasked_cards.to_vec();
                unmasked_cards.extend(play_unmasked_cards_vec);
                let classic = play_unmasked_cards.morph_to_classic().unwrap();
                assert!(classic.validate_rules());
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
                assert_eq!(hand_len, remainder_len + play_len);

                if hand.len() == 0 && final_winner == 0 {
                    final_winner = current + 1
                }

                round_max_cards = classic;
                current_winner = current;
            }
        }
        round_winner = current_winner;
    }

    println!("total cycle:{}", env::cycle_count());

    env::commit(&TaskCommit {
        room_id,
        players_hand,
        winner: final_winner,
        crypto_cards,
        unmasked_cards,
        count: env::cycle_count(),
    });
}
