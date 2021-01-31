use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::table::Table;
use crate::tiles::TileAssetData;
use bevy::prelude::*;
use bevy_mod_picking::{Group, PickableMesh};

pub struct PlaceAbleTile;
pub struct PlacedTile;

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
    mut tile_grid_set: ResMut<TileGridSet>,
) {
    let intersection = table_query
        .iter()
        .next()
        .unwrap()
        .intersection(&Group::default())
        .unwrap();

    if let Some(mouse_pos) = intersection.map(|it| it.position().clone()) {
        let mut transform = placeable_tile_query.iter_mut().next().unwrap();

        let mut grid_pos = GridPos::from_world(mouse_pos);

        while tile_grid_set.is_overlapping(grid_pos) {
            grid_pos = GridPos {
                y: grid_pos.y + 1,
                ..grid_pos
            }
        }

        let world_pos = grid_pos.to_world();

        transform.translation = world_pos;

        if mouse_button_input.just_pressed(MouseButton::Left) && tile_grid_set.try_insert(grid_pos)
        {
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
