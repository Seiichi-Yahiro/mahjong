mod player;
mod tiles;
mod wall;

use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    Loading,
    Play,
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Mahjong".to_string(),
            ..Default::default()
        })
        .add_resource(Msaa { samples: 8 })
        .add_plugins(DefaultPlugins)
        .add_resource(State::new(GameState::Loading))
        .add_startup_system(tiles::load_tile_asset_data_system.system())
        .add_stage_after(
            stage::UPDATE,
            "game_state",
            StateStage::<GameState>::default()
                .with_update_stage(
                    GameState::Loading,
                    SystemStage::single(tiles::create_materials_system.system()),
                )
                .with_exit_stage(
                    GameState::Loading,
                    Schedule::default()
                        .with_stage(
                            "setup_game",
                            SystemStage::parallel()
                                .with_system(player::setup_players_system.system())
                                .with_system(wall::build_wall_system.system()),
                        )
                        .with_stage(
                            "split_dead_wall",
                            SystemStage::single(wall::split_dead_wall_system.system()),
                        ),
                ),
        )
        .add_startup_system(setup.system())
        .run();
}

fn setup(commands: &mut Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.7, 0.7))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::unit_y()),
        perspective_projection: PerspectiveProjection {
            near: 0.01,
            far: 10.0,
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 4.0)),
        ..Default::default()
    });
}
