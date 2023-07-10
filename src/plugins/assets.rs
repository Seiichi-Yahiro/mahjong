pub mod background;
pub mod fonts;
pub mod tiles;

use crate::plugins::assets::background::BackgroundAssetData;
use crate::plugins::assets::fonts::Fonts;
use crate::plugins::assets::tiles::asset::TileAssetData;
use crate::AppState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::AssetLoading).continue_to_state(AppState::Menu),
        )
        .add_collection_to_loading_state::<_, BackgroundAssetData>(AppState::AssetLoading)
        .add_collection_to_loading_state::<_, TileAssetData>(AppState::AssetLoading)
        .add_collection_to_loading_state::<_, Fonts>(AppState::AssetLoading)
        .add_plugin(background::BackgroundPlugin)
        .add_plugin(tiles::TilesPlugin);
    }
}
