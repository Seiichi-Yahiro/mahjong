use crate::plugins::assets::tiles::asset::TileAssetData;
use crate::plugins::assets::tiles::NUMBER_OF_TILES_WITH_BONUS;
use bevy::prelude::*;
use bevy::utils::HashSet;

const HALF_TILE_WIDTH: f32 = TileAssetData::WIDTH / 2.0;
const HALF_TILE_DEPTH: f32 = TileAssetData::DEPTH / 2.0;

const CELL_WIDTH: f32 = HALF_TILE_WIDTH;
const CELL_DEPTH: f32 = HALF_TILE_DEPTH;
const CELL_HEIGHT: f32 = TileAssetData::HEIGHT;

const HALF_CELL_WIDTH: f32 = CELL_WIDTH / 2.0;
const HALF_CELL_DEPTH: f32 = CELL_DEPTH / 2.0;

const GRID_SIZE: f32 = 1.0;
const HALF_GRID_SIZE: f32 = GRID_SIZE / 2.0;

const MAX_X: f32 = HALF_GRID_SIZE - HALF_TILE_WIDTH;
const MAX_Z: f32 = HALF_GRID_SIZE - HALF_TILE_DEPTH;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Component)]
pub struct GridPos {
    x: i32,
    y: u32,
    z: i32,
}

impl GridPos {
    pub fn new(x: i32, y: u32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_world(pos: Vec3) -> Self {
        let x_mouse = (pos.x + HALF_CELL_WIDTH).clamp(-MAX_X + HALF_TILE_WIDTH, MAX_X);
        let z_mouse = (pos.z + HALF_CELL_DEPTH).clamp(-MAX_Z + HALF_TILE_DEPTH, MAX_Z);

        let x_grid = (x_mouse / CELL_WIDTH).floor() as i32;
        let z_grid = (z_mouse / CELL_DEPTH).floor() as i32;

        Self::new(x_grid, 0, z_grid)
    }

    pub fn to_world(self) -> Vec3 {
        Vec3::new(
            self.x as f32 * CELL_WIDTH,
            self.y as f32 * CELL_HEIGHT,
            self.z as f32 * CELL_DEPTH,
        )
    }
}

#[derive(Debug, Clone, Resource)]
pub struct Grid(HashSet<GridPos>);

impl Grid {
    pub fn new() -> Self {
        Self(HashSet::with_capacity(NUMBER_OF_TILES_WITH_BONUS as usize))
    }

    pub fn insert(&mut self, pos: GridPos) -> bool {
        self.0.insert(pos)
    }

    pub fn remove(&mut self, pos: &GridPos) -> bool {
        self.0.remove(pos)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_overlapping(&self, pos: GridPos) -> bool {
        for z in -1..=1 {
            for x in -1..=1 {
                let pos = GridPos::new(pos.x + x, pos.y, pos.z + z);
                if self.0.contains(&pos) {
                    return true;
                }
            }
        }

        false
    }
}
