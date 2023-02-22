pub mod asset;
pub mod bonus;
pub mod honor;
pub mod suit;
pub mod tile;

use crate::plugins::assets::tiles::asset::{reinterpret_cover_texture_as_2d_array, TileMaterial};
use crate::AppState;
use bevy::prelude::*;

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<TileMaterial>::default())
            .add_system_set(
                SystemSet::on_exit(AppState::AssetLoading)
                    .with_system(reinterpret_cover_texture_as_2d_array),
            );
    }
}

pub const NUMBER_OF_COVERS: u32 = 42;
pub const NUMBER_OF_TILE_WITHOUT_BONUS: u32 = (9 * 3 + 4 + 3) * 4;
pub const NUMBER_OF_TILES_WITH_BONUS: u32 = NUMBER_OF_TILE_WITHOUT_BONUS + 2 * 4;

pub trait EnumIter {
    fn next(self) -> Self;
}
