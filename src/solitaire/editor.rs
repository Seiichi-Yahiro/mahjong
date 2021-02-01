use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::table::Table;
use crate::tiles::{TileAssetData, NUMBER_OF_TILES_WITH_BONUS};
use crate::{camera, GameState, StateStagePlugin};
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy_mod_picking::{Group, PickableMesh};

pub struct EditorStateStagePlugin;

impl StateStagePlugin<GameState> for EditorStateStagePlugin {
    fn build(&self, state_stage: &mut StateStage<GameState>) {
        let state = GameState::Editor;

        state_stage
            .set_enter_stage(
                state,
                SystemStage::parallel()
                    .with_system(create_placeable_tile_system.system())
                    .with_system(create_ui_system.system()),
            )
            .set_update_stage(
                state,
                Schedule::default()
                    .with_stage(
                        "1",
                        SystemStage::serial()
                            .with_system(move_placeable_tile_system.system())
                            .with_system(is_placeable_system.system()),
                    )
                    .with_stage(
                        "2",
                        SystemStage::parallel()
                            .with_system(color_placeable_tile_system.system())
                            .with_system(place_tile_system.system())
                            .with_system(exit_editor_system.system())
                            .with_system(camera::camera_movement_system.system()),
                    )
                    .with_stage(
                        "3",
                        SystemStage::serial()
                            .with_system(undo_system.system())
                            .with_system(update_remaining_tiles_text_system.system())
                            .with_system(save_level_system.system()),
                    ),
            )
            .set_exit_stage(state, SystemStage::single(clean_up_system.system()));
    }
}

struct EditorEntity;

const ALPHA_VALUE: f32 = 0.3;

fn white_color() -> Color {
    Color::rgba(1.0, 1.0, 1.0, ALPHA_VALUE)
}

fn red_color() -> Color {
    Color::rgba(1.0, 0.0, 0.0, ALPHA_VALUE)
}

struct PlaceAbleTile(bool);
struct PlacedTile;

fn create_placeable_tile_system(
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
        .with(EditorEntity)
        .with(PlaceAbleTile(true))
        .with(GridPos::default());
}

fn move_placeable_tile_system(
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

fn is_placeable_system(
    tile_grid_set: Res<TileGridSet>,
    mut placeable_tile_query: Query<(&mut PlaceAbleTile, &GridPos)>,
) {
    for (mut placeable_tile, &grid_pos) in placeable_tile_query.iter_mut() {
        placeable_tile.0 = tile_grid_set.len() < NUMBER_OF_TILES_WITH_BONUS as usize
            && !tile_grid_set.is_overlapping(grid_pos)
            && tile_grid_set.is_supported_from_below(grid_pos);
    }
}

fn color_placeable_tile_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    placeable_tile_query: Query<
        (&Handle<StandardMaterial>, &PlaceAbleTile),
        Changed<PlaceAbleTile>,
    >,
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

fn place_tile_system(
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

            commands
                .spawn(pbr)
                .with(EditorEntity)
                .with(PlacedTile)
                .with(grid_pos);
        }
    }
}

struct PlacementOrder(Vec<(Entity, GridPos)>);

impl Default for PlacementOrder {
    fn default() -> Self {
        Self(Vec::with_capacity(NUMBER_OF_TILES_WITH_BONUS as usize))
    }
}

fn undo_system(
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

struct RemainingTilesText;

fn create_ui_system(commands: &mut Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Regular.ttf");

    commands
        .spawn(TextBundle {
            node: Default::default(),
            style: Style {
                align_self: AlignSelf::FlexStart,
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(15.0),
                    top: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                value: NUMBER_OF_TILES_WITH_BONUS.to_string(),
                font,
                style: TextStyle {
                    font_size: 50.0,
                    color: Color::WHITE,
                    alignment: TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    },
                },
            },
            ..Default::default()
        })
        .with(EditorEntity)
        .with(RemainingTilesText);
}

fn update_remaining_tiles_text_system(
    tile_grid_set: ChangedRes<TileGridSet>,
    mut text_query: Query<&mut Text, With<RemainingTilesText>>,
) {
    for mut text in text_query.iter_mut() {
        text.value = (NUMBER_OF_TILES_WITH_BONUS as usize - tile_grid_set.len()).to_string();
    }
}

fn save_level_system(world: &mut World, resources: &mut Resources) {
    use std::fs::File;
    use std::io::prelude::*;

    let key_input = resources.get::<Input<KeyCode>>().unwrap();
    let tile_grid_set = resources.get::<TileGridSet>().unwrap();

    if !key_input.just_pressed(KeyCode::S)
        || tile_grid_set.len() < NUMBER_OF_TILES_WITH_BONUS as usize
    {
        return;
    }

    info!("Saving scene!");

    let mut custom_world = World::new();

    for grid_pos in world
        .query_filtered::<&GridPos, With<PlacedTile>>()
        .into_iter()
    {
        custom_world.spawn((*grid_pos,));
    }

    let type_registry = resources.get::<TypeRegistry>().unwrap();
    let scene = DynamicScene::from_world(&custom_world, &type_registry);

    match scene.serialize_ron(&type_registry) {
        Err(err) => {
            error!("Failed to serialize scene: {:?}!", err);
        }
        Ok(level) => match File::create("assets/scenes/levels/custom/custom_level.scn.ron") {
            Err(err) => {
                error!("Failed to create file: {:?}!", err);
            }
            Ok(mut file) => match file.write_all(level.as_bytes()) {
                Err(err) => {
                    error!("Failed to write level to file {:?}!", err);
                }
                Ok(_) => {
                    info!("Scene successfully saved!");
                }
            },
        },
    }
}

fn exit_editor_system(key_input: Res<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    if key_input.just_pressed(KeyCode::Escape) {
        state.set_next(GameState::Menu).unwrap()
    }
}

fn clean_up_system(
    commands: &mut Commands,
    mut tile_grid_set: ResMut<TileGridSet>,
    query: Query<Entity, With<EditorEntity>>,
) {
    tile_grid_set.clear();

    for entity in query.iter() {
        commands.despawn_recursive(entity);
    }
}
