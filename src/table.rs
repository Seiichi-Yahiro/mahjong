use bevy::prelude::*;

pub struct TableAssetData {
    mesh: Handle<Mesh>,
    texture: Handle<Texture>,
}

impl TableAssetData {
    pub const BORDER_SIZE: f32 = 0.05;
    pub const BORDER_HEIGHT: f32 = 0.02;
    pub const FULL_SIZE: f32 = 0.9;
    pub const INNER_SIZE: f32 = Self::FULL_SIZE - Self::BORDER_SIZE;

    pub fn get_mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }

    pub fn get_texture(&self) -> Handle<Texture> {
        self.texture.clone()
    }
}

pub fn load_table_asset_data_system(commands: &mut Commands, asset_server: Res<AssetServer>) {
    let mesh = asset_server.load("mesh/table.gltf#Mesh0/Primitive0");
    let texture = asset_server.load("textures/table.png");

    commands.insert_resource(TableAssetData { mesh, texture });
}

pub fn spawn_table_system(
    commands: &mut Commands,
    table_asset_data: Res<TableAssetData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: table_asset_data.get_mesh(),
        material: materials.add(StandardMaterial::from(table_asset_data.get_texture())),
        ..Default::default()
    });
}
