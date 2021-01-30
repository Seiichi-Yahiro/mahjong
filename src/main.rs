mod editor;
mod table;
mod tiles;

use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;
use bevy_easings::EasingsPlugin;
use bevy_mod_picking::{PickSource, PickingPlugin};

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    Loading,
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
        .add_resource(State::new(GameState::Loading))
        .add_startup_system(table::load_table_asset_data_system.system())
        .add_startup_system(tiles::load_tile_asset_data_system.system())
        .add_stage_after(
            stage::UPDATE,
            "game_state",
            StateStage::<GameState>::default()
                .with_update_stage(
                    GameState::Loading,
                    SystemStage::single(tiles::blend_tile_textures_system.system()),
                )
                .with_exit_stage(
                    GameState::Loading,
                    SystemStage::parallel()
                        .with_system(setup.system())
                        .with_system(table::spawn_table_system.system()),
                )
                .with_enter_stage(
                    GameState::Editor,
                    SystemStage::parallel()
                        .with_system(editor::create_placeable_tile_system.system()),
                )
                .with_update_stage(
                    GameState::Editor,
                    SystemStage::parallel().with_system(editor::place_tile_system.system()),
                ),
        )
        .run();
}

fn setup(commands: &mut Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.2, 0.001))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::unit_y()),
            perspective_projection: PerspectiveProjection {
                near: 0.01,
                far: 10.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(PickSource::default());

    commands.spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 4.0)),
        ..Default::default()
    });
}
