mod tools;

use crate::camera;
use crate::solitaire::editor::tools::place_tool::PlaceAbleTile;
use crate::solitaire::editor::tools::Tool;
use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::tiles::NUMBER_OF_TILES_WITH_BONUS;
use crate::{GameState, StateStageExt, StateStagePlugin};
use bevy::ecs::ShouldRun;
use bevy::prelude::*;
use bevy::reflect::{TypeRegistry, TypeRegistryArc};
use bevy_egui::{egui, EguiContext};

pub struct EditorStateStagePlugin;

impl StateStagePlugin<GameState> for EditorStateStagePlugin {
    fn build(&self, state_stage: &mut StateStage<GameState>) {
        let state = GameState::Editor;

        state_stage
            .set_enter_stage(
                state,
                SystemStage::parallel()
                    .with_system(create_ui_state_system.system())
                    .with_system(tools::create_tool_state_system.system()),
            )
            .set_update_stage(
                state,
                Schedule::default()
                    .with_stage("ui", SystemStage::single(ui_system.system()))
                    .with_stage(
                        "input_based",
                        Schedule::default()
                            .with_run_criteria(ui_block_system.system())
                            .with_stage(
                                "tool",
                                StateStage::<Tool>::default()
                                    .add_plugin(tools::select_tool::SelectToolStateStagePlugin)
                                    .add_plugin(tools::place_tool::PlaceToolStateStagePlugin),
                            )
                            .with_stage(
                                "other",
                                SystemStage::parallel()
                                    .with_system(camera::camera_movement_system.system()),
                            ),
                    ),
            )
            .set_exit_stage(
                state,
                SystemStage::parallel()
                    .with_system(clean_up_system.system())
                    .with_system(tools::place_tool::clean_up_system.system()),
            );
    }
}

fn ui_block_system(ui_state: Res<UiState>) -> ShouldRun {
    if ui_state.dialog.is_some() {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

struct PlacedTile;

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

    egui::TopPanel::top("Menu_bar").show(ctx, |ui| {
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

    egui::TopPanel::top("Tools").show(ctx, |ui| {
        let mut tool_state = resources.get_mut::<State<Tool>>().unwrap();
        let mut current = *tool_state.current();

        ui.horizontal(|ui| {
            ui.selectable_value(&mut current, Tool::Select, "☝")
                .on_hover_text("Select tiles");
            ui.selectable_value(&mut current, Tool::Place, "🀄")
                .on_hover_text("Place tiles");
        });

        tool_state.overwrite_next(current).ok();
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
    query: Query<Entity, With<PlacedTile>>,
) {
    tile_grid_set.clear();

    for entity in query.iter() {
        commands.despawn(entity);
    }
}
