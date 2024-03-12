use crate::{
    cards::{ClassicCard, CryptoCard, EncodingCard, Value, ENCODING_CARDS_MAPPING},
    combination::Combination::*,
    errors::{PokerError, Result},
};
use ark_ec::AffineRepr;
use ark_std::iterable::Iterable;
use serde::{Deserialize, Serialize};

/// Different card play combinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Combination<T> {
    // Default
    DefaultCombination,

    // Single card
    Single(T),

    // Pair of cards
    Pair(T, T),

    // Three cards of the same rank
    ThreeOfAKind(T, T, T),

    // Three cards of the same rank with one single card
    ThreeWithOne(T, T, T, T),

    // Three cards of the same rank with one pair
    ThreeWithPair(T, T, T, T, T),

    // Five or more consecutive single cards
    Straight(Vec<T>),

    // Three or more consecutive pairs
    DoubleStraight(Vec<(T, T)>),

    // Two or more consecutive three of a kind
    TripleStraight(Vec<(T, T, T)>),

    // Triple straight with one single card
    TripleStraightWithOne(Vec<(T, T, T, T)>),

    // Triple straight with one pair
    TripleStraightWithPair(Vec<(T, T, T, T, T)>),

    // Four cards of the same rank with two single cards
    FourWithTwoSingle(T, T, T, T, T, T),

    // Four cards of the same rank with two pairs
    FourWithTwoPairs(T, T, T, T, T, T, T, T),

    // Four cards of the same rank
    Bomb(T, T, T, T),

    // Both Jokers in a standard deck
    Rocket(T, T),
}

impl<T: Clone + Copy> Combination<T> {
    #[inline]
    pub fn weight(&self) -> u8 {
        match self {
            DefaultCombination => 0,
            Single(_) => 1,
            Pair(_, _) => 1,
            ThreeOfAKind(_, _, _) => 1,
            ThreeWithOne(_, _, _, _) => 1,
            ThreeWithPair(_, _, _, _, _) => 1,
            Straight(_) => 1,
            DoubleStraight(_) => 1,
            TripleStraight(_) => 1,
            TripleStraightWithOne(_) => 1,
            TripleStraightWithPair(_) => 1,
            FourWithTwoSingle(_, _, _, _, _, _) => 1,
            FourWithTwoPairs(_, _, _, _, _, _, _, _) => 1,
            Bomb(_, _, _, _) => 2,
            Rocket(_, _) => 3,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            DefaultCombination => 0,
            Single(_) => 1,
            Pair(_, _) => 2,
            ThreeOfAKind(_, _, _) => 3,
            ThreeWithOne(_, _, _, _) => 4,
            ThreeWithPair(_, _, _, _, _) => 5,
            Straight(x) => x.len(),
            DoubleStraight(x) => 2 * x.len(),
            TripleStraight(x) => 3 * x.len(),
            TripleStraightWithOne(x) => 4 * x.len(),
            TripleStraightWithPair(x) => 5 * x.len(),
            FourWithTwoSingle(_, _, _, _, _, _) => 6,
            FourWithTwoPairs(_, _, _, _, _, _, _, _) => 8,
            Bomb(_, _, _, _) => 4,
            Rocket(_, _) => 2,
        }
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<T> {
        match self {
            DefaultCombination => vec![],
            Single(x) => vec![*x],
            Pair(x1, x2) => vec![*x1, *x2],
            ThreeOfAKind(x1, x2, x3) => vec![*x1, *x2, *x3],
            ThreeWithOne(x1, x2, x3, x4) => vec![*x1, *x2, *x3, *x4],
            ThreeWithPair(x1, x2, x3, x4, x5) => vec![*x1, *x2, *x3, *x4, *x5],
            Straight(x) => x.to_vec(),
            DoubleStraight(x) => x.iter().flat_map(|(x1, x2)| vec![*x1, *x2]).collect(),
            TripleStraight(x) => x
                .iter()
                .flat_map(|(x1, x2, x3)| vec![*x1, *x2, *x3])
                .collect(),
            TripleStraightWithOne(x) => x
                .iter()
                .flat_map(|(x1, x2, x3, x4)| vec![*x1, *x2, *x3, *x4])
                .collect(),
            TripleStraightWithPair(x) => x
                .iter()
                .flat_map(|(x1, x2, x3, x4, x5)| vec![*x1, *x2, *x3, *x4, *x5])
                .collect(),
            FourWithTwoSingle(x1, x2, x3, x4, x5, x6) => vec![*x1, *x2, *x3, *x4, *x5, *x6],
            FourWithTwoPairs(x1, x2, x3, x4, x5, x6, x7, x8) => {
                vec![*x1, *x2, *x3, *x4, *x5, *x6, *x7, *x8]
            }
            Bomb(x1, x2, x3, x4) => vec![*x1, *x2, *x3, *x4],
            Rocket(x1, x2) => vec![*x1, *x2],
        }
    }
}

pub type ClassicCardCombination = Combination<ClassicCard>;

impl Default for ClassicCardCombination {
    #[inline]
    fn default() -> Self {
        Self::DefaultCombination
    }
}

impl PartialEq for ClassicCardCombination {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DefaultCombination, DefaultCombination) => true,
            (Single(x), Single(y)) => x.get_value().eq(&&y.get_value()),

            (Pair(x, _), Pair(y, _)) => x.get_value().eq(&y.get_value()),

            (ThreeOfAKind(x, _, _), ThreeOfAKind(y, _, _)) => x.get_value().eq(&&y.get_value()),

            (ThreeWithOne(x, _, _, _), ThreeWithOne(y, _, _, _)) => {
                x.get_value().eq(&&y.get_value())
            }

            (ThreeWithPair(x, _, _, _, _), ThreeWithPair(y, _, _, _, _)) => {
                x.get_value().eq(&&y.get_value())
            }

            (Straight(x), Straight(y)) => {
                assert_eq!(x.len(), y.len()); // todo if x.len() ！= y.len() return false
                x.last()
                    .unwrap()
                    .get_value()
                    .eq(&&&y.last().unwrap().get_value())
            }

            (DoubleStraight(x), DoubleStraight(y)) => {
                assert_eq!(x.len(), y.len());
                x.last()
                    .unwrap()
                    .0
                    .get_value()
                    .eq(&y.last().unwrap().0.get_value())
            }

            (TripleStraight(x), TripleStraight(y)) => {
                assert_eq!(x.len(), y.len());
                x.last()
                    .unwrap()
                    .0
                    .get_value()
                    .eq(&y.last().unwrap().0.get_value())
            }

            (TripleStraightWithOne(x), TripleStraightWithOne(y)) => {
                assert_eq!(x.len(), y.len());
                x.last()
                    .unwrap()
                    .0
                    .get_value()
                    .eq(&y.last().unwrap().0.get_value())
            }

            (TripleStraightWithPair(x), TripleStraightWithPair(y)) => {
                assert_eq!(x.len(), y.len());
                x.last()
                    .unwrap()
                    .0
                    .get_value()
                    .eq(&y.last().unwrap().0.get_value())
            }

            (FourWithTwoSingle(x, _, _, _, _, _), FourWithTwoSingle(y, _, _, _, _, _)) => {
                x.get_value().eq(&&y.get_value())
            }

            (
                FourWithTwoPairs(x, _, _, _, _, _, _, _),
                FourWithTwoPairs(y, _, _, _, _, _, _, _),
            ) => x.get_value().eq(&y.get_value()),

            (Bomb(x, _, _, _), Bomb(y, _, _, _)) => x.get_value().eq(&y.get_value()),
            // (
            //     Rocket(x, _ ),
            //     Rocket(y, _),
            // ) => x.eq(y),
            _ => false,
        }
    }
}

impl PartialOrd for ClassicCardCombination {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.weight() == other.weight() {
            match (self, other) {
                (Single(x), Single(y)) => x.weight().partial_cmp(&y.weight()),

                (Pair(x, _), Pair(y, _)) => x.weight().partial_cmp(&y.weight()),

                (ThreeOfAKind(x, _, _), ThreeOfAKind(y, _, _)) => {
                    x.weight().partial_cmp(&y.weight())
                }

                (ThreeWithOne(x, _, _, _), ThreeWithOne(y, _, _, _)) => {
                    x.weight().partial_cmp(&y.weight())
                }

                (ThreeWithPair(x, _, _, _, _), ThreeWithPair(y, _, _, _, _)) => {
                    x.weight().partial_cmp(&y.weight())
                }

                (Straight(x), Straight(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .weight()
                        .partial_cmp(&y.last().unwrap().weight())
                }

                (DoubleStraight(x), DoubleStraight(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .0
                        .weight()
                        .partial_cmp(&y.last().unwrap().0.weight())
                }

                (TripleStraight(x), TripleStraight(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .0
                        .weight()
                        .partial_cmp(&y.last().unwrap().0.weight())
                }

                (TripleStraightWithOne(x), TripleStraightWithOne(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .0
                        .weight()
                        .partial_cmp(&y.last().unwrap().0.weight())
                }

                (TripleStraightWithPair(x), TripleStraightWithPair(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .0
                        .weight()
                        .partial_cmp(&y.last().unwrap().0.weight())
                }

                (FourWithTwoSingle(x, _, _, _, _, _), FourWithTwoSingle(y, _, _, _, _, _)) => {
                    x.weight().partial_cmp(&y.weight())
                }

                (
                    FourWithTwoPairs(x, _, _, _, _, _, _, _),
                    FourWithTwoPairs(y, _, _, _, _, _, _, _),
                ) => x.weight().partial_cmp(&y.weight()),

                (Bomb(x, _, _, _), Bomb(y, _, _, _)) => x.weight().partial_cmp(&y.weight()),
                //  (Rocket(_, _), Rocket(_, _)) => todo!(),
                _ => unimplemented!(),
            }
        } else {
            self.weight().partial_cmp(&other.weight())
        }
    }
}

impl ClassicCardCombination {
    #[inline]
    pub fn validate_rules(&self) -> bool {
        match self {
            DefaultCombination => false,

            Single(_) => true,

            Pair(x1, x2) => x1.get_value() == x2.get_value(),

            ThreeOfAKind(x1, x2, x3) => {
                x1.get_value() == x2.get_value() && x1.get_value() == x3.get_value()
            }

            ThreeWithOne(x1, x2, x3, x4) => {
                x1.get_value() == x2.get_value()
                    && x1.get_value() == x3.get_value()
                    && x1.get_value() != x4.get_value()
            }

            ThreeWithPair(x1, x2, x3, y1, y2) => {
                let condition1 = ThreeOfAKind(*x1, *x2, *x3).validate_rules();
                let condition2 = Pair(*y1, *y2).validate_rules();

                condition1 && condition2
            }

            Straight(x) => {
                if x.len() < 5 {
                    return false;
                }
                let last_card = x.last().unwrap();
                if last_card.weight() > Value::Ace.weight() {
                    return false;
                }

                x.windows(2).all(|x| x[1].weight() == x[0].weight() + 1)
            }

            DoubleStraight(x) => {
                let condition1 = x.iter().all(|(t1, t2)| t1.get_value() == t2.get_value());

                let stright = x.iter().map(|x| x.0).collect::<Vec<_>>();
                if stright.len() < 3 {
                    return false;
                }
                let last_card = stright.last().unwrap();
                if last_card.weight() > Value::Ace.weight() {
                    return false;
                }

                let condition2 = stright
                    .windows(2)
                    .all(|y| y[1].weight() == y[0].weight() + 1);

                condition1 && condition2
            }

            TripleStraight(x) => {
                let condition1 = x.iter().all(|(t1, t2, t3)| {
                    t1.get_value() == t2.get_value() && t2.get_value() == t3.get_value()
                });

                let stright = x.iter().map(|x| x.0).collect::<Vec<_>>();
                if stright.len() < 2 {
                    return false;
                }
                let last_card = stright.last().unwrap();
                if last_card.weight() > Value::Ace.weight() {
                    return false;
                }

                let condition2 = stright
                    .windows(2)
                    .all(|y| y[1].weight() == y[0].weight() + 1);

                condition1 && condition2
            }

            TripleStraightWithOne(x) => {
                let triple_stright = x.iter().map(|x| (x.0, x.1, x.2)).collect::<Vec<_>>();
                TripleStraight(triple_stright).validate_rules()
            }

            TripleStraightWithPair(x) => {
                let triple_stright = x.iter().map(|x| (x.0, x.1, x.2)).collect::<Vec<_>>();
                let condition1 = TripleStraight(triple_stright).validate_rules();
                let condition2 = x.iter().all(|x| x.3.get_value() == x.4.get_value());

                condition1 && condition2
            }

            FourWithTwoSingle(x1, x2, x3, x4, _, _) => {
                x1.get_value() == x2.get_value()
                    && x2.get_value() == x3.get_value()
                    && x3.get_value() == x4.get_value()
            }

            FourWithTwoPairs(x1, x2, x3, x4, y1, y2, y3, y4) => {
                // todo Joker is a single or a pair
                let condition1 = x1.get_value() == x2.get_value()
                    && x2.get_value() == x3.get_value()
                    && x3.get_value() == x4.get_value();
                let condition2 = y1.get_value() == y2.get_value();
                let condition3 = y3.get_value() == y4.get_value();

                condition1 && condition2 && condition3
            }

            Bomb(x1, x2, x3, x4) => {
                x1.get_value() == x2.get_value()
                    && x2.get_value() == x3.get_value()
                    && x3.get_value() == x4.get_value()
            }

            Rocket(_, _) => todo!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            DefaultCombination => vec![],
            Single(x) => x.to_bytes(),
            Pair(x1, x2) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes
            }
            ThreeOfAKind(x1, x2, x3) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes.extend(x3.to_bytes());
                bytes
            }
            ThreeWithOne(x1, x2, x3, x4) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes.extend(x3.to_bytes());
                bytes.extend(x4.to_bytes());
                bytes
            }
            ThreeWithPair(x1, x2, x3, x4, x5) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes.extend(x3.to_bytes());
                bytes.extend(x4.to_bytes());
                bytes.extend(x5.to_bytes());
                bytes
            }
            Straight(x) => {
                let mut bytes = vec![];
                for y in x.iter() {
                    bytes.extend(y.to_bytes());
                }
                bytes
            }
            DoubleStraight(x) => {
                let mut bytes = vec![];
                for y in x.iter() {
                    bytes.extend(y.0.to_bytes());
                    bytes.extend(y.1.to_bytes());
                }
                bytes
            }
            TripleStraight(x) => {
                let mut bytes = vec![];
                for y in x.iter() {
                    bytes.extend(y.0.to_bytes());
                    bytes.extend(y.1.to_bytes());
                    bytes.extend(y.2.to_bytes());
                }
                bytes
            }
            TripleStraightWithOne(x) => {
                let mut bytes = vec![];
                for y in x.iter() {
                    bytes.extend(y.0.to_bytes());
                    bytes.extend(y.1.to_bytes());
                    bytes.extend(y.2.to_bytes());
                    bytes.extend(y.3.to_bytes());
                }
                bytes
            }
            TripleStraightWithPair(x) => {
                let mut bytes = vec![];
                for y in x.iter() {
                    bytes.extend(y.0.to_bytes());
                    bytes.extend(y.1.to_bytes());
                    bytes.extend(y.2.to_bytes());
                    bytes.extend(y.3.to_bytes());
                    bytes.extend(y.4.to_bytes());
                }
                bytes
            }
            FourWithTwoSingle(x1, x2, x3, x4, x5, x6) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes.extend(x3.to_bytes());
                bytes.extend(x4.to_bytes());
                bytes.extend(x5.to_bytes());
                bytes.extend(x6.to_bytes());
                bytes
            }
            FourWithTwoPairs(x1, x2, x3, x4, x5, x6, x7, x8) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes.extend(x3.to_bytes());
                bytes.extend(x4.to_bytes());
                bytes.extend(x5.to_bytes());
                bytes.extend(x6.to_bytes());
                bytes.extend(x7.to_bytes());
                bytes.extend(x8.to_bytes());
                bytes
            }
            Bomb(x1, x2, x3, x4) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes.extend(x3.to_bytes());
                bytes.extend(x4.to_bytes());
                bytes
            }
            Rocket(x1, x2) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes
            }
        }
    }
}

pub type EncodingCardCombination = Combination<EncodingCard>;

impl EncodingCardCombination {
    pub fn flatten(&self) -> Vec<ark_bn254::Fr> {
        match self {
            DefaultCombination => vec![],
            Single(c) => {
                let (x, y) = c.0.xy().unwrap();
                vec![x, y]
            }
            Pair(c1, c2) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                vec![x1, y1, x2, y2]
            }
            ThreeOfAKind(c1, c2, c3) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                let (x3, y3) = c3.0.xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3]
            }
            ThreeWithOne(c1, c2, c3, c4) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                let (x3, y3) = c3.0.xy().unwrap();
                let (x4, y4) = c4.0.xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3, x4, y4]
            }
            ThreeWithPair(c1, c2, c3, c4, c5) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                let (x3, y3) = c3.0.xy().unwrap();
                let (x4, y4) = c4.0.xy().unwrap();
                let (x5, y5) = c5.0.xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3, x4, y4, x5, y5]
            }
            Straight(c) => {
                let mut res = vec![];
                for i in c.iter() {
                    let (x, y) = i.0.xy().unwrap();
                    res.push(x);
                    res.push(y)
                }
                res
            }
            DoubleStraight(c) => {
                let mut res = vec![];
                for (i1, i2) in c.iter() {
                    let (x1, y1) = i1.0.xy().unwrap();
                    let (x2, y2) = i2.0.xy().unwrap();
                    res.push(x1);
                    res.push(y1);
                    res.push(x2);
                    res.push(y2);
                }
                res
            }
            TripleStraight(c) => {
                let mut res = vec![];
                for (i1, i2, i3) in c.iter() {
                    let (x1, y1) = i1.0.xy().unwrap();
                    let (x2, y2) = i2.0.xy().unwrap();
                    let (x3, y3) = i3.0.xy().unwrap();
                    res.push(x1);
                    res.push(y1);
                    res.push(x2);
                    res.push(y2);
                    res.push(x3);
                    res.push(y3);
                }
                res
            }
            TripleStraightWithOne(c) => {
                let mut res = vec![];
                for (i1, i2, i3, i4) in c.iter() {
                    let (x1, y1) = i1.0.xy().unwrap();
                    let (x2, y2) = i2.0.xy().unwrap();
                    let (x3, y3) = i3.0.xy().unwrap();
                    let (x4, y4) = i4.0.xy().unwrap();
                    res.push(x1);
                    res.push(y1);
                    res.push(x2);
                    res.push(y2);
                    res.push(x3);
                    res.push(y3);
                    res.push(x4);
                    res.push(y4);
                }
                res
            }
            TripleStraightWithPair(c) => {
                let mut res = vec![];
                for (i1, i2, i3, i4, i5) in c.iter() {
                    let (x1, y1) = i1.0.xy().unwrap();
                    let (x2, y2) = i2.0.xy().unwrap();
                    let (x3, y3) = i3.0.xy().unwrap();
                    let (x4, y4) = i4.0.xy().unwrap();
                    let (x5, y5) = i5.0.xy().unwrap();
                    res.push(x1);
                    res.push(y1);
                    res.push(x2);
                    res.push(y2);
                    res.push(x3);
                    res.push(y3);
                    res.push(x4);
                    res.push(y4);
                    res.push(x5);
                    res.push(y5);
                }

                res
            }
            FourWithTwoSingle(c1, c2, c3, c4, c5, c6) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                let (x3, y3) = c3.0.xy().unwrap();
                let (x4, y4) = c4.0.xy().unwrap();
                let (x5, y5) = c5.0.xy().unwrap();
                let (x6, y6) = c6.0.xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3, x4, y4, x5, y5, x6, y6]
            }
            FourWithTwoPairs(c1, c2, c3, c4, c5, c6, c7, c8) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                let (x3, y3) = c3.0.xy().unwrap();
                let (x4, y4) = c4.0.xy().unwrap();
                let (x5, y5) = c5.0.xy().unwrap();
                let (x6, y6) = c6.0.xy().unwrap();
                let (x7, y7) = c7.0.xy().unwrap();
                let (x8, y8) = c8.0.xy().unwrap();
                vec![
                    x1, y1, x2, y2, x3, y3, x4, y4, x5, y5, x6, y6, x7, y7, x8, y8,
                ]
            }
            Bomb(c1, c2, c3, c4) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                let (x3, y3) = c3.0.xy().unwrap();
                let (x4, y4) = c4.0.xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3, x4, y4]
            }
            Rocket(c1, c2) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                vec![x1, y1, x2, y2]
            }
        }
    }

    #[inline]
    pub fn morph_to_classic(&self) -> Result<ClassicCardCombination> {
        match self {
            DefaultCombination => Ok(DefaultCombination),
            Single(x) => {
                let c = ENCODING_CARDS_MAPPING
                    .get(&x.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(Single(*c))
            }

            Pair(x1, x2) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(&x1.0)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(&x2.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(Pair(*c_1, *c_2))
            }

            ThreeOfAKind(x1, x2, x3) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(&x1.0)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(&x2.0)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(&x3.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(ThreeOfAKind(*c_1, *c_2, *c_3))
            }

            ThreeWithOne(x1, x2, x3, x4) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(&x1.0)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(&x2.0)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(&x3.0)
                    .ok_or(PokerError::MorphError)?;
                let c_4 = ENCODING_CARDS_MAPPING
                    .get(&x4.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(ThreeWithOne(*c_1, *c_2, *c_3, *c_4))
            }

            ThreeWithPair(x1, x2, x3, x4, x5) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(&x1.0)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(&x2.0)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(&x3.0)
                    .ok_or(PokerError::MorphError)?;
                let c_4 = ENCODING_CARDS_MAPPING
                    .get(&x4.0)
                    .ok_or(PokerError::MorphError)?;
                let c_5 = ENCODING_CARDS_MAPPING
                    .get(&x5.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(ThreeWithPair(*c_1, *c_2, *c_3, *c_4, *c_5))
            }

            Straight(x) => {
                let mut classic_card = vec![];
                for y in x.iter() {
                    let c = ENCODING_CARDS_MAPPING
                        .get(&y.0)
                        .ok_or(PokerError::MorphError)?;
                    classic_card.push(*c)
                }

                Ok(Straight(classic_card))
            }

            DoubleStraight(x) => {
                let mut classic_card = vec![];
                for (y1, y2) in x.iter() {
                    let c1 = ENCODING_CARDS_MAPPING
                        .get(&y1.0)
                        .ok_or(PokerError::MorphError)?;
                    let c2 = ENCODING_CARDS_MAPPING
                        .get(&y2.0)
                        .ok_or(PokerError::MorphError)?;
                    classic_card.push((*c1, *c2))
                }

                Ok(DoubleStraight(classic_card))
            }

            TripleStraight(x) => {
                let mut classic_card = vec![];
                for (y1, y2, y3) in x.iter() {
                    let c1 = ENCODING_CARDS_MAPPING
                        .get(&y1.0)
                        .ok_or(PokerError::MorphError)?;
                    let c2 = ENCODING_CARDS_MAPPING
                        .get(&y2.0)
                        .ok_or(PokerError::MorphError)?;
                    let c3 = ENCODING_CARDS_MAPPING
                        .get(&y3.0)
                        .ok_or(PokerError::MorphError)?;
                    classic_card.push((*c1, *c2, *c3))
                }

                Ok(TripleStraight(classic_card))
            }

            TripleStraightWithOne(x) => {
                let mut classic_card = vec![];
                for (y1, y2, y3, y4) in x.iter() {
                    let c1 = ENCODING_CARDS_MAPPING
                        .get(&y1.0)
                        .ok_or(PokerError::MorphError)?;
                    let c2 = ENCODING_CARDS_MAPPING
                        .get(&y2.0)
                        .ok_or(PokerError::MorphError)?;
                    let c3 = ENCODING_CARDS_MAPPING
                        .get(&y3.0)
                        .ok_or(PokerError::MorphError)?;
                    let c4 = ENCODING_CARDS_MAPPING
                        .get(&y4.0)
                        .ok_or(PokerError::MorphError)?;
                    classic_card.push((*c1, *c2, *c3, *c4))
                }

                Ok(TripleStraightWithOne(classic_card))
            }

            TripleStraightWithPair(x) => {
                let mut classic_card = vec![];
                for (y1, y2, y3, y4, y5) in x.iter() {
                    let c1 = ENCODING_CARDS_MAPPING
                        .get(&y1.0)
                        .ok_or(PokerError::MorphError)?;
                    let c2 = ENCODING_CARDS_MAPPING
                        .get(&y2.0)
                        .ok_or(PokerError::MorphError)?;
                    let c3 = ENCODING_CARDS_MAPPING
                        .get(&y3.0)
                        .ok_or(PokerError::MorphError)?;
                    let c4 = ENCODING_CARDS_MAPPING
                        .get(&y4.0)
                        .ok_or(PokerError::MorphError)?;
                    let c5 = ENCODING_CARDS_MAPPING
                        .get(&y5.0)
                        .ok_or(PokerError::MorphError)?;
                    classic_card.push((*c1, *c2, *c3, *c4, *c5))
                }

                Ok(TripleStraightWithPair(classic_card))
            }

            FourWithTwoSingle(x1, x2, x3, x4, x5, x6) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(&x1.0)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(&x2.0)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(&x3.0)
                    .ok_or(PokerError::MorphError)?;
                let c_4 = ENCODING_CARDS_MAPPING
                    .get(&x4.0)
                    .ok_or(PokerError::MorphError)?;
                let c_5 = ENCODING_CARDS_MAPPING
                    .get(&x5.0)
                    .ok_or(PokerError::MorphError)?;
                let c_6 = ENCODING_CARDS_MAPPING
                    .get(&x6.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(FourWithTwoSingle(*c_1, *c_2, *c_3, *c_4, *c_5, *c_6))
            }

            FourWithTwoPairs(x1, x2, x3, x4, x5, x6, x7, x8) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(&x1.0)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(&x2.0)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(&x3.0)
                    .ok_or(PokerError::MorphError)?;
                let c_4 = ENCODING_CARDS_MAPPING
                    .get(&x4.0)
                    .ok_or(PokerError::MorphError)?;
                let c_5 = ENCODING_CARDS_MAPPING
                    .get(&x5.0)
                    .ok_or(PokerError::MorphError)?;
                let c_6 = ENCODING_CARDS_MAPPING
                    .get(&x6.0)
                    .ok_or(PokerError::MorphError)?;
                let c_7 = ENCODING_CARDS_MAPPING
                    .get(&x7.0)
                    .ok_or(PokerError::MorphError)?;
                let c_8 = ENCODING_CARDS_MAPPING
                    .get(&x8.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(FourWithTwoPairs(
                    *c_1, *c_2, *c_3, *c_4, *c_5, *c_6, *c_7, *c_8,
                ))
            }

            Bomb(x1, x2, x3, x4) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(&x1.0)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(&x2.0)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(&x3.0)
                    .ok_or(PokerError::MorphError)?;
                let c_4 = ENCODING_CARDS_MAPPING
                    .get(&x4.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(Bomb(*c_1, *c_2, *c_3, *c_4))
            }

            Rocket(x1, x2) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(&x1.0)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(&x2.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(Rocket(*c_1, *c_2))
            }
        }
    }
}

pub type CryptoCardCombination = Combination<CryptoCard>;

impl CryptoCardCombination {
    pub fn flatten(&self) -> Vec<ark_bn254::Fr> {
        match self {
            DefaultCombination => vec![],
            Single(c) => c.0.flatten().to_vec(),
            Pair(c1, c2) => {
                let mut v1 = c1.0.flatten().to_vec();
                let v2 = c2.0.flatten().to_vec();
                v1.extend(v2);
                v1
            }
            ThreeOfAKind(c1, c2, c3) => {
                let mut v1 = c1.0.flatten().to_vec();
                let v2 = c2.0.flatten().to_vec();
                let v3 = c3.0.flatten().to_vec();
                v1.extend(v2);
                v1.extend(v3);
                v1
            }
            ThreeWithOne(c1, c2, c3, c4) => {
                let mut v1 = c1.0.flatten().to_vec();
                let v2 = c2.0.flatten().to_vec();
                let v3 = c3.0.flatten().to_vec();
                let v4 = c4.0.flatten().to_vec();
                v1.extend(v2);
                v1.extend(v3);
                v1.extend(v4);
                v1
            }
            ThreeWithPair(c1, c2, c3, c4, c5) => {
                let mut v1 = c1.0.flatten().to_vec();
                let v2 = c2.0.flatten().to_vec();
                let v3 = c3.0.flatten().to_vec();
                let v4 = c4.0.flatten().to_vec();
                let v5 = c5.0.flatten().to_vec();
                v1.extend(v2);
                v1.extend(v3);
                v1.extend(v4);
                v1.extend(v5);
                v1
            }
            Straight(c) => {
                let mut res = vec![];
                for i in c.iter() {
                    res.extend(i.0.flatten().to_vec())
                }
                res
            }
            DoubleStraight(c) => {
                let mut res = vec![];
                for (x1, x2) in c.iter() {
                    res.extend(x1.0.flatten().to_vec());
                    res.extend(x2.0.flatten().to_vec());
                }
                res
            }
            TripleStraight(c) => {
                let mut res = vec![];
                for (x1, x2, x3) in c.iter() {
                    res.extend(x1.0.flatten().to_vec());
                    res.extend(x2.0.flatten().to_vec());
                    res.extend(x3.0.flatten().to_vec());
                }
                res
            }
            TripleStraightWithOne(c) => {
                let mut res = vec![];
                for (x1, x2, x3, x4) in c.iter() {
                    res.extend(x1.0.flatten().to_vec());
                    res.extend(x2.0.flatten().to_vec());
                    res.extend(x3.0.flatten().to_vec());
                    res.extend(x4.0.flatten().to_vec());
                }
                res
            }
            TripleStraightWithPair(c) => {
                let mut res = vec![];
                for (x1, x2, x3, x4, x5) in c.iter() {
                    res.extend(x1.0.flatten().to_vec());
                    res.extend(x2.0.flatten().to_vec());
                    res.extend(x3.0.flatten().to_vec());
                    res.extend(x4.0.flatten().to_vec());
                    res.extend(x5.0.flatten().to_vec());
                }
                res
            }
            FourWithTwoSingle(c1, c2, c3, c4, c5, c6) => {
                let mut v1 = c1.0.flatten().to_vec();
                let v2 = c2.0.flatten().to_vec();
                let v3 = c3.0.flatten().to_vec();
                let v4 = c4.0.flatten().to_vec();
                let v5 = c5.0.flatten().to_vec();
                let v6 = c6.0.flatten().to_vec();
                v1.extend(v2);
                v1.extend(v3);
                v1.extend(v4);
                v1.extend(v5);
                v1.extend(v6);
                v1
            }
            FourWithTwoPairs(c1, c2, c3, c4, c5, c6, c7, c8) => {
                let mut v1 = c1.0.flatten().to_vec();
                let v2 = c2.0.flatten().to_vec();
                let v3 = c3.0.flatten().to_vec();
                let v4 = c4.0.flatten().to_vec();
                let v5 = c5.0.flatten().to_vec();
                let v6 = c6.0.flatten().to_vec();
                let v7 = c7.0.flatten().to_vec();
                let v8 = c8.0.flatten().to_vec();
                v1.extend(v2);
                v1.extend(v3);
                v1.extend(v4);
                v1.extend(v5);
                v1.extend(v6);
                v1.extend(v7);
                v1.extend(v8);
                v1
            }
            Bomb(c1, c2, c3, c4) => {
                let mut v1 = c1.0.flatten().to_vec();
                let v2 = c2.0.flatten().to_vec();
                let v3 = c3.0.flatten().to_vec();
                let v4 = c4.0.flatten().to_vec();
                v1.extend(v2);
                v1.extend(v3);
                v1.extend(v4);
                v1
            }
            Rocket(c1, c2) => {
                let mut v1 = c1.0.flatten().to_vec();
                let v2 = c2.0.flatten().to_vec();
                v1.extend(v2);
                v1
            }
        }
    }

    pub fn morph_to_encoding(&self, reveals: &[EncodingCard]) -> EncodingCardCombination {
        match self {
            DefaultCombination => DefaultCombination,
            Single(_) => Single(reveals[0]),
            Pair(_, _) => Pair(reveals[0], reveals[1]),
            ThreeOfAKind(_, _, _) => ThreeOfAKind(reveals[0], reveals[1], reveals[2]),
            ThreeWithOne(_, _, _, _) => {
                ThreeWithOne(reveals[0], reveals[1], reveals[2], reveals[3])
            }
            ThreeWithPair(_, _, _, _, _) => {
                ThreeWithPair(reveals[0], reveals[1], reveals[2], reveals[3], reveals[4])
            }
            Straight(_) => Straight(reveals.to_vec()),
            DoubleStraight(_) => DoubleStraight(
                reveals
                    .chunks(2)
                    .map(|x| (x[0], x[1]))
                    .collect::<Vec<(_, _)>>(),
            ),
            TripleStraight(_) => TripleStraight(
                reveals
                    .chunks(3)
                    .map(|x| (x[0], x[1], x[2]))
                    .collect::<Vec<(_, _, _)>>(),
            ),
            TripleStraightWithOne(_) => TripleStraightWithOne(
                reveals
                    .chunks(4)
                    .map(|x| (x[0], x[1], x[2], x[3]))
                    .collect::<Vec<(_, _, _, _)>>(),
            ),
            TripleStraightWithPair(_) => TripleStraightWithPair(
                reveals
                    .chunks(5)
                    .map(|x| (x[0], x[1], x[2], x[3], x[4]))
                    .collect::<Vec<(_, _, _, _, _)>>(),
            ),
            FourWithTwoSingle(_, _, _, _, _, _) => FourWithTwoSingle(
                reveals[0], reveals[1], reveals[2], reveals[3], reveals[4], reveals[5],
            ),
            FourWithTwoPairs(_, _, _, _, _, _, _, _) => FourWithTwoPairs(
                reveals[0], reveals[1], reveals[2], reveals[3], reveals[4], reveals[5], reveals[6],
                reveals[7],
            ),
            Bomb(_, _, _, _) => Bomb(reveals[0], reveals[1], reveals[2], reveals[3]),
            Rocket(_, _) => Rocket(reveals[0], reveals[1]),
        }
    }
}
