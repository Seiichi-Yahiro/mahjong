mod camera;
mod clamped_value;
mod menu;
mod solitaire;
mod table;
mod tiles;

use crate::menu::MenuStateStagePlugin;
use crate::solitaire::editor::EditorStateStagePlugin;
use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::solitaire::play::PlayStateStagePlugin;
use crate::table::TableAssetData;
use crate::tiles::TileAssetData;
use bevy::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::{InteractablePickingPlugin, PickingPlugin};

pub trait StateStagePlugin<S> {
    fn build(&self, state_stage: &mut StateStage<S>);
}

trait StateStageExt<S> {
    fn add_plugin<P: StateStagePlugin<S>>(self, plugin: P) -> Self;
}

impl<S> StateStageExt<S> for StateStage<S> {
    fn add_plugin<P: StateStagePlugin<S>>(mut self, plugin: P) -> Self {
        plugin.build(&mut self);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    Menu,
    Play,
    Editor,
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Mahjong".to_string(),
            ..Default::default()
        })
        .add_resource(Msaa { samples: 8 })
        .add_plugins(DefaultPlugins)
        .add_plugin(EasingsPlugin)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(EguiPlugin)
        .init_resource::<TableAssetData>()
        .init_resource::<TileAssetData>()
        .register_type::<GridPos>()
        .add_resource(TileGridSet::new())
        .add_resource(State::new(GameState::Menu))
        .add_startup_system(create_light_system.system())
        .add_startup_system(camera::create_camera_system.system())
        .add_startup_system(table::spawn_table_system.system())
        .add_stage_after(
            stage::UPDATE,
            "game_state",
            StateStage::<GameState>::default()
                .add_plugin(MenuStateStagePlugin)
                .add_plugin(PlayStateStagePlugin)
                .add_plugin(EditorStateStagePlugin),
        )
        .add_system_to_stage(stage::POST_UPDATE, tiles::add_tile_material_system.system())
        .run();
}

fn create_light_system(commands: &mut Commands) {
    commands.spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 4.0)),
        ..Default::default()
    });
}
