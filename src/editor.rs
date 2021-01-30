use crate::table::{Table, TableAssetData};
use crate::tiles::TileAssetData;
use bevy::prelude::*;
use bevy_mod_picking::{Group, PickableMesh};

const GRID_WIDTH: f32 = TileAssetData::WIDTH / 2.0;
const GRID_HEIGHT: f32 = TileAssetData::DEPTH / 2.0;

const HALF_TILE_WIDTH: f32 = TileAssetData::WIDTH / 2.0;
const HALF_TILE_HEIGHT: f32 = TileAssetData::HEIGHT / 2.0;
const HALF_TILE_DEPTH: f32 = TileAssetData::DEPTH / 2.0;

const MAX_X: f32 = TableAssetData::INNER_SIZE / 2.0 - HALF_TILE_WIDTH;
const MAX_Z: f32 = TableAssetData::INNER_SIZE / 2.0 - HALF_TILE_DEPTH;

pub struct PlaceAbleTile;
pub struct PlacedTile;

#[derive(Debug, Copy, Clone)]
pub struct GridPos {
    x: i32,
    y: i32,
    z: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_world(Vec3 { x, y, z }: Vec3) -> Self {
        use bevy::math::*;

        let x_mouse = clamp(x + HALF_TILE_WIDTH, -MAX_X + HALF_TILE_WIDTH, MAX_X);
        let z_mouse = clamp(z + HALF_TILE_DEPTH, -MAX_Z + HALF_TILE_DEPTH, MAX_Z);

        let x_grid = (x_mouse / GRID_WIDTH).floor() as i32;
        let y_grid = 0;
        let z_grid = (z_mouse / GRID_HEIGHT).floor() as i32;

        GridPos::new(x_grid, y_grid, z_grid)
    }

    pub fn to_world(self) -> Vec3 {
        Vec3::new(
            self.x as f32 * GRID_WIDTH,
            self.y as f32 * TileAssetData::HEIGHT + HALF_TILE_HEIGHT,
            self.z as f32 * GRID_HEIGHT,
        )
    }
}

pub fn create_placeable_tile_system(
    commands: &mut Commands,
    tile_asset_data: Res<TileAssetData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = StandardMaterial {
        albedo: Color::rgba(1.0, 1.0, 1.0, 0.25),
        albedo_texture: Some(tile_asset_data.get_mesh_texture()),
        shaded: false,
    };

    let pbr = PbrBundle {
        mesh: tile_asset_data.get_mesh(),
        material: materials.add(material),
        visible: Visible {
            is_visible: true,
            is_transparent: true,
        },
        transform: Transform::from_translation(Vec3::unit_y() * TileAssetData::HEIGHT / 2.0),
        ..Default::default()
    };

    commands.spawn(pbr).with(PlaceAbleTile);
}

pub fn place_tile_system(
    commands: &mut Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    table_query: Query<&PickableMesh, With<Table>>,
    mut placeable_tile_query: Query<&mut Transform, With<PlaceAbleTile>>,
    tile_asset_data: Res<TileAssetData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let intersection = table_query
        .iter()
        .next()
        .unwrap()
        .intersection(&Group::default())
        .unwrap();

    if let Some(mouse_pos) = intersection.map(|it| it.position().clone()) {
        let mut transform = placeable_tile_query.iter_mut().next().unwrap();

        let grid_pos = GridPos::from_world(mouse_pos);
        let world_pos = grid_pos.to_world();

        transform.translation = world_pos;

        if mouse_button_input.just_pressed(MouseButton::Left) {
            info!("Spawned tile at: {:?}!", grid_pos);

            let pbr = PbrBundle {
                mesh: tile_asset_data.get_mesh(),
                material: materials.add(StandardMaterial::from(tile_asset_data.get_mesh_texture())),
                transform: *transform,
                ..Default::default()
            };

            commands.spawn(pbr).with(PlacedTile);
        }
    }
}
