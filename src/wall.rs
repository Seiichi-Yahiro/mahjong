use crate::tiles::{Tile, TileAssetData};
use bevy::prelude::*;
use rand::Rng;
use std::ops::Range;

const STACK_SIZE: usize = 2;
const STACKS_PER_SIDE: usize = 17;
const TILES_PER_SIDE: usize = STACKS_PER_SIDE * 2;
const TOTAL_TILES: usize = TILES_PER_SIDE * 4;

const STACKS_IN_DEAD_WALL: usize = 7;

pub struct Index(pub usize);
pub struct Wall;
pub struct LiveWall;
pub struct DeadWall;

pub fn build_wall_system(commands: &mut Commands, tile_asset_data: Res<TileAssetData>) {
    Tile::new_set(false)
        .into_iter()
        .enumerate()
        .for_each(|(index, tile)| {
            let transform = calculate_wall_transform_from_index(index);
            let rotation = Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI));
            let pbr = tile_asset_data.new_pbr(tile, transform * rotation);
            commands
                .spawn(pbr)
                .with(Index(index))
                .with(Wall)
                .with(LiveWall);
        });
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

pub fn split_dead_wall_system(
    commands: &mut Commands,
    wall_query: Query<(Entity, &Index), With<Wall>>,
) {
    let dice = rand::thread_rng().gen_range(2..=12);
    let dead_wall_range = calculate_dead_wall_range(dice);

    for (entity, Index(index)) in wall_query.iter() {
        if !dead_wall_range.contains(index) {
            continue;
        }

        commands.remove_one::<LiveWall>(entity);
        commands.insert_one(entity, DeadWall);
    }
}

fn calculate_dead_wall_range(dice: usize) -> Range<usize> {
    let side_offset = ((dice - 1) % 4) * TILES_PER_SIDE;
    let end_dead_wall_index = side_offset + dice * STACK_SIZE;
    let begin_dead_wall_index =
        (end_dead_wall_index + TOTAL_TILES - STACKS_IN_DEAD_WALL * STACK_SIZE) % TOTAL_TILES;

    begin_dead_wall_index..end_dead_wall_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_wall_range_from_negative_wrap() {
        let range = calculate_dead_wall_range(5);
        // range.contains works also with start > end
        assert_eq!(132..10, range);
    }

    #[test]
    fn dead_wall_range_from_highest_dice() {
        let range = calculate_dead_wall_range(12);
        assert_eq!(112..126, range);
    }
}
