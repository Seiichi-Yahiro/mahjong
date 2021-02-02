use crate::LoadingAssets;
use bevy::prelude::*;

pub struct UiAssetData {
    font: Handle<Font>,
    transparent: Handle<ColorMaterial>,
    button_mat: Handle<ColorMaterial>,
    button_hovered_mat: Handle<ColorMaterial>,
}

impl UiAssetData {
    pub fn get_font(&self) -> Handle<Font> {
        self.font.clone()
    }

    pub fn get_transparent(&self) -> Handle<ColorMaterial> {
        self.transparent.clone()
    }

    pub fn get_button_mat(&self) -> Handle<ColorMaterial> {
        self.button_mat.clone()
    }

    pub fn get_button_hovered_mat(&self) -> Handle<ColorMaterial> {
        self.button_hovered_mat.clone()
    }
}

pub fn load_ui_asset_data_system(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let font = asset_server.load("fonts/FiraSans-Regular.ttf");
    loading_assets.0.push(font.id);

    let transparent = materials.add(Color::NONE.into());
    let button_mat = materials.add(Color::hex("D0CFC1").unwrap().into());
    let button_hovered_mat = materials.add(Color::hex("01B677").unwrap().into());

    commands.insert_resource(UiAssetData {
        font,
        transparent,
        button_mat,
        button_hovered_mat,
    });
}

pub fn create_ui_camera_system(commands: &mut Commands) {
    commands.spawn(CameraUiBundle::default());
}
