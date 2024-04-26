use ark_serialize::CanonicalSerialize;
use poker_core::{
    cards::{ClassicCard, Suite, Value},
    combination::ClassicCardCombination,
    play::PlayAction,
    task::{Task0, TaskCommit, HAND_NUM, N_PLAYERS},
};
use risc0_zkvm::guest::env;

pub fn main() {
    let cycle_0 = env::cycle_count();
    let task: Task0 = env::read();
    println!("read cycle:{}", env::cycle_count() - cycle_0);

    let Task0 {
        room_id,
        mut first_player,
        mut players_env,
        players_hand,
    } = task;

    let mut input_hand = players_hand.clone();
    let first_player_copy = first_player;

    let n_rounds = players_env.len();
    let mut is_first_play = true;
    let mut packs = vec![];
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

        for (turn_id, player) in round_env.iter_mut().enumerate() {
            let current = (first_player + turn_id) % N_PLAYERS;

            let action: u8 = player.action.into();
            let mut msg = vec![action, round_id as u8, turn_id as u8];
            msg.extend(room_id.to_be_bytes());

            if player.action == PlayAction::PLAY {
                let play_crypto_cards = player.play_crypto_cards.take().unwrap().to_vec();
                let play_unmasked_cards = player.play_unmasked_cards.take().unwrap();
                unmasked_cards.extend(play_unmasked_cards.to_vec());

                let classic = play_unmasked_cards.morph_to_classic().unwrap();
                assert!(classic.check_format());
                assert!(classic > round_max_cards);

                // Check if Heart 3 is played first
                if is_first_play {
                    assert!(classic.contains(&ClassicCard::new(Value::Three, Suite::Heart)));
                    is_first_play = false;
                }

                let hand = input_hand.get_mut(current).unwrap();
                let hand_len_before_remove = hand.len();

                for index in play_crypto_cards.iter() {
                    assert_eq!(current, index / HAND_NUM);
                    let element = &players_hand[current][index % HAND_NUM];

                    if let Some(index) = hand.iter().position(|&x| x == *element) {
                        hand.remove(index);
                    }

                    let mut e1_bytes = vec![];
                    element.0.e1.serialize_compressed(&mut e1_bytes).unwrap();
                    msg.extend(e1_bytes);

                    let mut e2_bytes = vec![];
                    element.0.e2.serialize_compressed(&mut e2_bytes).unwrap();
                    msg.extend(e2_bytes);
                }

                crypto_cards.extend(play_crypto_cards);

                let hand_len_after_remove = hand.len();
                assert_eq!(
                    hand_len_before_remove,
                    hand_len_after_remove + classic.len()
                );

                if hand_len_after_remove == 0 {
                    packs.push(msg);
                    break 'outer;
                }

                round_max_cards = classic;
                round_first_player = current;
            }

            packs.push(msg);
        }

        first_player = round_first_player;
    }

    let remaining_hand: Vec<_> = input_hand.iter().map(|x| x.len()).collect();

    println!("total cycle:{}", env::cycle_count());

    env::commit(&TaskCommit {
        room_id,
        first_player: first_player_copy,
        remaining_hand,
        players_hand,
        crypto_cards,
        unmasked_cards,
    });
}
