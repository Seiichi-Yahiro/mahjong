use crate::tiles::{EnumIter, Wind};
use bevy::prelude::*;
use rand::prelude::IteratorRandom;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlayerType {
    Human,
    Ai,
}

#[derive(Debug, Copy, Clone)]
pub struct Points(i32);

impl Points {
    pub fn new() -> Self {
        Self(25_000)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Seat(pub Wind);

#[derive(Debug, Copy, Clone)]
pub struct Dealer;

pub fn setup_players_system(commands: &mut Commands) {
    let mut wind = Wind::iter().choose(&mut rand::thread_rng()).unwrap();

    for seat in Wind::iter() {
        let player_type = if seat == Wind::South {
            PlayerType::Human
        } else {
            PlayerType::Ai
        };

        wind = wind.next();

        let current_entity = commands.spawn((Seat(seat), wind, Points::new(), player_type));

        if wind == Wind::East {
            info!("{:?} got assigned as Dealer!", seat);
            current_entity.with(Dealer);
        }
    }
}
