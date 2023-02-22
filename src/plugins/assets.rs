pub mod background;
pub mod tiles;

use crate::plugins::assets::background::BackgroundAssetData;
use crate::plugins::assets::tiles::asset::TileAssetData;
use crate::AppState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .continue_to_state(AppState::Editor)
                .with_collection::<BackgroundAssetData>()
                .with_collection::<TileAssetData>(),
        )
        .add_plugin(background::BackgroundPlugin)
        .add_plugin(tiles::TilesPlugin);
    }
}
