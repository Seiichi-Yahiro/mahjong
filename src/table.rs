use bevy::prelude::*;
use bevy_mod_picking::PickableMesh;

pub struct Table;

pub struct TableAssetData {
    mesh: Handle<Mesh>,
    texture: Handle<Texture>,
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

impl FromResources for TableAssetData {
    fn from_resources(resources: &Resources) -> Self {
        let asset_server = resources.get::<AssetServer>().unwrap();

        let mesh = asset_server.load("mesh/table.gltf#Mesh0/Primitive0");
        let texture = asset_server.load("textures/table.png");

        Self { mesh, texture }
    }
}

pub fn spawn_table_system(
    commands: &mut Commands,
    table_asset_data: Res<TableAssetData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: table_asset_data.get_mesh(),
            material: materials.add(StandardMaterial::from(table_asset_data.get_texture())),
            ..Default::default()
        })
        .with(Table)
        .with(PickableMesh::default());
}
