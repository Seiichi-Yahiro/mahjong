use crate::grid::GridPos;
use crate::plugins::assets::tiles::asset::TileAssetData;
use crate::AppState;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::camera::Viewport;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Editor).with_system(create_placeable_tile),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Editor)
                .with_system(move_placeable_tile)
                .with_system(place_tile.after(move_placeable_tile)),
        );
    }
}

#[derive(Component)]
struct PlaceableTile;

fn create_placeable_tile(
    mut commands: Commands,
    tile_asset_data: Res<TileAssetData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = StandardMaterial {
        base_color: Color::rgba(1.0, 1.0, 1.0, 0.3),
        base_color_texture: Some(tile_asset_data.get_mesh_texture()),
        perceptual_roughness: 1.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
    };

    let pbr = PbrBundle {
        mesh: tile_asset_data.get_mesh(),
        material: materials.add(material),
        ..default()
    };

    commands
        .spawn(pbr)
        .insert(PlaceableTile)
        .insert(GridPos::default())
        .insert(NotShadowCaster)
        .insert(NotShadowReceiver);
}

fn move_placeable_tile(
    mut placeable_tile_query: Query<(&mut Transform, &mut GridPos), With<PlaceableTile>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    windows: Res<Windows>,
) {
    let (mut transform, mut grid_pos) = placeable_tile_query.get_single_mut().unwrap();
    let (camera_transform, camera) = camera_query.get_single().unwrap();
    let window = windows.get_primary().unwrap();

    if let Some(ray) = window
        .cursor_position()
        .and_then(|cursor_pos| camera.viewport_to_world(camera_transform, cursor_pos))
    {
        let plane_normal = Vec3::Y; // TODO get data from background
        let plane_pos = Vec3::new(0.0, -TileAssetData::HEIGHT / 2.0, 0.0);
        let lambda = (plane_pos - ray.origin).dot(plane_normal) / ray.direction.dot(plane_normal);

        let pos =
            ray.origin + ray.direction * lambda + Vec3::new(0.0, TileAssetData::HEIGHT / 2.0, 0.0);

        *grid_pos = GridPos::from_world(pos);
        transform.translation = grid_pos.to_world();
    }
}

fn place_tile(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    tile_asset_data: Res<TileAssetData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    placeable_tile_query: Query<&GridPos, With<PlaceableTile>>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    let grid_pos = placeable_tile_query.get_single().unwrap();

    let pbr = PbrBundle {
        mesh: tile_asset_data.get_mesh(),
        material: materials.add(StandardMaterial::from(tile_asset_data.get_mesh_texture())),
        transform: Transform::from_translation(grid_pos.to_world()),
        ..default()
    };

    commands.spawn(pbr).insert(*grid_pos);
}
