use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::table::Table;
use crate::tiles::{TileAssetData, NUMBER_OF_TILES_WITH_BONUS};
use crate::{camera, GameState, StateStagePlugin};
use bevy::prelude::*;
use bevy::reflect::{TypeRegistry, TypeRegistryArc};
use bevy_egui::{egui, EguiContext};
use bevy_mod_picking::{Group, PickableMesh};

pub struct EditorStateStagePlugin;

impl StateStagePlugin<GameState> for EditorStateStagePlugin {
    fn build(&self, state_stage: &mut StateStage<GameState>) {
        let state = GameState::Editor;

        state_stage
            .set_enter_stage(
                state,
                SystemStage::parallel()
                    .with_system(create_ui_state_system.system())
                    .with_system(create_placeable_tile_system.system()),
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
                            .with_system(ui_system.system())
                            .with_system(color_placeable_tile_system.system())
                            .with_system(place_tile_system.system())
                            .with_system(camera::camera_movement_system.system()),
                    )
                    .with_stage("3", SystemStage::serial().with_system(undo_system.system())),
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

#[derive(Default)]
struct UiState {
    file_name: String,
    save_result: Option<Result<(), String>>,
    timer: Timer,
}

fn create_ui_state_system(commands: &mut Commands) {
    commands.insert_resource(UiState::default());
}

fn ui_system(world: &mut World, resources: &mut Resources) {
    let mut ui_state = resources.get_mut::<UiState>().unwrap();
    let mut egui_context = resources.get_mut::<EguiContext>().unwrap();
    let ctx = &mut egui_context.ctx;

    egui::SidePanel::left("side_panel", 150.0).show(ctx, |ui| {
        let placed_tiles = resources.get::<TileGridSet>().unwrap().len();

        ui.label(format!(
            "Tiles: {}/{}",
            placed_tiles, NUMBER_OF_TILES_WITH_BONUS
        ));

        ui.label("Filename:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut ui_state.file_name);

            let can_save = !ui_state.file_name.is_empty()
                && placed_tiles == NUMBER_OF_TILES_WITH_BONUS as usize;

            let button = egui::Button::new("💾").enabled(can_save);
            if ui.add(button).clicked {
                let type_registry = resources.get::<TypeRegistry>().unwrap();
                ui_state.save_result = Some(save_level(world, &ui_state.file_name, &type_registry));
                ui_state.timer = Timer::from_seconds(3.0, false);
            }
        });

        if ui_state.save_result.is_some() {
            ui_state
                .timer
                .tick(resources.get::<Time>().unwrap().delta_seconds());

            if ui_state.timer.finished() {
                ui_state.save_result = None;
            } else {
                let msg = match ui_state.save_result.as_ref().unwrap() {
                    Ok(_) => "Saved successfully!",
                    Err(err) => err,
                };

                let alpha = 1.0 - ui_state.timer.elapsed() / ui_state.timer.duration();
                let color =
                    egui::Color32::from_rgba_unmultiplied(160, 160, 160, (255.0 * alpha) as u8);
                let label = egui::Label::new(msg).text_color(color);
                ui.add(label);
            }
        }

        ui.with_layout(
            egui::Layout::bottom_up(egui::Align::Center).with_cross_justify(true),
            |ui| {
                if ui.button("Back").clicked {
                    let mut state = resources.get_mut::<State<GameState>>().unwrap();
                    state.set_next(GameState::Menu).unwrap();
                }
            },
        );
    });
}

fn save_level(
    world: &World,
    file_name: &str,
    type_registry: &TypeRegistryArc,
) -> Result<(), String> {
    use std::fs::File;
    use std::io::prelude::*;

    info!("Saving level!");

    let mut custom_world = World::new();

    for grid_pos in world
        .query_filtered::<&GridPos, With<PlacedTile>>()
        .into_iter()
    {
        custom_world.spawn((*grid_pos,));
    }

    let scene = DynamicScene::from_world(&custom_world, type_registry);

    match scene.serialize_ron(type_registry) {
        Err(err) => {
            error!("Failed to serialize level: {:?}!", err);
            Err(err.to_string())
        }
        Ok(level) => {
            match File::create(format!("assets/scenes/levels/custom/{}.scn.ron", file_name)) {
                Err(err) => {
                    error!("Failed to create file: {:?}!", err);
                    Err(err.to_string())
                }
                Ok(mut file) => match file.write_all(level.as_bytes()) {
                    Err(err) => {
                        error!("Failed to write level to file {:?}!", err);
                        Err(err.to_string())
                    }
                    Ok(_) => {
                        info!("Level successfully saved!");
                        Ok(())
                    }
                },
            }
        }
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
