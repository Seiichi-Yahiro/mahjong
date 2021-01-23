use crate::player::Players;
use crate::tiles::{Tile, TileAssetData, Wind};
use bevy::prelude::*;
use bevy_easings::{Ease, EaseFunction, EaseMethod, EasingChainComponent, EasingType};
use rand::prelude::SliceRandom;
use rand::Rng;
use std::collections::VecDeque;

const STACK_SIZE: usize = 2;
const STACKS_PER_SIDE: usize = 17;
const TILES_PER_SIDE: usize = STACKS_PER_SIDE * 2;
const TOTAL_TILES: usize = TILES_PER_SIDE * 4;

const TILES_IN_DEAD_WALL: usize = 7 * STACK_SIZE;

#[derive(Debug, Copy, Clone)]
struct TileEntity {
    tile: Tile,
    entity: Entity,
}

pub struct Wall {
    living_tiles: VecDeque<TileEntity>,
    rest: Vec<TileEntity>,
}

pub struct Kans {
    tiles: Vec<TileEntity>,
}

impl Kans {
    fn new(tiles: Vec<TileEntity>) -> Self {
        Self { tiles }
    }
}

pub struct RevealDora;
pub struct DoraTile;

pub struct Doras {
    tiles: Vec<TileEntity>,
    revealed: usize,
}

impl Doras {
    fn new(tiles: Vec<TileEntity>) -> Self {
        Self { revealed: 0, tiles }
    }

    fn reveal_dora(&mut self) -> Result<TileEntity, &'static str> {
        if self.revealed == 4 {
            Err("Already revealed 4 dora tiles!")
        } else {
            self.revealed += 1;
            Ok(self.tiles[(self.revealed - 1) * 2])
        }
    }

    pub fn reveal_system(
        commands: &mut Commands,
        mut doras: ResMut<Doras>,
        events: Query<Entity, With<RevealDora>>,
        dora_query: Query<(Entity, &Transform), With<DoraTile>>,
    ) {
        for entity in events.iter() {
            match doras.reveal_dora() {
                Ok(tile_entity) => {
                    match dora_query
                        .iter()
                        .find(|(entity, _)| *entity == tile_entity.entity)
                    {
                        None => {
                            error!("Could not find dora tile to reveal!")
                        }
                        Some((entity, transform)) => {
                            let flip_animation = calculate_tile_flip_animation(*transform);
                            commands.insert_one(entity, flip_animation);
                        }
                    }
                }
                Err(err) => {
                    warn!("{}", err)
                }
            }
            commands.despawn(entity);
        }
    }
}

pub fn build_wall_system(
    commands: &mut Commands,
    players: Res<Players>,
    tile_asset_data: Res<TileAssetData>,
) {
    let mut living_tiles: VecDeque<TileEntity> = {
        let mut tiles = Tile::new_set(false);
        tiles.shuffle(&mut rand::thread_rng());

        let living_offset = calculate_living_tiles_offset(players.dealer().seat);

        tiles
            .into_iter()
            .enumerate()
            .map(|(index, tile)| {
                let transform = calculate_wall_transform_from_index(
                    TOTAL_TILES + index + living_offset - TILES_IN_DEAD_WALL,
                );
                let rotation =
                    Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI));

                let pbr = tile_asset_data.new_pbr(tile, transform * rotation);

                let entity = commands.spawn(pbr).with(tile).current_entity().unwrap();

                TileEntity { tile, entity }
            })
            .collect()
    };

    let rest = living_tiles.drain(0..1 * STACK_SIZE).rev().collect();

    let doras = {
        let mut dora_tiles = living_tiles.drain(0..4 * STACK_SIZE).rev().collect();
        swap_neighbors(&mut dora_tiles);

        dora_tiles.iter().for_each(|it| {
            commands.insert_one(it.entity, DoraTile);
        });

        Doras::new(dora_tiles)
    };

    let kans = {
        let mut kan_tiles = living_tiles.drain(0..2 * STACK_SIZE).rev().collect();
        swap_neighbors(&mut kan_tiles);
        Kans::new(kan_tiles)
    };

    let wall = Wall { living_tiles, rest };

    commands.insert_resource(wall);
    commands.insert_resource(doras);
    commands.insert_resource(kans);

    commands.spawn((RevealDora,));
}

fn swap_neighbors(v: &mut Vec<TileEntity>) {
    for i in (0..v.len()).step_by(2) {
        v.swap(i, i + 1);
    }
}

fn calculate_wall_transform_from_index(index: usize) -> Transform {
    let half_wall_length = (STACKS_PER_SIDE as f32 / 2.0) * TileAssetData::WIDTH;
    let z = half_wall_length + TileAssetData::WIDTH;

    let x_index = STACKS_PER_SIDE - (index % TILES_PER_SIDE) / 2;
    let x = x_index as f32 * TileAssetData::WIDTH - TileAssetData::WIDTH / 2.0 - half_wall_length;

    let y_index = (index + 1) % STACK_SIZE;
    let y = y_index as f32 * TileAssetData::HEIGHT + TileAssetData::HEIGHT / 2.0;

    let rotation = Transform::from_rotation(Quat::from_rotation_y(
        (index / TILES_PER_SIDE) as f32 * -std::f32::consts::FRAC_PI_2,
    ));

    let translation = Transform::from_translation(Vec3::new(x, y, z));

    rotation * translation
}

fn calculate_living_tiles_offset(seat: Wind) -> usize {
    let dice = rand::thread_rng().gen_range(2..=12);
    info!("Rolled: {:?}!", dice);

    let side = match seat {
        Wind::East => 1,
        Wind::South => 0,
        Wind::West => 3,
        Wind::North => 2,
    };

    let side_offset = (((4 + side) - (dice - 1) % 4) % 4) * TILES_PER_SIDE;

    side_offset + dice * STACK_SIZE
}

fn calculate_tile_flip_animation(transform: Transform) -> EasingChainComponent<Transform> {
    let transform_middle = Transform {
        translation: Vec3::new(
            0.0,
            -(TileAssetData::DEPTH / 2.0 - TileAssetData::HEIGHT / 2.0),
            0.0,
        ),
        rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        scale: Vec3::one(),
    };

    let transform_to = Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI));

    transform
        .ease_to(
            transform * transform_middle,
            EaseFunction::CircularOut,
            EasingType::Once {
                duration: std::time::Duration::from_millis(500),
            },
        )
        .ease_to(
            transform * transform_to,
            EaseFunction::CircularIn,
            EasingType::Once {
                duration: std::time::Duration::from_millis(500),
            },
        )
}
