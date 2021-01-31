use crate::table::TableAssetData;
use crate::tiles::TileAssetData;
use bevy::prelude::*;

const GRID_WIDTH: f32 = TileAssetData::WIDTH / 2.0;
const GRID_HEIGHT: f32 = TileAssetData::DEPTH / 2.0;

const HALF_GRID_WIDTH: f32 = GRID_WIDTH / 2.0;
const HALF_GRID_HEIGHT: f32 = GRID_HEIGHT / 2.0;

const HALF_TILE_WIDTH: f32 = TileAssetData::WIDTH / 2.0;
const HALF_TILE_HEIGHT: f32 = TileAssetData::HEIGHT / 2.0;
const HALF_TILE_DEPTH: f32 = TileAssetData::DEPTH / 2.0;

const MAX_X: f32 = TableAssetData::INNER_SIZE / 2.0 - HALF_TILE_WIDTH;
const MAX_Z: f32 = TableAssetData::INNER_SIZE / 2.0 - HALF_TILE_DEPTH;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Reflect)]
#[reflect(Component)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_world(Vec3 { x, z, .. }: Vec3) -> Self {
        use bevy::math::*;

        let x_mouse = clamp(x + HALF_GRID_WIDTH, -MAX_X + HALF_TILE_WIDTH, MAX_X);
        let z_mouse = clamp(z + HALF_GRID_HEIGHT, -MAX_Z + HALF_TILE_DEPTH, MAX_Z);

        let x_grid = (x_mouse / GRID_WIDTH).floor() as i32;
        let z_grid = (z_mouse / GRID_HEIGHT).floor() as i32;

        GridPos::new(x_grid, 0, z_grid)
    }

    pub fn to_world(self) -> Vec3 {
        Vec3::new(
            self.x as f32 * GRID_WIDTH,
            self.y as f32 * TileAssetData::HEIGHT + HALF_TILE_HEIGHT,
            self.z as f32 * GRID_HEIGHT,
        )
    }
}

#[derive(Default, Debug)]
pub struct TileGridSet {
    set: std::collections::HashSet<GridPos>,
}

impl TileGridSet {
    pub fn new() -> Self {
        Self {
            set: std::collections::HashSet::new(),
        }
    }

    pub fn insert(&mut self, pos: GridPos) -> bool {
        self.set.insert(pos)
    }

    pub fn remove(&mut self, pos: &GridPos) -> bool {
        self.set.remove(pos)
    }

    pub fn is_overlapping(&self, pos: GridPos) -> bool {
        for z in -1..=1 {
            for x in -1..=1 {
                let pos = GridPos::new(pos.x + x, pos.y, pos.z + z);
                if self.set.contains(&pos) {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_supported_from_below(&self, GridPos { x, y, z }: GridPos) -> bool {
        y == 0 || {
            let y = y - 1;

            let center = self.set.contains(&GridPos::new(x, y, z));

            let left = self.set.contains(&GridPos::new(x - 1, y, z));
            let right = self.set.contains(&GridPos::new(x + 1, y, z));

            let top = self.set.contains(&GridPos::new(x, y, z - 1));
            let bottom = self.set.contains(&GridPos::new(x, y, z + 1));

            let top_left = self.set.contains(&GridPos::new(x - 1, y, z - 1));
            let top_right = self.set.contains(&GridPos::new(x + 1, y, z - 1));

            let bottom_left = self.set.contains(&GridPos::new(x - 1, y, z + 1));
            let bottom_right = self.set.contains(&GridPos::new(x + 1, y, z + 1));

            center
                || (left && right)
                || (top && bottom)
                || (left && top_right && bottom_right)
                || (right && top_left && bottom_left)
                || (top && bottom_left && bottom_right)
                || (bottom && top_left && top_right)
                || (top_left && top_right && bottom_left && bottom_right)
        }
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }
}
