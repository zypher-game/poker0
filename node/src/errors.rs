use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NodeError {
    PlayFirstError,
    // PlayCardError,
    PlayRoundError,
    RoundOverError,
}

// 1 : the current play is smaller than the previous round
// 2 : the play round is less than current round

impl Display for NodeError {
    fn fmt(&self, formatter: &mut Formatter) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::PlayFirstError => "The heart 3 must play first",
            //   Self::PlayCardError => "The current play is smaller than the previous round",
            Self::PlayRoundError => "The play round is less than current round",
            Self::RoundOverError => "The current round has ended",
        })
    }
}
