use crate::grid::Grid3D;
use crate::plugins::assets::tiles::asset::{TileAssetData, TileMaterial};
use crate::{AppState, Background};
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
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
            .add_system_set(SystemSet::on_exit(AppState::Editor).with_system(cleanup));
    }
}

#[derive(Component)]
struct EditorEntity;

struct EditorRaycastSet;

fn setup_raycast(
    mut commands: Commands,
    camera_query: Query<Entity, With<Camera3d>>,
    background_query: Query<&Transform, With<Background>>,
    grid: Res<Grid3D>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera = camera_query.get_single().unwrap();
    commands
        .entity(camera)
        .insert(RaycastSource::<EditorRaycastSet>::new_transform_empty());

    let background_transform = background_query.get_single().unwrap();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // + and - grid_cell_size only work because x and z have a subdivision of 1
    let min = grid.min().as_vec3() * grid.cell_size() - grid.cell_size();
    let max = grid.max().as_vec3() * grid.cell_size() + grid.cell_size();

    let top_left = Vec3::new(min.x, 0.0, min.z);
    let bottom_right = Vec3::new(max.x, 0.0, max.z);
    let bottom_left = Vec3::new(top_left.x, 0.0, bottom_right.z);
    let top_right = Vec3::new(bottom_right.x, 0.0, top_left.z);

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            top_left.to_array(),
            bottom_left.to_array(),
            top_right.to_array(),
            bottom_right.to_array(),
        ],
    );

    mesh.set_indices(Some(Indices::U32(vec![0, 1, 2, 2, 1, 3])));

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.0, 0.0, 0.0, 0.20),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, background_transform.translation.y + 0.0001, 0.0),
            ..default()
        })
        .insert(RaycastMesh::<EditorRaycastSet>::default())
        .insert(NotShadowReceiver)
        .insert(NotShadowCaster)
        .insert(EditorEntity);
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

fn cleanup(
    mut commands: Commands,
    camera_query: Query<Entity, With<Camera3d>>,
    editor_entity_query: Query<Entity, With<EditorEntity>>,
) {
    let camera = camera_query.get_single().unwrap();
    commands
        .entity(camera)
        .remove::<RaycastSource<EditorRaycastSet>>();

    for entity in editor_entity_query.iter() {
        commands.entity(entity).despawn_recursive();
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
        base_color: Color::rgba(1.0, 1.0, 1.0, 0.4),
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
        .insert(NotShadowReceiver)
        .insert(EditorEntity);
}

fn move_placeable_tile(
    mut placeable_tile_query: Query<(&mut Transform, &mut Visibility), With<PlaceableTile>>,
    intersections: Query<&Intersection<EditorRaycastSet>>,
    grid: Res<Grid3D>,
) {
    let (mut transform, mut visibility) = placeable_tile_query.get_single_mut().unwrap();

    for intersection in intersections.iter() {
        if let Some(intersection_pos) = intersection.position() {
            let bias = Vec3::new(0.0, 0.0001, 0.0);
            transform.translation = grid.snap_world_pos_to_grid(*intersection_pos + bias);
            visibility.is_visible = true;
        } else {
            visibility.is_visible = false;
        }
    }
}

fn place_tile(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    tile_asset_data: Res<TileAssetData>,
    mut materials: ResMut<Assets<TileMaterial>>,
    placeable_tile_query: Query<(&Transform, &Visibility), With<PlaceableTile>>,
    mut grid: ResMut<Grid3D>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    let (transform, visibility) = placeable_tile_query.get_single().unwrap();

    if !visibility.is_visible || !grid.insert_from_world(transform.translation) {
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
        .insert(RaycastMesh::<EditorRaycastSet>::default())
        .insert(EditorEntity);
}
