mod camera;
mod clamped_value;
mod menu;
mod solitaire;
mod table;
mod tiles;

use crate::menu::MenuStatePlugin;
use crate::solitaire::editor::EditorStatePlugin;
use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::solitaire::play::PlayStatePlugin;
use crate::table::TableAssetData;
use crate::tiles::TileAssetData;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::{InteractablePickingPlugin, PickingPlugin};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameState {
    Menu,
    Play,
    Editor,
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Mahjong".to_string(),
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 8 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(EguiPlugin)
        .init_resource::<TableAssetData>()
        .init_resource::<TileAssetData>()
        .register_type::<GridPos>()
        .insert_resource(TileGridSet::new())
        .add_state(GameState::Menu)
        .add_startup_system(create_light_system.system())
        .add_startup_system(camera::create_camera_system.system())
        .add_startup_system(table::spawn_table_system.system())
        .add_plugin(MenuStatePlugin)
        .add_plugin(PlayStatePlugin)
        .add_plugin(EditorStatePlugin)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            tiles::add_tile_material_system.system(),
        )
        .run();
}

fn create_light_system(mut commands: Commands) {
    commands.spawn().insert_bundle(LightBundle {
        transform: Transform::from_xyz(0.0, 5.0, 4.0),
        ..Default::default()
    });
}
