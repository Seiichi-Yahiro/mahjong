use crate::player::Players;
use crate::tiles::{Tile, TileAssetData, Wind};
use bevy::prelude::*;
use rand::prelude::SliceRandom;
use rand::Rng;
use std::collections::VecDeque;

const STACK_SIZE: usize = 2;
const STACKS_PER_SIDE: usize = 17;
const TILES_PER_SIDE: usize = STACKS_PER_SIDE * 2;
const TOTAL_TILES: usize = TILES_PER_SIDE * 4;

const TILES_IN_DEAD_WALL: usize = 7 * STACK_SIZE;

#[derive(Debug)]
struct TileEntity {
    tile: Tile,
    entity: Entity,
}

pub struct Wall {
    living_tiles: VecDeque<TileEntity>,
    kan_tiles: Vec<TileEntity>,
    dora_tiles: Vec<TileEntity>,
    rest: Vec<TileEntity>,
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
                let pbr = tile_asset_data.new_pbr(tile, transform);

                let entity = commands.spawn(pbr).with(tile).current_entity().unwrap();

                TileEntity { tile, entity }
            })
            .collect()
    };

    let rest = living_tiles.drain(0..1 * STACK_SIZE).rev().collect();

    let dora_tiles = {
        let mut dora_tiles = living_tiles.drain(0..4 * STACK_SIZE).rev().collect();
        swap_neighbors(&mut dora_tiles);
        dora_tiles
    };

    let kan_tiles = {
        let mut kan_tiles = living_tiles.drain(0..2 * STACK_SIZE).rev().collect();
        swap_neighbors(&mut kan_tiles);
        kan_tiles
    };

    let wall = Wall {
        living_tiles,
        kan_tiles,
        dora_tiles,
        rest,
    };

    commands.insert_resource(wall);
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
