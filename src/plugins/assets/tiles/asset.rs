use crate::plugins::assets::tiles::tile::Tile;
use crate::plugins::assets::tiles::NUMBER_OF_COVERS;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct TileAssetData {
    #[asset(path = "mesh/tile.gltf#Mesh0/Primitive0")]
    mesh: Handle<Mesh>,
    #[asset(path = "textures/tile.png")]
    mesh_texture: Handle<Image>,
    #[asset(path = "textures/covers.png")]
    cover_texture: Handle<Image>,
}

impl TileAssetData {
    pub const WIDTH: f32 = 0.03;
    pub const HEIGHT: f32 = 0.023;
    pub const DEPTH: f32 = 0.039;

    pub fn get_mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }

    pub fn get_mesh_texture(&self) -> Handle<Image> {
        self.mesh_texture.clone()
    }

    pub fn get_cover_texture(&self) -> Handle<Image> {
        self.cover_texture.clone()
    }
}

pub fn reinterpret_cover_texture_as_2d_array(
    tile_asset_data: Res<TileAssetData>,
    mut images: ResMut<Assets<Image>>,
) {
    let img = images
        .get_mut(&tile_asset_data.get_cover_texture())
        .unwrap();

    img.reinterpret_stacked_2d_as_array(NUMBER_OF_COVERS);
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "0a6f9176-b54d-46f9-8794-32a197b2e36f"]
pub struct TileMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub mesh_texture: Handle<Image>,
    #[texture(2, dimension = "2d_array")]
    #[sampler(3)]
    pub cover_texture: Handle<Image>,
    #[uniform(4)]
    pub cover_id: i32,
}

impl Material for TileMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tile.wgsl".into()
    }
}

impl TileMaterial {
    pub fn new(tile_asset_data: &TileAssetData, tile: Tile) -> Self {
        Self {
            mesh_texture: tile_asset_data.get_mesh_texture(),
            cover_texture: tile_asset_data.get_cover_texture(),
            cover_id: tile.cover_id(),
        }
    }
}
