use crate::plugins::assets::tiles::asset::TileAssetData;
use bevy::prelude::*;

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
pub struct GridPos(IVec3);

impl GridPos {
    pub fn from_world(pos: Vec3) -> Self {
        let x_mouse = (pos.x + HALF_CELL_WIDTH).clamp(-MAX_X + HALF_TILE_WIDTH, MAX_X);
        let z_mouse = (pos.z + HALF_CELL_DEPTH).clamp(-MAX_Z + HALF_TILE_DEPTH, MAX_Z);

        let x_grid = (x_mouse / CELL_WIDTH).floor() as i32;
        let z_grid = (z_mouse / CELL_DEPTH).floor() as i32;

        Self(IVec3::new(x_grid, 0, z_grid))
    }

    pub fn to_world(self) -> Vec3 {
        Vec3::new(
            self.0.x as f32 * CELL_WIDTH,
            self.0.y as f32 * CELL_HEIGHT,
            self.0.z as f32 * CELL_DEPTH,
        )
    }
}
