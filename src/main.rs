mod grid;
mod plugins;

use crate::grid::Grid;
use crate::plugins::assets::background::BackgroundAssetData;
use crate::plugins::assets::tiles::asset::{TileAssetData, TileMaterial};
use crate::plugins::assets::tiles::honor::Dragon;
use bevy::log::LogPlugin;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    AssetLoading,
    Menu,
    Editor,
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "Mahjong".to_string(),
                        ..default()
                    },
                    ..default()
                })
                .set(LogPlugin {
                    filter: "info,wgpu_core=warn,wgpu_hal=error,mahjong=debug".into(),
                    level: bevy::log::Level::DEBUG,
                }),
        )
        .add_startup_system(setup_camera)
        .add_startup_system(setup_light)
        .add_state(AppState::AssetLoading)
        .insert_resource(Grid::new())
        .add_plugin(plugins::assets::AssetsPlugin)
        .add_plugin(plugins::editor::EditorPlugin)
        .add_system_set(
            SystemSet::on_exit(AppState::AssetLoading).with_system(setup_background), //.with_system(setup_tile),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.2, 0.001)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        projection: PerspectiveProjection {
            near: 0.01,
            far: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn setup_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 32_000.0,
            shadow_projection: OrthographicProjection {
                left: -5.0,
                right: 5.0,
                bottom: -5.0,
                top: 5.0,
                near: -1.0,
                far: 5.5,
                ..Default::default()
            },
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 5.0, 0.5),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)
                * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn setup_tile(
    mut commands: Commands,
    tile_asset_data: Res<TileAssetData>,
    mut materials: ResMut<Assets<TileMaterial>>,
) {
    commands.spawn(MaterialMeshBundle {
        mesh: tile_asset_data.get_mesh(),
        material: materials.add(TileMaterial::new(&tile_asset_data, Dragon::Green.into())),
        ..default()
    });
}

#[derive(Component)]
struct Background;

fn setup_background(
    mut commands: Commands,
    background_asset_data: Res<BackgroundAssetData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = Mesh::from(shape::Plane { size: 2.0 });
    let number_of_repetitions = 25.0;
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            [number_of_repetitions, number_of_repetitions],
            [number_of_repetitions, 0.0],
            [0.0, 0.0],
            [0.0, number_of_repetitions],
        ],
    );

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(background_asset_data.get_texture()),
                perceptual_roughness: 0.8,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -TileAssetData::HEIGHT / 2.0, 0.0),
            ..default()
        })
        .insert(Background);
}
