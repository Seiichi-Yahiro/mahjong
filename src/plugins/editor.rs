use crate::grid::Grid3D;
use crate::plugins::assets::tiles::asset::{TileAssetData, TileMaterial};
use crate::{AppState, Background};
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy_mod_raycast::{
    DefaultRaycastingPlugin, Intersection, RaycastMesh, RaycastMethod, RaycastSource, RaycastSystem,
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DefaultRaycastingPlugin::<EditorRaycastSet>::default())
            .add_system_to_stage(
                CoreStage::First,
                update_raycast_with_cursor.before(RaycastSystem::BuildRays::<EditorRaycastSet>),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::Editor)
                    .with_system(setup_raycast)
                    .with_system(create_placeable_tile),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Editor)
                    .with_system(move_placeable_tile)
                    .with_system(place_tile.after(move_placeable_tile)),
            )
            .add_system_set(SystemSet::on_exit(AppState::Editor).with_system(clean_raycast));
    }
}

struct EditorRaycastSet;

fn setup_raycast(
    mut commands: Commands,
    camera_query: Query<Entity, With<Camera3d>>,
    background_query: Query<Entity, With<Background>>,
) {
    let camera = camera_query.get_single().unwrap();
    commands
        .entity(camera)
        .insert(RaycastSource::<EditorRaycastSet>::new_transform_empty());

    let background = background_query.get_single().unwrap();
    commands
        .entity(background)
        .insert(RaycastMesh::<EditorRaycastSet>::default());
}

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RaycastSource<EditorRaycastSet>>,
) {
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in &mut query {
        pick_source.cast_method = RaycastMethod::Screenspace(cursor_position);
    }
}

fn clean_raycast(
    mut commands: Commands,
    camera_query: Query<Entity, With<Camera3d>>,
    background_query: Query<Entity, With<Background>>,
) {
    let camera = camera_query.get_single().unwrap();
    commands
        .entity(camera)
        .remove::<RaycastSource<EditorRaycastSet>>();

    let background = background_query.get_single().unwrap();
    commands
        .entity(background)
        .remove::<RaycastMesh<EditorRaycastSet>>();
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
        .insert(NotShadowCaster)
        .insert(NotShadowReceiver);
}

fn move_placeable_tile(
    mut placeable_tile_query: Query<&mut Transform, With<PlaceableTile>>,
    intersections: Query<&Intersection<EditorRaycastSet>>,
    grid: Res<Grid3D>,
) {
    for intersection in intersections.iter() {
        if let Some(intersection_pos) = intersection.position() {
            let mut transform = placeable_tile_query.get_single_mut().unwrap();
            let bias = Vec3::new(0.0, 0.0001, 0.0);
            transform.translation = grid.snap_world_pos_to_grid(*intersection_pos + bias);
        }
    }
}

fn place_tile(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    tile_asset_data: Res<TileAssetData>,
    mut materials: ResMut<Assets<TileMaterial>>,
    placeable_tile_query: Query<&Transform, With<PlaceableTile>>,
    mut grid: ResMut<Grid3D>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    let transform = placeable_tile_query.get_single().unwrap();

    if !grid.insert_from_world(transform.translation) {
        return;
    }

    let material_mesh = MaterialMeshBundle {
        mesh: tile_asset_data.get_mesh(),
        material: materials.add(TileMaterial::new_without_cover(&tile_asset_data)),
        transform: *transform,
        ..default()
    };

    commands
        .spawn(material_mesh)
        .insert(RaycastMesh::<EditorRaycastSet>::default());
}
