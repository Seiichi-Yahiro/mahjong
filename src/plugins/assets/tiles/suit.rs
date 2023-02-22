use crate::plugins::assets::tiles::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Suit {
    Dot(Number),
    Bamboo(Number),
    Char(Number),
}

impl EnumIter for Suit {
    fn next(self) -> Self {
        match self {
            Self::Dot(number) => Self::Dot(number.next()),
            Self::Bamboo(number) => Self::Bamboo(number.next()),
            Self::Char(number) => Self::Char(number.next()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Number {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl Number {
    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        [
            Self::One,
            Self::Two,
            Self::Three,
            Self::Four,
            Self::Five,
            Self::Six,
            Self::Seven,
            Self::Eight,
            Self::Nine,
        ]
        .iter()
        .copied()
    }
}

impl EnumIter for Number {
    fn next(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::Three,
            Self::Three => Self::Four,
            Self::Four => Self::Five,
            Self::Five => Self::Six,
            Self::Six => Self::Seven,
            Self::Seven => Self::Eight,
            Self::Eight => Self::Nine,
            Self::Nine => Self::One,
        }
    }
}
