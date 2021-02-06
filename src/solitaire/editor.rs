use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::table::Table;
use crate::tiles::{TileAssetData, NUMBER_OF_TILES_WITH_BONUS};
use crate::{camera, GameState, StateStagePlugin};
use bevy::ecs::ShouldRun;
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
                    .with_stage("ui", SystemStage::single(ui_system.system()))
                    .with_stage(
                        "other",
                        Schedule::default()
                            .with_run_criteria(ui_block_system.system())
                            .with_stage(
                                "1",
                                SystemStage::parallel()
                                    .with_system(move_placeable_tile_system.system())
                                    .with_system(is_placeable_system.system())
                                    .with_system(color_placeable_tile_system.system())
                                    .with_system(place_tile_system.system())
                                    .with_system(camera::camera_movement_system.system()),
                            )
                            .with_stage(
                                "2",
                                SystemStage::parallel().with_system(undo_system.system()),
                            ),
                    ),
            )
            .set_exit_stage(state, SystemStage::single(clean_up_system.system()));
    }
}

fn ui_block_system(ui_state: Res<UiState>) -> ShouldRun {
    if ui_state.dialog.is_some() {
        ShouldRun::No
    } else {
        ShouldRun::Yes
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

    if keyboard_input.pressed(KeyCode::LControl) && keyboard_input.just_pressed(KeyCode::Z) {
        if let Some((entity, grid_pos)) = state.0.pop() {
            tile_grid_set.remove(&grid_pos);
            commands.despawn(entity);
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Dialog {
    Save,
    Error,
}

#[derive(Default)]
struct UiState {
    file_name: String,
    can_save: bool,
    error_msg: String,
    dialog: Option<Dialog>,
}

impl UiState {
    fn open_dialog(&mut self, dialog: Dialog) {
        self.dialog = Some(dialog);
    }

    fn close_dialog(&mut self) {
        self.dialog = None;
    }

    fn error(&mut self, msg: String) {
        self.error_msg = msg;
        self.dialog = Some(Dialog::Error);
    }
}

fn create_ui_state_system(commands: &mut Commands) {
    commands.insert_resource(UiState::default());
}

fn ui_system(world: &mut World, resources: &mut Resources) {
    let mut ui_state = resources.get_mut::<UiState>().unwrap();
    let mut egui_context = resources.get_mut::<EguiContext>().unwrap();
    let ctx = &mut egui_context.ctx;

    let placed_tiles = resources.get::<TileGridSet>().unwrap().len();

    ui_state.can_save = placed_tiles > 0
        && placed_tiles <= NUMBER_OF_TILES_WITH_BONUS as usize
        && placed_tiles % 2 == 0;

    egui::TopPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu(ui, "File", |ui| {
                let save_button = egui::Button::new("Save as...").enabled(ui_state.can_save);
                if ui.add(save_button).clicked {
                    ui_state.open_dialog(Dialog::Save);
                }

                if ui.button("Back to menu").clicked {
                    let mut state = resources.get_mut::<State<GameState>>().unwrap();
                    state.set_next(GameState::Menu).unwrap();
                }
            });
        });
    });

    egui::Area::new("info").movable(false).show(ctx, |ui| {
        let color = if ui_state.can_save {
            egui::Color32::GREEN
        } else {
            egui::Color32::RED
        };
        ui.colored_label(
            color,
            format!("Tiles: {}/{}", placed_tiles, NUMBER_OF_TILES_WITH_BONUS),
        );

        if let Some(GridPos { x, y, z }) = world
            .query_filtered::<&GridPos, With<PlaceAbleTile>>()
            .next()
        {
            ui.label(format!("Coordinates: {}, {}, {}", x, y, z));
        }
    });

    if let Some(dialog) = ui_state.dialog {
        match dialog {
            Dialog::Save => {
                egui::Window::new("💾 Save as")
                    .collapsible(false)
                    .fixed_size([250.0, 200.0])
                    .show(ctx, |ui| {
                        ui.label("Filename:");
                        ui.text_edit_singleline(&mut ui_state.file_name);

                        ui.separator();

                        ui.horizontal(|ui| {
                            let can_save = !ui_state.file_name.is_empty();
                            let save_button = egui::Button::new("Save").enabled(can_save);
                            if ui.add(save_button).clicked {
                                let type_registry = resources.get::<TypeRegistry>().unwrap();

                                match save_level(world, &ui_state.file_name, &type_registry) {
                                    Ok(_) => ui_state.close_dialog(),
                                    Err(msg) => {
                                        ui_state.error(msg);
                                    }
                                }
                            }

                            if ui.button("Cancel").clicked {
                                ui_state.close_dialog();
                            }
                        });
                    });
            }
            Dialog::Error => {
                egui::Window::new("⁉ Error")
                    .collapsible(false)
                    .min_width(250.0)
                    .show(ctx, |ui| {
                        ui.horizontal_wrapped_for_text(egui::TextStyle::Body, |ui| {
                            ui.label(&ui_state.error_msg);
                        });

                        ui.separator();

                        if ui.button("Ok").clicked {
                            ui_state.close_dialog();
                        }
                    });
            }
        }
    }
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

    // TODO bevy 0.4 cannot load scn.ron files
    match scene.serialize_ron(type_registry) {
        Err(err) => {
            error!("Failed to serialize level: {:?}!", err);
            Err(err.to_string())
        }
        Ok(level) => match File::create(format!("assets/scenes/levels/custom/{}.scn", file_name)) {
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
        },
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
