use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::table::Table;
use crate::tiles::TileAssetData;
use bevy::prelude::*;
use bevy_mod_picking::{Group, PickableMesh};

const ALPHA_VALUE: f32 = 0.3;

fn white_color() -> Color {
    Color::rgba(1.0, 1.0, 1.0, ALPHA_VALUE)
}

fn red_color() -> Color {
    Color::rgba(1.0, 0.0, 0.0, ALPHA_VALUE)
}

pub struct PlaceAbleTile;
pub struct PlacedTile;

pub fn create_placeable_tile_system(
    commands: &mut Commands,
    tile_asset_data: Res<TileAssetData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = StandardMaterial {
        albedo: white_color(),
        albedo_texture: Some(tile_asset_data.get_mesh_texture()),
        shaded: true,
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

    commands
        .spawn(pbr)
        .with(PlaceAbleTile)
        .with(GridPos::default());
}

pub fn move_placeable_tile_system(
    tile_grid_set: Res<TileGridSet>,
    table_query: Query<&PickableMesh, With<Table>>,
    mut placeable_tile_query: Query<(&mut Transform, &mut GridPos), With<PlaceAbleTile>>,
) {
    let intersection = table_query
        .iter()
        .next()
        .unwrap()
        .intersection(&Group::default())
        .unwrap();

    if let Some(mouse_pos) = intersection.map(|it| it.position().clone()) {
        for (mut transform, mut grid_pos) in placeable_tile_query.iter_mut() {
            *grid_pos = {
                let mut new_grid_pos = GridPos::from_world(mouse_pos);

                while tile_grid_set.is_overlapping(new_grid_pos) {
                    new_grid_pos = GridPos {
                        y: new_grid_pos.y + 1,
                        ..new_grid_pos
                    }
                }

                new_grid_pos
            };

            transform.translation = grid_pos.to_world();
        }
    }
}

pub fn color_placeable_tile_system(
    tile_grid_set: Res<TileGridSet>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    placeable_tile_query: Query<(&Handle<StandardMaterial>, &GridPos), With<PlaceAbleTile>>,
) {
    for (material_handle, grid_pos) in placeable_tile_query.iter() {
        let color = if tile_grid_set.is_supported_from_below(*grid_pos) {
            white_color()
        } else {
            red_color()
        };

        materials.get_mut(material_handle).unwrap().albedo = color;
    }
}

pub fn place_tile_system(
    commands: &mut Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    tile_asset_data: Res<TileAssetData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tile_grid_set: ResMut<TileGridSet>,
    table_query: Query<&PickableMesh, With<Table>>,
    placeable_tile_query: Query<&GridPos, With<PlaceAbleTile>>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    let is_hovering_table = table_query
        .iter()
        .next()
        .unwrap()
        .intersection(&Group::default())
        .unwrap()
        .is_some();

    let grid_pos = *placeable_tile_query.iter().next().unwrap();

    if is_hovering_table && tile_grid_set.try_insert(grid_pos) {
        info!("Spawned tile at: {:?}!", grid_pos);

        let pbr = PbrBundle {
            mesh: tile_asset_data.get_mesh(),
            material: materials.add(StandardMaterial::from(tile_asset_data.get_mesh_texture())),
            transform: Transform::from_translation(grid_pos.to_world()),
            ..Default::default()
        };

        commands.spawn(pbr).with(PlacedTile).with(grid_pos);
    }
}
