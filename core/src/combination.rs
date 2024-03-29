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

    // Three or more consecutive pairs
    ConnectedPairs(Vec<(T, T)>),

    // Three cards of the same rank
    Three(T, T, T),

    // Three cards of the same rank with one single card
    ThreeWithOne(T, T, T, T),

    // Three cards of the same rank with one pair
    ThreeWithPair(T, T, T, T, T),

    // Five or more consecutive single cards
    Straight(Vec<T>),

    // Four cards of the same rank with two cards
    FourWithTwo(T, T, T, T, T, T),

    // Four cards of the same rank
    Bomb(T, T, T, T),

    /// AAA is maximum
    AAA(T, T, T),
}

impl<T: Clone + Copy> Combination<T> {
    #[inline]
    pub fn weight(&self) -> u8 {
        match self {
            DefaultCombination => 0,
            Single(_) => 1,
            Pair(_, _) => 1,
            ConnectedPairs(_) => 1,
            Three(_, _, _) => 1,
            ThreeWithOne(_, _, _, _) => 1,
            ThreeWithPair(_, _, _, _, _) => 1,
            Straight(_) => 1,
            FourWithTwo(_, _, _, _, _, _) => 1,
            Bomb(_, _, _, _) => 2,
            AAA(_, _, _) => 3,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            DefaultCombination => 0,
            Single(_) => 1,
            Pair(_, _) => 2,
            Three(_, _, _) => 3,
            ThreeWithOne(_, _, _, _) => 4,
            ThreeWithPair(_, _, _, _, _) => 5,
            Straight(x) => x.len(),
            ConnectedPairs(x) => 2 * x.len(),
            FourWithTwo(_, _, _, _, _, _) => 6,
            Bomb(_, _, _, _) => 4,
            AAA(_, _, _) => 3,
        }
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<T> {
        match self {
            DefaultCombination => vec![],
            Single(x) => vec![*x],
            Pair(x1, x2) => vec![*x1, *x2],
            Three(x1, x2, x3) => vec![*x1, *x2, *x3],
            ThreeWithOne(x1, x2, x3, x4) => vec![*x1, *x2, *x3, *x4],
            ThreeWithPair(x1, x2, x3, x4, x5) => vec![*x1, *x2, *x3, *x4, *x5],
            Straight(x) => x.to_vec(),
            ConnectedPairs(x) => x.iter().flat_map(|(x1, x2)| vec![*x1, *x2]).collect(),
            FourWithTwo(x1, x2, x3, x4, x5, x6) => vec![*x1, *x2, *x3, *x4, *x5, *x6],
            Bomb(x1, x2, x3, x4) => vec![*x1, *x2, *x3, *x4],
            AAA(x1, x2, x3) => vec![*x1, *x2, *x3],
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
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DefaultCombination, DefaultCombination) => true,

            (Single(x), Single(y)) => x.get_value().eq(&y.get_value()),

            (Pair(x, _), Pair(y, _)) => x.get_value().eq(&y.get_value()),

            (Three(x, _, _), Three(y, _, _)) => x.get_value().eq(&y.get_value()),

            (ThreeWithOne(x, _, _, _), ThreeWithOne(y, _, _, _)) => {
                x.get_value().eq(&y.get_value())
            }

            (ThreeWithPair(x, _, _, _, _), ThreeWithPair(y, _, _, _, _)) => {
                x.get_value().eq(&&y.get_value())
            }

            (Straight(x), Straight(y)) => {
                assert_eq!(x.len(), y.len()); // todo if x.len() ！= y.len() return false
                x.last()
                    .unwrap()
                    .get_value()
                    .eq(&y.last().unwrap().get_value())
            }

            (ConnectedPairs(x), ConnectedPairs(y)) => {
                assert_eq!(x.len(), y.len());
                x.last()
                    .unwrap()
                    .0
                    .get_value()
                    .eq(&y.last().unwrap().0.get_value())
            }

            (FourWithTwo(x, _, _, _, _, _), FourWithTwo(y, _, _, _, _, _)) => {
                x.get_value().eq(&y.get_value())
            }

            (Bomb(x, _, _, _), Bomb(y, _, _, _)) => x.get_value().eq(&y.get_value()),

            _ => unreachable!(),
        }
    }
}

impl PartialOrd for ClassicCardCombination {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.weight() == other.weight() {
            match (self, other) {
                (Single(x), Single(y)) => x.weight().partial_cmp(&y.weight()),

                (Pair(x, _), Pair(y, _)) => x.weight().partial_cmp(&y.weight()),

                (Three(x, _, _), Three(y, _, _)) => x.weight().partial_cmp(&y.weight()),

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

                (ConnectedPairs(x), ConnectedPairs(y)) => {
                    assert_eq!(x.len(), y.len());
                    x.last()
                        .unwrap()
                        .0
                        .weight()
                        .partial_cmp(&y.last().unwrap().0.weight())
                }

                (FourWithTwo(x, _, _, _, _, _), FourWithTwo(y, _, _, _, _, _)) => {
                    x.weight().partial_cmp(&y.weight())
                }

                (Bomb(x, _, _, _), Bomb(y, _, _, _)) => x.weight().partial_cmp(&y.weight()),

                _ => unreachable!(),
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

            Three(x1, x2, x3) => {
                x1.get_value() == x2.get_value()
                    && x1.get_value() == x3.get_value()
                    && x1.get_value() != Value::Ace
            }

            ThreeWithOne(x1, x2, x3, x4) => {
                x1.get_value() == x2.get_value()
                    && x1.get_value() == x3.get_value()
                    && x1.get_value() != x4.get_value()
            }

            ThreeWithPair(x1, x2, x3, y1, y2) => {
                let condition1 = Three(*x1, *x2, *x3).validate_rules();
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

            ConnectedPairs(x) => {
                let condition1 = x.iter().all(|(t1, t2)| t1.get_value() == t2.get_value());

                let stright = x.iter().map(|x| x.0).collect::<Vec<_>>();
                if stright.len() < 3 {
                    return false;
                }

                let condition2 = stright
                    .windows(2)
                    .all(|y| y[1].weight() == y[0].weight() + 1);

                condition1 && condition2
            }

            FourWithTwo(x1, x2, x3, x4, _, _) => {
                x1.get_value() == x2.get_value()
                    && x2.get_value() == x3.get_value()
                    && x3.get_value() == x4.get_value()
            }

            Bomb(x1, x2, x3, x4) => {
                x1.get_value() == x2.get_value()
                    && x2.get_value() == x3.get_value()
                    && x3.get_value() == x4.get_value()
            }

            AAA(x1, x2, x3) => {
                x1.get_value() == Value::Ace
                    && x2.get_value() == Value::Ace
                    && x3.get_value() == Value::Ace
            }
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
            Three(x1, x2, x3) => {
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
            ConnectedPairs(x) => {
                let mut bytes = vec![];
                for y in x.iter() {
                    bytes.extend(y.0.to_bytes());
                    bytes.extend(y.1.to_bytes());
                }
                bytes
            }
            FourWithTwo(x1, x2, x3, x4, x5, x6) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes.extend(x3.to_bytes());
                bytes.extend(x4.to_bytes());
                bytes.extend(x5.to_bytes());
                bytes.extend(x6.to_bytes());
                bytes
            }
            Bomb(x1, x2, x3, x4) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes.extend(x3.to_bytes());
                bytes.extend(x4.to_bytes());
                bytes
            }
            AAA(x1, x2, x3) => {
                let mut bytes = x1.to_bytes();
                bytes.extend(x2.to_bytes());
                bytes.extend(x3.to_bytes());
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
            Three(c1, c2, c3) => {
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
            ConnectedPairs(c) => {
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
            FourWithTwo(c1, c2, c3, c4, c5, c6) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                let (x3, y3) = c3.0.xy().unwrap();
                let (x4, y4) = c4.0.xy().unwrap();
                let (x5, y5) = c5.0.xy().unwrap();
                let (x6, y6) = c6.0.xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3, x4, y4, x5, y5, x6, y6]
            }
            Bomb(c1, c2, c3, c4) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                let (x3, y3) = c3.0.xy().unwrap();
                let (x4, y4) = c4.0.xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3, x4, y4]
            }
            AAA(c1, c2, c3) => {
                let (x1, y1) = c1.0.xy().unwrap();
                let (x2, y2) = c2.0.xy().unwrap();
                let (x3, y3) = c3.0.xy().unwrap();
                vec![x1, y1, x2, y2, x3, y3]
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

            Three(x1, x2, x3) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(&x1.0)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(&x2.0)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(&x3.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(Three(*c_1, *c_2, *c_3))
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

            ConnectedPairs(x) => {
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

                Ok(ConnectedPairs(classic_card))
            }

            FourWithTwo(x1, x2, x3, x4, x5, x6) => {
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

                Ok(FourWithTwo(*c_1, *c_2, *c_3, *c_4, *c_5, *c_6))
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

            AAA(x1, x2, x3) => {
                let c_1 = ENCODING_CARDS_MAPPING
                    .get(&x1.0)
                    .ok_or(PokerError::MorphError)?;
                let c_2 = ENCODING_CARDS_MAPPING
                    .get(&x2.0)
                    .ok_or(PokerError::MorphError)?;
                let c_3 = ENCODING_CARDS_MAPPING
                    .get(&x3.0)
                    .ok_or(PokerError::MorphError)?;

                Ok(AAA(*c_1, *c_2, *c_3))
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
            Three(c1, c2, c3) => {
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
            ConnectedPairs(c) => {
                let mut res = vec![];
                for (x1, x2) in c.iter() {
                    res.extend(x1.0.flatten().to_vec());
                    res.extend(x2.0.flatten().to_vec());
                }
                res
            }
            FourWithTwo(c1, c2, c3, c4, c5, c6) => {
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
            AAA(c1, c2, c3) => {
                let mut v1 = c1.0.flatten().to_vec();
                let v2 = c2.0.flatten().to_vec();
                let v3 = c3.0.flatten().to_vec();
                v1.extend(v2);
                v1.extend(v3);
                v1
            }
        }
    }

    pub fn morph_to_encoding(&self, reveals: &[EncodingCard]) -> EncodingCardCombination {
        match self {
            DefaultCombination => DefaultCombination,
            Single(_) => Single(reveals[0]),
            Pair(_, _) => Pair(reveals[0], reveals[1]),
            Three(_, _, _) => Three(reveals[0], reveals[1], reveals[2]),
            ThreeWithOne(_, _, _, _) => {
                ThreeWithOne(reveals[0], reveals[1], reveals[2], reveals[3])
            }
            ThreeWithPair(_, _, _, _, _) => {
                ThreeWithPair(reveals[0], reveals[1], reveals[2], reveals[3], reveals[4])
            }
            Straight(_) => Straight(reveals.to_vec()),
            ConnectedPairs(_) => ConnectedPairs(
                reveals
                    .chunks(2)
                    .map(|x| (x[0], x[1]))
                    .collect::<Vec<(_, _)>>(),
            ),
            FourWithTwo(_, _, _, _, _, _) => FourWithTwo(
                reveals[0], reveals[1], reveals[2], reveals[3], reveals[4], reveals[5],
            ),
            Bomb(_, _, _, _) => Bomb(reveals[0], reveals[1], reveals[2], reveals[3]),
            AAA(_, _, _) => AAA(reveals[0], reveals[1], reveals[2]),
        }
    }
}
