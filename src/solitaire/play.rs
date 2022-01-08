use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::tiles::{
    Bonus, Plant, Season, Tile, TileAssetData, TileMaterial, NUMBER_OF_TILES_WITH_BONUS,
};
use crate::{camera, GameState};
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use bevy_mod_picking::{Hover, PickableBundle, PickableMesh, PickingCamera};
use itertools::Itertools;
use rand::prelude::SliceRandom;
use std::collections::HashMap;
use std::ops::Deref;

pub struct PlayStatePlugin;

impl Plugin for PlayStatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        let state = GameState::Play;

        app.init_resource::<UiState>()
            .add_system_set(
                SystemSet::on_enter(state)
                    .with_system(load_levels_system.system())
                    .with_system(create_ui_state_system.system()),
            )
            .add_system_set(
                SystemSet::on_update(state)
                    //.with_run_criteria(ui_block_system.system())
                    .with_system(select_system.system().label(SystemLabel::Select))
                    .with_system(camera::camera_movement_system.system()),
            )
            .add_system_set(
                SystemSet::on_update(state)
                    .with_system(ui_system.system().label(SystemLabel::UI))
                    .with_system(
                        spawn_tiles
                            .system()
                            .label(SystemLabel::Spawn)
                            .after(SystemLabel::UI),
                    )
                    .with_system(
                        pair_check_system
                            .system()
                            .label(SystemLabel::PairCheck)
                            .after(SystemLabel::Select),
                    )
                    .with_system(
                        color_tiles_system
                            .system()
                            .label(SystemLabel::Color)
                            .before(SystemLabel::PairCheck),
                    )
                    .with_system(
                        mark_selectable_tiles_system
                            .system()
                            .label(SystemLabel::MarkSelectable)
                            .before(SystemLabel::Select)
                            .after(SystemLabel::Spawn),
                    ),
            )
            .add_system_set(SystemSet::on_exit(state).with_system(clean_up_system.system()));
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum SystemLabel {
    UI,
    Spawn,
    Select,
    PairCheck,
    MarkSelectable,
    Color,
}

fn ui_block_system(egui_context: Res<EguiContext>) -> ShouldRun {
    if egui_context.ctx().is_pointer_over_area() {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

struct CurrentLevel {
    scene: Handle<DynamicScene>,
}

impl CurrentLevel {
    fn new(scene: Handle<DynamicScene>) -> Self {
        Self { scene }
    }
}

struct Levels {
    pre_made: Vec<String>,
    custom: Vec<String>,
}

fn load_levels_system(mut commands: Commands) {
    let pre_made = get_level_file_names_from_folder("assets/scenes/levels/pre_made");
    let custom = get_level_file_names_from_folder("assets/scenes/levels/custom");

    commands.insert_resource::<Option<CurrentLevel>>(None);
    commands.insert_resource(Levels { pre_made, custom });
}

fn get_level_file_names_from_folder(folder: &str) -> Vec<String> {
    use std::fs;
    use std::path::Path;

    match fs::read_dir(folder) {
        Err(err) => {
            error!("Failed to read level directory: {:?}", err);
            Vec::new()
        }
        Ok(read_dir) => read_dir
            .filter_map(|entry_result| {
                entry_result
                    .ok()
                    .filter(|entry| {
                        entry
                            .file_type()
                            .map(|file_type| file_type.is_file())
                            .unwrap_or(false)
                    })
                    .filter(|entry| {
                        Path::new(&entry.file_name())
                            .extension()
                            .map(|ext| ext == "scn")
                            .unwrap_or(false)
                    })
                    .and_then(|entry| {
                        Path::new(&entry.file_name())
                            .file_stem()
                            .and_then(|file_name| file_name.to_str())
                            .map(|file_name| file_name.to_string())
                    })
            })
            .collect(),
    }
}

#[derive(Default)]
struct UiState {
    mark_free_tiles: bool,
    select_level_dialog: bool,
}

fn create_ui_state_system(mut commands: Commands) {
    commands.insert_resource(UiState::default());
}

fn ui_system(
    egui_ctx: ResMut<EguiContext>,
    mut ui_state: ResMut<UiState>,
    mut game_state: ResMut<State<GameState>>,
    levels: Res<Levels>,
    mut current_level: ResMut<Option<CurrentLevel>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut tile_grid_set: ResMut<TileGridSet>,
) {
    let ctx = egui_ctx.ctx();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, "Menu", |ui| {
                if ui.button("Select level...").clicked() {
                    ui_state.select_level_dialog = true;
                }

                if ui.button("Back to menu").clicked() {
                    game_state.set(GameState::Menu).unwrap();
                }
            });

            egui::menu::menu_button(ui, "Options", |ui| {
                ui.checkbox(&mut ui_state.mark_free_tiles, "Mark free tiles");
            });
        });
    });

    egui::Window::new("Select Level")
        .vscroll(true)
        .open(&mut ui_state.select_level_dialog)
        .show(ctx, |ui| {
            let mut for_file_name = |ui: &mut egui::Ui, folder: &str, file_name: &String| {
                if ui.button(file_name).clicked() {
                    if let Some(current_level) = current_level.deref() {
                        scene_spawner.despawn(current_level.scene.clone());
                    }

                    tile_grid_set.clear();

                    let path = format!("scenes/levels/{}/{}.scn", folder, file_name);
                    let scene_handle = asset_server.load(path.as_str());
                    *current_level = Some(CurrentLevel::new(scene_handle.clone()));
                    scene_spawner.spawn_dynamic(scene_handle);
                }
            };

            ui.collapsing("Premade", |ui| {
                levels
                    .pre_made
                    .iter()
                    .for_each(|file_name| for_file_name(ui, "pre_made", file_name));
            });

            ui.collapsing("Custom", |ui| {
                levels
                    .custom
                    .iter()
                    .for_each(|file_name| for_file_name(ui, "custom", file_name));
            });
        });
}

// TODO ugly logic
fn spawn_tiles(
    mut commands: Commands,
    tile_asset_data: Res<TileAssetData>,
    mut tile_grid_set: ResMut<TileGridSet>,
    query: Query<(Entity, &GridPos), Added<GridPos>>,
) {
    let mut should_run = false;

    for (_, grid_pos) in query.iter() {
        tile_grid_set.insert(*grid_pos);
        should_run = true;
    }

    if !should_run {
        return;
    }

    let pairs = {
        let grid_pairs = tile_grid_set.best_effort_pairs().into_iter();
        let tile_pairs = create_tile_pairs().into_iter();
        grid_pairs.zip(tile_pairs).fold(
            HashMap::with_capacity(NUMBER_OF_TILES_WITH_BONUS as usize),
            |mut acc, ((g_a, g_b), (t_a, t_b))| {
                acc.insert(g_a, t_a);
                acc.insert(g_b, t_b);
                acc
            },
        )
    };

    for (entity, grid_pos) in query.iter() {
        let tile = *pairs.get(grid_pos).unwrap();

        let pbr = PbrBundle {
            mesh: tile_asset_data.get_mesh(),
            transform: Transform::from_translation(grid_pos.to_world()),
            ..Default::default()
        };

        commands
            .entity(entity)
            .insert_bundle(pbr)
            .insert_bundle((tile, TileMaterial(tile)));
    }
}

fn create_tile_pairs() -> Vec<(Tile, Tile)> {
    let mut seasons = Season::iter().map(Tile::from).collect_vec();
    seasons.shuffle(&mut rand::thread_rng());

    let mut plants = Plant::iter().map(Tile::from).collect_vec();
    plants.shuffle(&mut rand::thread_rng());

    let bonus = seasons.into_iter().chain(plants.into_iter()).tuples();

    let mut tiles = Tile::new_normal_set()
        .into_iter()
        .flat_map(|tile| std::iter::repeat((tile, tile)).take(2))
        .chain(bonus)
        .collect::<Vec<_>>();

    tiles.shuffle(&mut rand::thread_rng());

    tiles
}

fn mark_selectable_tiles_system(
    mut commands: Commands,
    tile_grid_set: Res<TileGridSet>,
    query: Query<(Entity, &GridPos)>,
) {
    if !tile_grid_set.is_changed() {
        return;
    }

    for (entity, grid_pos) in query.iter() {
        if !tile_grid_set.is_blocked(*grid_pos) {
            commands
                .entity(entity)
                .insert_bundle(PickableBundle::default());
        }
    }
}

struct Selected;

fn select_system(
    mut commands: Commands,
    mouse_button_inputs: Res<Input<MouseButton>>,
    camera_query: Query<&PickingCamera>,
    select_query: Query<Option<&Selected>, (With<PickableMesh>, With<Tile>)>,
) {
    if mouse_button_inputs.just_pressed(MouseButton::Left) {
        match camera_query.single() {
            Err(err) => {
                error!("{:?}", err);
            }
            Ok(camera) => camera.intersect_top().iter().for_each(|(entity, _)| {
                if let Ok(selected) = select_query.get(*entity) {
                    if selected.is_some() {
                        commands.entity(*entity).remove::<Selected>();
                    } else {
                        commands.entity(*entity).insert(Selected);
                    }
                };
            }),
        }
    }
}

// TODO only on change
fn color_tiles_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    ui_state: Res<UiState>,
    query: Query<(&Hover, Option<&Selected>, &Handle<StandardMaterial>), With<Tile>>,
) {
    const DEFAULT_COLOR: Color = Color::WHITE;
    const HOVER_COLOR: Color = Color::rgb(0.3, 0.5, 0.8);
    const SELECTED_COLOR: Color = Color::rgb(0.3, 0.8, 0.5);

    for (hover, selected, material_handle) in query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = match hover.hovered() {
                _ if selected.is_some() => SELECTED_COLOR,
                _ if ui_state.mark_free_tiles => HOVER_COLOR,
                true => HOVER_COLOR,
                false => DEFAULT_COLOR,
            };
        }
    }
}

fn pair_check_system(
    mut commands: Commands,
    mut tile_grid_set: ResMut<TileGridSet>,
    query: Query<(Entity, &Tile, &GridPos), With<Selected>>,
) {
    if query.iter().take(2).count() == 2 {
        let mut selected = query.iter().take(2);
        let (first_entity, &first_tile, first_grid_pos) = selected.next().unwrap();
        let (second_entity, &second_tile, second_grid_pos) = selected.next().unwrap();

        let is_pair = match (first_tile, second_tile) {
            (Tile::Bonus(Bonus::Plant(_)), Tile::Bonus(Bonus::Plant(_)))
            | (Tile::Bonus(Bonus::Season(_)), Tile::Bonus(Bonus::Season(_))) => true,
            (first, second) => first == second,
        };

        if is_pair {
            tile_grid_set.remove(first_grid_pos);
            tile_grid_set.remove(second_grid_pos);

            commands.entity(first_entity).despawn();
            commands.entity(second_entity).despawn();
        } else {
            commands.entity(first_entity).remove::<Selected>();
            commands.entity(second_entity).remove::<Selected>();
        }
    }
}

fn clean_up_system(
    mut current_level: ResMut<Option<CurrentLevel>>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut tile_grid_set: ResMut<TileGridSet>,
) {
    if let Some(current_level) = current_level.deref() {
        scene_spawner.despawn(current_level.scene.clone());
    }

    *current_level = None;
    tile_grid_set.clear();
}
