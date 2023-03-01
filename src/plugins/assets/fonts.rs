use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct Fonts {
    #[asset(path = "fonts/Roboto-Regular.ttf")]
    roboto: Handle<Font>,
}

impl Fonts {
    pub fn roboto(&self) -> Handle<Font> {
        self.roboto.clone()
    }
}
