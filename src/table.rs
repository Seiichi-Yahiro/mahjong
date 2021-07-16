use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;

pub struct Table;

pub struct TableAssetData {
    mesh: Handle<Mesh>,
    texture: Handle<Texture>,
    normal_map: Handle<Texture>,
    roughness_map: Handle<Texture>,
}

impl TableAssetData {
    pub const BORDER_THICKNESS: f32 = 0.05;
    pub const BORDER_HEIGHT: f32 = 0.02;
    pub const FULL_SIZE: f32 = 0.9;
    pub const INNER_SIZE: f32 = Self::FULL_SIZE - 2.0 * Self::BORDER_THICKNESS;

    pub fn get_mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }

    pub fn get_texture(&self) -> Handle<Texture> {
        self.texture.clone()
    }
}

impl FromWorld for TableAssetData {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let mesh = asset_server.load("mesh/table.gltf#Mesh0/Primitive0");
        let texture = asset_server.load("textures/table/table.png");
        let normal_map = asset_server.load("textures/table/table_normal.png");
        let roughness_map = asset_server.load("textures/table/table_roughness.png");

        Self {
            mesh,
            texture,
            normal_map,
            roughness_map,
        }
    }
}

pub fn spawn_table_system(
    mut commands: Commands,
    table_asset_data: Res<TableAssetData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn()
        .insert_bundle(PbrBundle {
            mesh: table_asset_data.get_mesh(),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(table_asset_data.texture.clone()),
                normal_map: Some(table_asset_data.normal_map.clone()),
                roughness: 1.0,
                metallic_roughness_texture: Some(table_asset_data.roughness_map.clone()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert(Table)
        .insert_bundle(PickableBundle::default());
}
