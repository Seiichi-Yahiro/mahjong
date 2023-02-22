#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Bonus {
    Season(Season),
    Plant(Plant),
}

impl From<Season> for Bonus {
    fn from(season: Season) -> Self {
        Bonus::Season(season)
    }
}

impl From<Plant> for Bonus {
    fn from(plant: Plant) -> Self {
        Bonus::Plant(plant)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        [Self::Spring, Self::Summer, Self::Fall, Self::Winter]
            .iter()
            .copied()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Plant {
    Plum,
    Orchid,
    Chrysanthemum,
    Bamboo,
}

impl Plant {
    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        [Self::Plum, Self::Orchid, Self::Chrysanthemum, Self::Bamboo]
            .iter()
            .copied()
    }
}
