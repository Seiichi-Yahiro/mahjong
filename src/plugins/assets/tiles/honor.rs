use crate::plugins::assets::tiles::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Honor {
    Wind(Wind),
    Dragon(Dragon),
}

impl From<Wind> for Honor {
    fn from(wind: Wind) -> Self {
        Honor::Wind(wind)
    }
}

impl From<Dragon> for Honor {
    fn from(dragon: Dragon) -> Self {
        Honor::Dragon(dragon)
    }
}

impl EnumIter for Honor {
    fn next(self) -> Self {
        match self {
            Self::Wind(honor) => Self::Wind(honor.next()),
            Self::Dragon(honor) => Self::Dragon(honor.next()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Wind {
    East,
    South,
    West,
    North,
}

impl Wind {
    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        [Self::East, Self::South, Self::West, Self::North]
            .iter()
            .copied()
    }
}

impl EnumIter for Wind {
    fn next(self) -> Self {
        match self {
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::North => Self::East,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Dragon {
    White,
    Green,
    Red,
}

impl Dragon {
    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        [Self::White, Self::Green, Self::Red].iter().copied()
    }
}

impl EnumIter for Dragon {
    fn next(self) -> Self {
        match self {
            Self::White => Self::Green,
            Self::Green => Self::Red,
            Self::Red => Self::White,
        }
    }
}
