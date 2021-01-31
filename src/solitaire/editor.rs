use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::table::Table;
use crate::tiles::{TileAssetData, NUMBER_OF_TILES_WITH_BONUS};
use bevy::prelude::*;
use bevy_mod_picking::{Group, PickableMesh};

const ALPHA_VALUE: f32 = 0.3;

fn white_color() -> Color {
    Color::rgba(1.0, 1.0, 1.0, ALPHA_VALUE)
}

fn red_color() -> Color {
    Color::rgba(1.0, 0.0, 0.0, ALPHA_VALUE)
}

pub struct PlaceAbleTile(bool);
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
        .with(PlaceAbleTile(true))
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

pub fn is_placeable_system(
    tile_grid_set: Res<TileGridSet>,
    mut placeable_tile_query: Query<(&mut PlaceAbleTile, &GridPos)>,
) {
    for (mut placeable_tile, &grid_pos) in placeable_tile_query.iter_mut() {
        placeable_tile.0 = tile_grid_set.len() < NUMBER_OF_TILES_WITH_BONUS as usize
            && !tile_grid_set.is_overlapping(grid_pos)
            && tile_grid_set.is_supported_from_below(grid_pos);
    }
}

pub fn color_placeable_tile_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    placeable_tile_query: Query<(&Handle<StandardMaterial>, &PlaceAbleTile)>,
) {
    for (material_handle, &PlaceAbleTile(is_placeable)) in placeable_tile_query.iter() {
        let color = if is_placeable {
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
    placeable_tile_query: Query<(&PlaceAbleTile, &GridPos)>,
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

    if !is_hovering_table {
        return;
    }

    for (&PlaceAbleTile(is_placeable), &grid_pos) in placeable_tile_query.iter() {
        if is_placeable {
            tile_grid_set.insert(grid_pos);
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
}

pub struct PlacementOrder(Vec<(Entity, GridPos)>);

impl Default for PlacementOrder {
    fn default() -> Self {
        Self(Vec::with_capacity(NUMBER_OF_TILES_WITH_BONUS as usize))
    }
}

pub fn undo_system(
    commands: &mut Commands,
    mut state: Local<PlacementOrder>,
    keyboard_input: Res<Input<KeyCode>>,
    mut tile_grid_set: ResMut<TileGridSet>,
    query: Query<(Entity, &GridPos), Added<PlacedTile>>,
) {
    for (entity, &grid_pos) in query.iter() {
        state.0.push((entity, grid_pos));
    }

    if keyboard_input.just_pressed(KeyCode::U) {
        if let Some((entity, grid_pos)) = state.0.pop() {
            tile_grid_set.remove(&grid_pos);
            commands.despawn(entity);
        }
    }
}
