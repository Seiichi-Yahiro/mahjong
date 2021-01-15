use crate::tiles::{Tile, TileAssetData};
use bevy::prelude::*;

const TILE_LENGTH: usize = 17;
const TILES_PER_SIDE: usize = TILE_LENGTH * 2;

pub fn build_wall_system(commands: &mut Commands, tile_asset_data: Res<TileAssetData>) {
    Tile::new_set(false)
        .into_iter()
        .enumerate()
        .for_each(|(index, tile)| {
            let transform = calculate_wall_transform_from_index(index);
            let rotation = Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI));
            let pbr = tile_asset_data.new_pbr(tile, transform * rotation);
            commands.spawn(pbr);
        });
}

fn calculate_wall_transform_from_index(index: usize) -> Transform {
    let half_wall_length = (TILE_LENGTH as f32 / 2.0) * TileAssetData::WIDTH;
    let z = half_wall_length + TileAssetData::WIDTH;

    let x_index = TILE_LENGTH - (index % TILES_PER_SIDE) / 2;
    let x = x_index as f32 * TileAssetData::WIDTH - TileAssetData::WIDTH / 2.0 - half_wall_length;

    let y_index = (index + 1) % 2;
    let y = y_index as f32 * TileAssetData::HEIGHT + TileAssetData::HEIGHT / 2.0;

    let rotation = Transform::from_rotation(Quat::from_rotation_y(
        (index / TILES_PER_SIDE) as f32 * -std::f32::consts::FRAC_PI_2,
    ));

    let translation = Transform::from_translation(Vec3::new(x, y, z));

    rotation * translation
}
