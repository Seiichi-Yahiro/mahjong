use crate::plugins::assets::tiles::bonus::{Bonus, Plant, Season};
use crate::plugins::assets::tiles::honor::{Dragon, Honor, Wind};
use crate::plugins::assets::tiles::suit::{Number, Suit};
use crate::plugins::assets::tiles::EnumIter;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Component)]
pub enum Tile {
    Suit(Suit),
    Honor(Honor),
    Bonus(Bonus),
}

impl Tile {
    pub fn new_normal_set() -> Vec<Self> {
        let dots = Number::iter().map(|num| Suit::Dot(num));
        let bamboos = Number::iter().map(|num| Suit::Bamboo(num));
        let chars = Number::iter().map(|num| Suit::Char(num));
        let suits = dots.chain(bamboos).chain(chars).map(Tile::from);

        let winds = Wind::iter().map(Honor::from);
        let dragons = Dragon::iter().map(Honor::from);
        let honors = winds.chain(dragons).map(Tile::from);

        suits.chain(honors).collect()
    }

    pub fn new_bonus_set() -> Vec<Self> {
        let seasons = Season::iter().map(Bonus::from);
        let plants = Plant::iter().map(Bonus::from);
        seasons.chain(plants).map(Tile::from).collect()
    }

    pub fn cover_id(&self) -> i32 {
        match self {
            Tile::Suit(Suit::Dot(Number::One)) => 0,
            Tile::Suit(Suit::Dot(Number::Two)) => 1,
            Tile::Suit(Suit::Dot(Number::Three)) => 2,
            Tile::Suit(Suit::Dot(Number::Four)) => 3,
            Tile::Suit(Suit::Dot(Number::Five)) => 4,
            Tile::Suit(Suit::Dot(Number::Six)) => 5,
            Tile::Suit(Suit::Dot(Number::Seven)) => 6,
            Tile::Suit(Suit::Dot(Number::Eight)) => 7,
            Tile::Suit(Suit::Dot(Number::Nine)) => 8,
            Tile::Suit(Suit::Bamboo(Number::One)) => 9,
            Tile::Suit(Suit::Bamboo(Number::Two)) => 10,
            Tile::Suit(Suit::Bamboo(Number::Three)) => 11,
            Tile::Suit(Suit::Bamboo(Number::Four)) => 12,
            Tile::Suit(Suit::Bamboo(Number::Five)) => 13,
            Tile::Suit(Suit::Bamboo(Number::Six)) => 14,
            Tile::Suit(Suit::Bamboo(Number::Seven)) => 15,
            Tile::Suit(Suit::Bamboo(Number::Eight)) => 16,
            Tile::Suit(Suit::Bamboo(Number::Nine)) => 17,
            Tile::Suit(Suit::Char(Number::One)) => 18,
            Tile::Suit(Suit::Char(Number::Two)) => 19,
            Tile::Suit(Suit::Char(Number::Three)) => 20,
            Tile::Suit(Suit::Char(Number::Four)) => 21,
            Tile::Suit(Suit::Char(Number::Five)) => 22,
            Tile::Suit(Suit::Char(Number::Six)) => 23,
            Tile::Suit(Suit::Char(Number::Seven)) => 24,
            Tile::Suit(Suit::Char(Number::Eight)) => 25,
            Tile::Suit(Suit::Char(Number::Nine)) => 26,
            Tile::Honor(Honor::Wind(Wind::East)) => 27,
            Tile::Honor(Honor::Wind(Wind::South)) => 28,
            Tile::Honor(Honor::Wind(Wind::West)) => 29,
            Tile::Honor(Honor::Wind(Wind::North)) => 30,
            Tile::Honor(Honor::Dragon(Dragon::White)) => 31,
            Tile::Honor(Honor::Dragon(Dragon::Green)) => 32,
            Tile::Honor(Honor::Dragon(Dragon::Red)) => 33,
            Tile::Bonus(Bonus::Season(Season::Spring)) => 34,
            Tile::Bonus(Bonus::Season(Season::Summer)) => 35,
            Tile::Bonus(Bonus::Season(Season::Fall)) => 36,
            Tile::Bonus(Bonus::Season(Season::Winter)) => 37,
            Tile::Bonus(Bonus::Plant(Plant::Plum)) => 38,
            Tile::Bonus(Bonus::Plant(Plant::Orchid)) => 39,
            Tile::Bonus(Bonus::Plant(Plant::Chrysanthemum)) => 40,
            Tile::Bonus(Bonus::Plant(Plant::Bamboo)) => 41,
        }
    }
}

impl From<Suit> for Tile {
    fn from(suit: Suit) -> Self {
        Tile::Suit(suit)
    }
}

impl From<Honor> for Tile {
    fn from(honor: Honor) -> Self {
        Tile::Honor(honor)
    }
}

impl From<Bonus> for Tile {
    fn from(bonus: Bonus) -> Self {
        Tile::Bonus(bonus)
    }
}

impl From<Wind> for Tile {
    fn from(wind: Wind) -> Self {
        Tile::Honor(wind.into())
    }
}

impl From<Dragon> for Tile {
    fn from(dragon: Dragon) -> Self {
        Tile::Honor(dragon.into())
    }
}

impl From<Season> for Tile {
    fn from(season: Season) -> Self {
        Tile::Bonus(season.into())
    }
}

impl From<Plant> for Tile {
    fn from(plant: Plant) -> Self {
        Tile::Bonus(plant.into())
    }
}

impl EnumIter for Tile {
    fn next(self) -> Self {
        match self {
            Self::Suit(tile) => Self::Suit(tile.next()),
            Self::Honor(tile) => Self::Honor(tile.next()),
            tile => tile,
        }
    }
}
