mod camera;
mod clamped_value;
mod menu;
mod solitaire;
mod table;
mod tiles;
mod ui;

use crate::menu::MenuStateStagePlugin;
use crate::solitaire::editor::EditorStateStagePlugin;
use crate::solitaire::grid::{GridPos, TileGridSet};
use bevy::asset::{HandleId, LoadState};
use bevy::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_mod_picking::PickingPlugin;

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
    Loading,
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
        .init_resource::<LoadingAssets>()
        .register_type::<GridPos>()
        .add_resource(TileGridSet::new())
        .add_resource(State::new(GameState::Loading))
        .add_startup_system(ui::create_ui_camera_system.system())
        .add_startup_system(ui::load_ui_asset_data_system.system())
        .add_startup_system(table::load_table_asset_data_system.system())
        .add_startup_system(tiles::load_tile_asset_data_system.system())
        .add_stage_after(
            stage::UPDATE,
            "game_state",
            StateStage::<GameState>::default()
                .with_update_stage(
                    GameState::Loading,
                    SystemStage::single(check_load_state_system.system()),
                )
                .with_exit_stage(
                    GameState::Loading,
                    SystemStage::parallel()
                        .with_system(create_light_system.system())
                        .with_system(camera::create_camera_system.system())
                        .with_system(table::spawn_table_system.system())
                        .with_system(tiles::blend_tile_textures_system.system()),
                )
                .add_plugin(MenuStateStagePlugin)
                .add_plugin(EditorStateStagePlugin),
        )
        .run();
}

fn create_light_system(commands: &mut Commands) {
    commands.spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 4.0)),
        ..Default::default()
    });
}

#[derive(Default)]
pub struct LoadingAssets(Vec<HandleId>);

fn check_load_state_system(
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut state: ResMut<State<GameState>>,
) {
    match asset_server.get_group_load_state(loading_assets.0.iter().cloned()) {
        LoadState::NotLoaded | LoadState::Loading => {
            return;
        }
        LoadState::Loaded => {
            loading_assets.0.clear();
            state.set_next(GameState::Menu).unwrap();
        }
        LoadState::Failed => {
            panic!("Failed to load assets!");
        }
    };
}
