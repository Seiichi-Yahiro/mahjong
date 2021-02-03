use crate::solitaire::grid::{GridPos, TileGridSet};
use crate::tiles::{Bonus, Plant, Season, Tile, TileAssetData};
use crate::{camera, GameState, StateStagePlugin};
use bevy::prelude::*;
use bevy_mod_picking::{Group, HoverEvents, InteractableMesh, MouseDownEvents, PickableMesh};
use rand::prelude::SliceRandom;

pub struct PlayStateStagePlugin;

impl StateStagePlugin<GameState> for PlayStateStagePlugin {
    fn build(&self, state_stage: &mut StateStage<GameState>) {
        let state = GameState::Play;

        state_stage
            .set_enter_stage(
                state,
                SystemStage::parallel().with_system(load_level_system.system()),
            )
            .set_update_stage(
                state,
                Schedule::default()
                    .with_stage(
                        "1",
                        SystemStage::parallel()
                            .with_system(spawn_tiles.system())
                            .with_system(select_system.system())
                            .with_system(pair_check_system.system())
                            .with_system(camera::camera_movement_system.system()),
                    )
                    .with_stage(
                        "2",
                        SystemStage::parallel()
                            .with_system(mark_selectable_tiles_system.system())
                            .with_system(color_tiles_system.system()),
                    ),
            );
    }
}

fn load_level_system(asset_server: Res<AssetServer>, mut scene_spawner: ResMut<SceneSpawner>) {
    // TODO bevy 0.4 cannot load scn.ron files
    let scene_handle = asset_server.load("scenes/levels/turtle.scn");
    scene_spawner.spawn_dynamic(scene_handle);
}

struct State {
    tiles: Vec<Tile>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tiles: {
                let mut tiles = Tile::new_set(true);
                tiles.shuffle(&mut rand::thread_rng());
                tiles
            },
        }
    }
}

fn spawn_tiles(
    commands: &mut Commands,
    mut state: Local<State>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tile_asset_data: Res<TileAssetData>,
    mut tile_grid_set: ResMut<TileGridSet>,
    query: Query<(Entity, &GridPos), Added<GridPos>>,
) {
    for (entity, grid_pos) in query.iter() {
        tile_grid_set.insert(*grid_pos);

        let tile = state.tiles.pop().unwrap();

        let pbr = PbrBundle {
            mesh: tile_asset_data.get_mesh(),
            material: materials.add(StandardMaterial::from(tile_asset_data.get_texture(tile))),
            transform: Transform::from_translation(grid_pos.to_world()),
            ..Default::default()
        };

        commands.insert(entity, pbr);
        commands.insert_one(entity, tile);
    }
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
    just_selected_query: Query<&Handle<StandardMaterial>, Added<Selected>>,
    query: Query<(
        &InteractableMesh,
        Option<&Selected>,
        &Handle<StandardMaterial>,
    )>,
) {
    let default_color = Color::WHITE;
    let hover_color = Color::rgb(0.3, 0.5, 0.8);
    let selected_color = Color::rgb(0.3, 0.8, 0.5);

    for material_handle in just_selected_query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.albedo = selected_color;
        }
    }

    for entity in query.removed::<Selected>() {
        if let Ok(material_handle) = query.get_component::<Handle<StandardMaterial>>(*entity) {
            if let Some(material) = materials.get_mut(material_handle) {
                material.albedo = default_color;
            }
        }
    }

    for (interactable, selected, material_handle) in query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            match interactable.hover_event(&Group::default()).unwrap() {
                HoverEvents::None => {}
                HoverEvents::JustEntered => {
                    material.albedo = hover_color;
                }
                HoverEvents::JustExited => {
                    material.albedo = if selected.is_some() {
                        selected_color
                    } else {
                        default_color
                    }
                }
            }
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
            (Tile::Bonus(first), Tile::Bonus(second)) => match first {
                Bonus::Season(season) => match season {
                    Season::Spring => second == Bonus::Plant(Plant::Plum),
                    Season::Summer => second == Bonus::Plant(Plant::Orchid),
                    Season::Fall => second == Bonus::Plant(Plant::Chrysanthemum),
                    Season::Winter => second == Bonus::Plant(Plant::Bamboo),
                },
                Bonus::Plant(plant) => match plant {
                    Plant::Plum => second == Bonus::Season(Season::Spring),
                    Plant::Orchid => second == Bonus::Season(Season::Summer),
                    Plant::Chrysanthemum => second == Bonus::Season(Season::Fall),
                    Plant::Bamboo => second == Bonus::Season(Season::Winter),
                },
            },
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
