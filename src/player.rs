use crate::tiles::Wind;
use bevy::prelude::*;
use rand::Rng;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlayerType {
    Human,
    Ai,
}

#[derive(Debug)]
pub struct Player {
    pub r#type: PlayerType,
    pub seat: Wind,
    pub wind: Wind,
    pub points: i32,
}

impl Player {
    pub fn new(seat: Wind, wind: Wind, r#type: PlayerType) -> Self {
        Self {
            seat,
            wind,
            r#type,
            points: 25_000,
        }
    }
}

#[derive(Debug)]
pub struct Players {
    players: Vec<Player>,
    current: usize,
    dealer: usize,
}

impl Players {
    pub fn new() -> Self {
        let wind = Wind::iter()
            .cycle()
            .skip(rand::thread_rng().gen_range(0..4));

        let players: Vec<Player> = Wind::iter()
            .zip(wind)
            .map(|(seat, wind)| {
                let player_type = if seat == Wind::South {
                    PlayerType::Human
                } else {
                    PlayerType::Ai
                };

                Player::new(seat, wind, player_type)
            })
            .collect();

        let dealer = players
            .iter()
            .enumerate()
            .find(|(_, player)| player.wind == Wind::East)
            .map(|(index, player)| {
                info!("Dealer sits {:?}.", player.seat);
                index
            })
            .unwrap();

        Self {
            players,
            dealer,
            current: dealer,
        }
    }

    pub fn dealer(&self) -> &Player {
        &self.players[self.dealer]
    }
}
