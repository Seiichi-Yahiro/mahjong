use crate::tiles::{TileAssetData, Wind};
use crate::wall::{TileEntity, Wall, HALF_WALL_LENGTH};
use bevy::prelude::*;
use bevy_easings::{Ease, EaseFunction, EaseMethod, EasingChainComponent, EasingType};
use rand::Rng;

const TILES_IN_HAND: usize = 13;
const MAX_TILES_IN_HAND: usize = TILES_IN_HAND + 1;

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
    pub tiles: Vec<TileEntity>,
}

impl Player {
    pub fn new(seat: Wind, wind: Wind, r#type: PlayerType) -> Self {
        Self {
            seat,
            wind,
            r#type,
            points: 25_000,
            tiles: Vec::with_capacity(MAX_TILES_IN_HAND),
        }
    }

    fn add_tiles(&mut self, tiles: &[TileEntity]) -> Result<(), String> {
        let new_len = self.tiles.len() + tiles.len();

        if new_len > MAX_TILES_IN_HAND {
            let msg = format!("Player sitting {:?} cannot draw {} tiles as there are already {} tiles in his hand ({} > {})!", self.seat, tiles.len(), self.tiles.len(), new_len, MAX_TILES_IN_HAND);
            Err(msg)
        } else {
            self.tiles.extend_from_slice(tiles);
            Ok(())
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

    pub fn current_player(&self) -> &Player {
        &self.players[self.current]
    }

    pub fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current]
    }

    pub fn end_turn(&mut self) {
        self.current = (self.current + 1) % self.players.len();
    }

    pub fn draw_tile_system(
        commands: &mut Commands,
        mut players: ResMut<Players>,
        mut wall: ResMut<Wall>,
        events: Query<(Entity, &DrawTiles)>,
        transform_query: Query<&Transform>,
    ) {
        for (event, &DrawTiles(amount)) in events.iter() {
            let tiles = wall.draw(amount);

            let current_number_of_tiles = players.current_player().tiles.len();
            let current_seat = players.current_player().seat;

            match players.current_player_mut().add_tiles(&tiles) {
                Ok(_) => {
                    for (index, tile_entity) in tiles.iter().enumerate() {
                        match transform_query.get(tile_entity.entity) {
                            Ok(transform) => {
                                let animation = calculate_wall_to_hand_animation(
                                    index + current_number_of_tiles,
                                    current_seat,
                                    *transform,
                                );
                                commands.insert_one(tile_entity.entity, animation);
                            }
                            Err(err) => {
                                error!("Could not query tile because of {:?}!", err)
                            }
                        }
                    }
                }
                Err(msg) => {
                    error!("{}", msg);
                }
            }

            commands.despawn(event);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DrawTiles(pub usize);

fn calculate_hand_transform_from_index(index: usize) -> Transform {
    let half_hand_length = TileAssetData::WIDTH * (TILES_IN_HAND as f32) / 2.0;
    let x = index as f32 * TileAssetData::WIDTH + TileAssetData::WIDTH / 2.0 - half_hand_length;

    let y = TileAssetData::DEPTH / 2.0;

    let z = HALF_WALL_LENGTH + TileAssetData::WIDTH + TileAssetData::DEPTH;

    Transform {
        translation: Vec3::new(x, y, z),
        rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        scale: Vec3::one(),
    }
}

fn calculate_rotation_from_seat(seat: Wind) -> Quat {
    let angle = match seat {
        Wind::East => -std::f32::consts::FRAC_PI_2,
        Wind::South => 0.0,
        Wind::West => std::f32::consts::FRAC_PI_2,
        Wind::North => std::f32::consts::PI,
    };

    Quat::from_rotation_y(angle)
}

fn calculate_wall_to_hand_animation(
    index: usize,
    seat: Wind,
    transform: Transform,
) -> EasingChainComponent<Transform> {
    let seat_rotation = calculate_rotation_from_seat(seat);
    let transform_to =
        Transform::from_rotation(seat_rotation) * calculate_hand_transform_from_index(index);

    let up_covered = Transform {
        translation: transform.translation + Vec3::new(0.0, TileAssetData::HEIGHT * 2.0, 0.0),
        ..transform
    };

    let up_hand_covered = Transform {
        translation: Vec3::new(
            transform_to.translation.x,
            up_covered.translation.y,
            transform_to.translation.z,
        ),
        rotation: seat_rotation * Quat::from_rotation_x(std::f32::consts::PI),
        scale: Vec3::one(),
    };

    transform
        .ease_to(
            up_covered,
            EaseMethod::Linear,
            EasingType::Once {
                duration: std::time::Duration::from_millis(500),
            },
        )
        .ease_to(
            up_hand_covered,
            EaseFunction::QuadraticOut,
            EasingType::Once {
                duration: std::time::Duration::from_millis(1000),
            },
        )
        .ease_to(
            transform_to,
            EaseFunction::QuadraticOut,
            EasingType::Once {
                duration: std::time::Duration::from_millis(500),
            },
        )
}
