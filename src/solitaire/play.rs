use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::tiles::{
    Bonus, Plant, Season, Tile, TileAssetData, TileMaterial, NUMBER_OF_TILES_WITH_BONUS,
};
use crate::{camera, GameState, StateStagePlugin};
use bevy::prelude::*;
use bevy::utils::{AHashExt, HashMap};
use bevy_egui::{egui, EguiContext};
use bevy_mod_picking::{Group, InteractableMesh, MouseDownEvents, PickableMesh};
use itertools::Itertools;
use rand::prelude::SliceRandom;
use std::ops::Deref;

pub struct PlayStateStagePlugin;

impl StateStagePlugin<GameState> for PlayStateStagePlugin {
    fn build(&self, state_stage: &mut StateStage<GameState>) {
        let state = GameState::Play;

        state_stage
            .set_enter_stage(
                state,
                SystemStage::parallel()
                    .with_system(load_levels_system.system())
                    .with_system(create_ui_state_system.system()),
            )
            .set_update_stage(
                state,
                Schedule::default()
                    .with_stage("1", SystemStage::single(ui_system.system()))
                    .with_stage(
                        "2",
                        SystemStage::parallel()
                            .with_system(spawn_tiles.system())
                            .with_system(select_system.system())
                            .with_system(pair_check_system.system())
                            .with_system(camera::camera_movement_system.system()),
                    )
                    .with_stage(
                        "3",
                        SystemStage::parallel()
                            .with_system(mark_selectable_tiles_system.system())
                            .with_system(color_tiles_system.system()),
                    ),
            )
            .set_exit_stage(state, SystemStage::single(clean_up_system.system()));
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

fn load_levels_system(commands: &mut Commands) {
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
}

fn create_ui_state_system(commands: &mut Commands) {
    commands.insert_resource(UiState::default());
}

fn ui_system(_world: &mut World, resources: &mut Resources) {
    let mut ui_state = resources.get_mut::<UiState>().unwrap();
    let mut egui_context = resources.get_mut::<EguiContext>().unwrap();
    let ctx = &mut egui_context.ctx;

    egui::SidePanel::left("side_panel", 150.0).show(ctx, |ui| {
        ui.checkbox(&mut ui_state.mark_free_tiles, "Mark free tiles");

        ui.collapsing("Levels", |ui| {
            ui.vertical_centered_justified(|ui| {
                let levels = resources.get::<Levels>().unwrap();
                let mut current_level = resources.get_mut::<Option<CurrentLevel>>().unwrap();
                let asset_server = resources.get::<AssetServer>().unwrap();
                let mut scene_spawner = resources.get_mut::<SceneSpawner>().unwrap();
                let mut tile_grid_set = resources.get_mut::<TileGridSet>().unwrap();

                let mut for_file_name = |ui: &mut egui::Ui, folder: &str, file_name: &String| {
                    if ui.button(file_name).clicked {
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
        });

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

fn spawn_tiles(
    commands: &mut Commands,
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

        commands.insert(entity, pbr);
        commands.insert(entity, (tile, TileMaterial(tile)));
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
    commands: &mut Commands,
    tile_grid_set: ChangedRes<TileGridSet>,
    query: Query<(Entity, &GridPos)>,
) {
    for (entity, grid_pos) in query.iter() {
        if !tile_grid_set.is_blocked(*grid_pos) {
            commands.insert(
                entity,
                (PickableMesh::default(), InteractableMesh::default()),
            );
        }
    }
}

struct Selected;

fn select_system(
    commands: &mut Commands,
    mouse_button_inputs: Res<Input<MouseButton>>,
    query: Query<(Entity, &InteractableMesh, Option<&Selected>)>,
) {
    if mouse_button_inputs.just_pressed(MouseButton::Left) {
        for (entity, interactable, selected) in query.iter() {
            if let MouseDownEvents::MouseJustPressed = interactable
                .mouse_down_event(&Group::default(), MouseButton::Left)
                .unwrap()
            {
                if selected.is_some() {
                    commands.remove_one::<Selected>(entity);
                } else {
                    commands.insert_one(entity, Selected);
                }
            }
        }
    }
}

fn color_tiles_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    ui_state: Res<UiState>,
    query: Query<(
        &InteractableMesh,
        Option<&Selected>,
        &Handle<StandardMaterial>,
    )>,
) {
    let default_color = Color::WHITE;
    let hover_color = Color::rgb(0.3, 0.5, 0.8);
    let selected_color = Color::rgb(0.3, 0.8, 0.5);

    for (interactable, selected, material_handle) in query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.albedo = match interactable.hover(&Group::default()).unwrap() {
                _ if selected.is_some() => selected_color,
                _ if ui_state.mark_free_tiles => hover_color,
                true => hover_color,
                false => default_color,
            };
        }
    }
}

fn pair_check_system(
    commands: &mut Commands,
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

            commands.despawn(first_entity);
            commands.despawn(second_entity);
        } else {
            commands.remove_one::<Selected>(first_entity);
            commands.remove_one::<Selected>(second_entity);
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
