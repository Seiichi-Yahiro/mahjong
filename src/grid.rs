use bevy::prelude::*;
use bevy::utils::HashSet;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Component)]
pub struct GridPos(IVec3);

impl GridPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }
}

#[derive(Debug, Default, Clone, Resource)]
pub struct Grid3D {
    set: HashSet<GridPos>,
    cell_size: Vec3,
    cell_subdivisions: UVec3,
    min: IVec3,
    max: IVec3,
}

impl Grid3D {
    pub fn new(
        cell_size: Vec3,
        cell_subdivisions: UVec3,
        min: Option<IVec3>,
        max: Option<IVec3>,
    ) -> Self {
        Self {
            set: HashSet::new(),
            cell_size: cell_size / (cell_subdivisions + 1).as_vec3(),
            cell_subdivisions,
            min: min
                .map(|it| it * (cell_subdivisions + 1).as_ivec3())
                .unwrap_or_else(|| IVec3::splat(i32::MIN)),
            max: max
                .map(|it| it * (cell_subdivisions + 1).as_ivec3())
                .unwrap_or_else(|| IVec3::splat(i32::MAX)),
        }
    }

    pub fn insert_from_world(&mut self, world_pos: Vec3) -> bool {
        let grid_pos = self.grid_pos_from_world(world_pos);

        grid_pos.0.cmpge(self.min).all()
            && grid_pos.0.cmple(self.max).all()
            && !self.is_overlapping(grid_pos)
            && self.set.insert(grid_pos)
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

    pub fn cell_size(&self) -> Vec3 {
        self.cell_size
    }

    pub fn min(&self) -> IVec3 {
        self.min
    }

    pub fn max(&self) -> IVec3 {
        self.max
    }

    pub fn grid_pos_from_world(&self, world_pos: Vec3) -> GridPos {
        GridPos(
            (world_pos / self.cell_size)
                .round()
                .as_ivec3()
                .clamp(self.min, self.max),
        )
    }

    pub fn snap_world_pos_to_grid(&self, world_pos: Vec3) -> Vec3 {
        self.grid_pos_from_world(world_pos).0.as_vec3() * self.cell_size
    }

    pub fn is_overlapping(&self, pos: GridPos) -> bool {
        let [x_range, y_range, z_range] = self
            .cell_subdivisions
            .to_array()
            .map(|it| it as i32)
            .map(|sub_steps| -sub_steps..=sub_steps);

        for x in x_range {
            for y in y_range.clone() {
                for z in z_range.clone() {
                    let grid_pos = GridPos::new(pos.0.x + x, pos.0.y + y, pos.0.z + z);
                    if self.set.contains(&grid_pos) {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::{Grid3D, GridPos};
    use bevy::math::{IVec3, UVec3, Vec3};

    #[test]
    fn should_calculate_grid_pos_without_subdivisions() {
        let grid = Grid3D::new(Vec3::ONE, UVec3::ZERO, None, None);
        let world_pos = Vec3::new(1.2, 0.6, -3.5);
        let grid_pos = grid.grid_pos_from_world(world_pos);
        let expected = GridPos::new(1, 1, -4);

        assert_eq!(grid_pos, expected);
    }

    #[test]
    fn should_calculate_grid_pos_with_subdivisions() {
        let grid = Grid3D::new(Vec3::ONE, UVec3::new(1, 2, 3), None, None);
        let world_pos = Vec3::new(1.2, 0.6, -3.5);
        let grid_pos = grid.grid_pos_from_world(world_pos);
        let expected = GridPos::new(2, 2, -14);

        assert_eq!(grid_pos, expected);
    }

    #[test]
    fn should_clamp_grid_pos() {
        let grid = Grid3D::new(
            Vec3::ONE,
            UVec3::ZERO,
            Some(IVec3::NEG_ONE * 2),
            Some(IVec3::ZERO),
        );
        let world_pos = Vec3::new(1.2, -1.4, -3.5);
        let grid_pos = grid.grid_pos_from_world(world_pos);
        let expected = GridPos::new(0, -1, -2);

        assert_eq!(grid_pos, expected);
    }

    #[test]
    fn should_overlap() {
        let mut grid = Grid3D::new(Vec3::ONE, UVec3::ONE, None, None);

        grid.insert_from_world(Vec3::ONE);

        let world_pos = Vec3::new(1.5, 1.0, 1.0);
        let grid_pos = grid.grid_pos_from_world(world_pos);
        let result = grid.is_overlapping(grid_pos);

        assert!(result);
    }

    #[test]
    fn should_not_overlap() {
        let mut grid = Grid3D::new(Vec3::ONE, UVec3::ONE, None, None);

        grid.insert_from_world(Vec3::ONE);

        let world_pos = Vec3::new(2.0, 1.0, 1.0);
        let grid_pos = grid.grid_pos_from_world(world_pos);
        let result = grid.is_overlapping(grid_pos);

        assert!(!result);
    }
}
