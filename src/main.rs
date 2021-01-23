mod player;
mod tiles;
mod wall;

use crate::player::Players;
use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;
use bevy_easings::EasingsPlugin;

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
        .add_plugin(EasingsPlugin)
        .add_resource(Players::new())
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
                    Schedule::default().with_stage(
                        "setup_game",
                        SystemStage::parallel()
                            .with_system(wall::build_wall_system.system())
                            .with_system(player::draw_hand_system.system()),
                    ),
                )
                .with_update_stage(
                    GameState::Play,
                    SystemStage::parallel()
                        .with_system(wall::Doras::reveal_system.system())
                        .with_system(player::Players::draw_tile_system.system()),
                ),
        )
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    commands.spawn(PbrBundle {
        mesh: asset_server.load("mesh/table.gltf#Mesh0/Primitive0"),
        material: materials.add(StandardMaterial::from(
            asset_server.load("textures/table.png"),
        )),
        ..Default::default()
    });
}
