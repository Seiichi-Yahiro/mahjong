use crate::table::TableAssetData;
use crate::tiles::{TileAssetData, NUMBER_OF_TILES_WITH_BONUS};
use bevy::prelude::*;
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use std::collections::HashSet;

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

    pub fn from_world(pos: Vec3) -> Self {
        let x = pos.x;
        let z = pos.z;

        let x_mouse = (x + HALF_GRID_WIDTH).clamp(-MAX_X + HALF_TILE_WIDTH, MAX_X);
        let z_mouse = (z + HALF_GRID_HEIGHT).clamp(-MAX_Z + HALF_TILE_DEPTH, MAX_Z);

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

#[derive(Default, Debug, Clone)]
pub struct TileGridSet {
    set: std::collections::HashSet<GridPos>,
}

impl TileGridSet {
    pub fn new() -> Self {
        Self {
            set: std::collections::HashSet::with_capacity(NUMBER_OF_TILES_WITH_BONUS as usize),
        }
    }

    pub fn insert(&mut self, pos: GridPos) -> bool {
        self.set.insert(pos)
    }

    pub fn remove(&mut self, pos: &GridPos) -> bool {
        self.set.remove(pos)
    }

    pub fn clear(&mut self) {
        self.set.clear();
    }

    pub fn len(&self) -> usize {
        self.set.len()
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

    pub fn is_blocked(&self, GridPos { x, y, z }: GridPos) -> bool {
        let has_tile_above = self.is_overlapping(GridPos::new(x, y + 1, z));

        has_tile_above || {
            let has_tile_left = (-1..=1)
                .into_iter()
                .map(|z_offset| GridPos::new(x - 2, y, z + z_offset))
                .any(|grid_pos| self.set.contains(&grid_pos));

            let has_tile_right = (-1..=1)
                .into_iter()
                .map(|z_offset| GridPos::new(x + 2, y, z + z_offset))
                .any(|grid_pos| self.set.contains(&grid_pos));

            has_tile_left && has_tile_right
        }
    }

    pub fn best_effort_pairs(&self) -> Vec<(GridPos, GridPos)> {
        let mut tile_grid_set = self.clone();

        const CAPACITY: usize = NUMBER_OF_TILES_WITH_BONUS as usize / 2;
        let mut pairs = Vec::with_capacity(CAPACITY);
        let mut free_tiles = HashSet::with_capacity(CAPACITY);

        loop {
            free_tiles.extend(
                tile_grid_set
                    .set
                    .iter()
                    .cloned()
                    .filter(|grid_pos| !tile_grid_set.is_blocked(*grid_pos)),
            );

            if free_tiles.len() < 2 {
                break;
            }

            let random = free_tiles
                .iter()
                .cloned()
                .choose_multiple(&mut rand::thread_rng(), 2);

            random.iter().for_each(|grid_pos| {
                tile_grid_set.remove(grid_pos);
                free_tiles.remove(grid_pos);
            });

            pairs.push((random[0], random[1]));
        }

        pairs.extend(tile_grid_set.set.into_iter().tuples::<(_, _)>());

        pairs
    }
}
