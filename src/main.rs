mod camera;
mod clamped_value;
mod solitaire;
mod table;
mod tiles;

use crate::solitaire::grid::{GridPos, TileGridSet};
use bevy::asset::{HandleId, LoadState};
use bevy::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_mod_picking::PickingPlugin;

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
        .add_startup_system(create_ui_camera_system.system())
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
                .with_enter_stage(
                    GameState::Editor,
                    SystemStage::parallel()
                        .with_system(solitaire::editor::create_placeable_tile_system.system())
                        .with_system(solitaire::editor::create_ui_system.system()),
                )
                .with_update_stage(
                    GameState::Editor,
                    Schedule::default()
                        .with_stage(
                            "1",
                            SystemStage::serial()
                                .with_system(solitaire::editor::move_placeable_tile_system.system())
                                .with_system(solitaire::editor::is_placeable_system.system()),
                        )
                        .with_stage(
                            "2",
                            SystemStage::parallel()
                                .with_system(
                                    solitaire::editor::color_placeable_tile_system.system(),
                                )
                                .with_system(solitaire::editor::place_tile_system.system())
                                .with_system(solitaire::editor::exit_editor_system.system())
                                .with_system(camera::camera_movement_system.system()),
                        )
                        .with_stage(
                            "3",
                            SystemStage::serial()
                                .with_system(solitaire::editor::undo_system.system())
                                .with_system(
                                    solitaire::editor::update_remaining_tiles_text_system.system(),
                                )
                                .with_system(solitaire::editor::save_level_system.system()),
                        ),
                )
                .with_exit_stage(
                    GameState::Editor,
                    SystemStage::single(solitaire::editor::clean_up_system.system()),
                ),
        )
        .run();
}

fn create_ui_camera_system(commands: &mut Commands) {
    commands.spawn(CameraUiBundle::default());
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
            state.set_next(GameState::Editor).unwrap();
        }
        LoadState::Failed => {
            panic!("Failed to load assets!");
        }
    };
}
