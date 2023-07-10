use crate::AppState;
use bevy::prelude::*;
use bevy::render::texture::ImageSampler;
use bevy_asset_loader::prelude::*;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(repeat_background_texture.in_schedule(OnExit(AppState::AssetLoading)));
    }
}

#[derive(AssetCollection, Resource)]
pub struct BackgroundAssetData {
    #[asset(path = "textures/background.png")]
    texture: Handle<Image>,
}

impl BackgroundAssetData {
    pub fn get_texture(&self) -> Handle<Image> {
        self.texture.clone()
    }
}

fn repeat_background_texture(
    background_asset_data: Res<BackgroundAssetData>,
    mut textures: ResMut<Assets<Image>>,
) {
    let texture = textures.get_mut(&background_asset_data.texture).unwrap();
    let mut descriptor = ImageSampler::linear_descriptor();
    descriptor.address_mode_u = bevy::render::render_resource::AddressMode::Repeat;
    descriptor.address_mode_v = bevy::render::render_resource::AddressMode::Repeat;
    texture.sampler_descriptor = ImageSampler::Descriptor(descriptor);
}
