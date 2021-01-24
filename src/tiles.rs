use crate::GameState;
use bevy::ecs::bevy_utils::HashMap;
use bevy::prelude::*;
use bevy::utils::AHashExt;

pub trait EnumIter {
    fn next(self) -> Self;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tile {
    Suit(Suit),
    Honor(Honor),
    Bonus(Bonus),
}

impl Tile {
    pub fn new_set(with_bonus: bool) -> Vec<Self> {
        let dots = Number::iter().map(|num| Suit::Dot(num));
        let bamboos = Number::iter().map(|num| Suit::Bamboo(num));
        let chars = Number::iter().map(|num| Suit::Char(num));
        let suits = dots.chain(bamboos).chain(chars).map(Tile::from);

        let winds = Wind::iter().map(Honor::from);
        let dragons = Dragon::iter().map(Honor::from);
        let honors = winds.chain(dragons).map(Tile::from);

        let mut tiles = suits.chain(honors).collect::<Vec<Self>>().repeat(4);

        if with_bonus {
            let seasons = Season::iter().map(Bonus::from);
            let plants = Plant::iter().map(Bonus::from);
            let bonus = seasons.chain(plants).map(Tile::from);

            tiles.extend(bonus);
        }

        tiles
    }
}

impl From<Suit> for Tile {
    fn from(suit: Suit) -> Self {
        Tile::Suit(suit)
    }
}

impl From<Honor> for Tile {
    fn from(honor: Honor) -> Self {
        Tile::Honor(honor)
    }
}

impl From<Bonus> for Tile {
    fn from(bonus: Bonus) -> Self {
        Tile::Bonus(bonus)
    }
}

impl From<Wind> for Tile {
    fn from(wind: Wind) -> Self {
        Tile::Honor(wind.into())
    }
}

impl From<Dragon> for Tile {
    fn from(dragon: Dragon) -> Self {
        Tile::Honor(dragon.into())
    }
}

impl EnumIter for Tile {
    fn next(self) -> Self {
        match self {
            Self::Suit(tile) => Self::Suit(tile.next()),
            Self::Honor(tile) => Self::Honor(tile.next()),
            tile => tile,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Suit {
    Dot(Number),
    Bamboo(Number),
    Char(Number),
}

impl EnumIter for Suit {
    fn next(self) -> Self {
        match self {
            Self::Dot(number) => Self::Dot(number.next()),
            Self::Bamboo(number) => Self::Bamboo(number.next()),
            Self::Char(number) => Self::Char(number.next()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Number {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl Number {
    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        [
            Self::One,
            Self::Two,
            Self::Three,
            Self::Four,
            Self::Five,
            Self::Six,
            Self::Seven,
            Self::Eight,
            Self::Nine,
        ]
        .iter()
        .copied()
    }
}

impl EnumIter for Number {
    fn next(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::Three,
            Self::Three => Self::Four,
            Self::Four => Self::Five,
            Self::Five => Self::Six,
            Self::Six => Self::Seven,
            Self::Seven => Self::Eight,
            Self::Eight => Self::Nine,
            Self::Nine => Self::One,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Honor {
    Wind(Wind),
    Dragon(Dragon),
}

impl From<Wind> for Honor {
    fn from(wind: Wind) -> Self {
        Honor::Wind(wind)
    }
}

impl From<Dragon> for Honor {
    fn from(dragon: Dragon) -> Self {
        Honor::Dragon(dragon)
    }
}

impl EnumIter for Honor {
    fn next(self) -> Self {
        match self {
            Self::Wind(honor) => Self::Wind(honor.next()),
            Self::Dragon(honor) => Self::Dragon(honor.next()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Wind {
    East,
    South,
    West,
    North,
}

impl Wind {
    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        [Self::East, Self::South, Self::West, Self::North]
            .iter()
            .copied()
    }
}

impl EnumIter for Wind {
    fn next(self) -> Self {
        match self {
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::North => Self::East,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Dragon {
    White,
    Green,
    Red,
}

impl Dragon {
    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        [Self::White, Self::Green, Self::Red].iter().copied()
    }
}

impl EnumIter for Dragon {
    fn next(self) -> Self {
        match self {
            Self::White => Self::Green,
            Self::Green => Self::Red,
            Self::Red => Self::White,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Bonus {
    Season(Season),
    Plant(Plant),
}

impl From<Season> for Bonus {
    fn from(season: Season) -> Self {
        Bonus::Season(season)
    }
}

impl From<Plant> for Bonus {
    fn from(plant: Plant) -> Self {
        Bonus::Plant(plant)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        [Self::Spring, Self::Summer, Self::Fall, Self::Winter]
            .iter()
            .copied()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Plant {
    Plum,
    Orchid,
    Chrysanthemum,
    Bamboo,
}

impl Plant {
    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        [Self::Plum, Self::Orchid, Self::Chrysanthemum, Self::Bamboo]
            .iter()
            .copied()
    }
}

pub struct TileAssetData {
    mesh: Handle<Mesh>,
    mesh_texture: Handle<Texture>,
    covers: HashMap<Tile, Handle<Texture>>,
    textures: HashMap<Tile, Handle<Texture>>,
}

impl TileAssetData {
    pub const WIDTH: f32 = 0.03;
    pub const HEIGHT: f32 = 0.023;
    pub const DEPTH: f32 = 0.039;

    pub fn get_mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }

    pub fn get_texture(&self, tile: Tile) -> Handle<Texture> {
        self.textures.get(&tile.into()).unwrap().clone()
    }
}

pub fn load_tile_asset_data_system(commands: &mut Commands, asset_server: Res<AssetServer>) {
    let mesh = asset_server.load("mesh/tile.gltf#Mesh0/Primitive0");
    let mesh_texture = asset_server.load("textures/tile.png");

    let covers = [
        (Tile::Suit(Suit::Dot(Number::One)), "dots/1"),
        (Tile::Suit(Suit::Dot(Number::Two)), "dots/2"),
        (Tile::Suit(Suit::Dot(Number::Three)), "dots/3"),
        (Tile::Suit(Suit::Dot(Number::Four)), "dots/4"),
        (Tile::Suit(Suit::Dot(Number::Five)), "dots/5"),
        (Tile::Suit(Suit::Dot(Number::Six)), "dots/6"),
        (Tile::Suit(Suit::Dot(Number::Seven)), "dots/7"),
        (Tile::Suit(Suit::Dot(Number::Eight)), "dots/8"),
        (Tile::Suit(Suit::Dot(Number::Nine)), "dots/9"),
        (Tile::Suit(Suit::Bamboo(Number::One)), "bamboos/1"),
        (Tile::Suit(Suit::Bamboo(Number::Two)), "bamboos/2"),
        (Tile::Suit(Suit::Bamboo(Number::Three)), "bamboos/3"),
        (Tile::Suit(Suit::Bamboo(Number::Four)), "bamboos/4"),
        (Tile::Suit(Suit::Bamboo(Number::Five)), "bamboos/5"),
        (Tile::Suit(Suit::Bamboo(Number::Six)), "bamboos/6"),
        (Tile::Suit(Suit::Bamboo(Number::Seven)), "bamboos/7"),
        (Tile::Suit(Suit::Bamboo(Number::Eight)), "bamboos/8"),
        (Tile::Suit(Suit::Bamboo(Number::Nine)), "bamboos/9"),
        (Tile::Suit(Suit::Char(Number::One)), "chars/1"),
        (Tile::Suit(Suit::Char(Number::Two)), "chars/2"),
        (Tile::Suit(Suit::Char(Number::Three)), "chars/3"),
        (Tile::Suit(Suit::Char(Number::Four)), "chars/4"),
        (Tile::Suit(Suit::Char(Number::Five)), "chars/5"),
        (Tile::Suit(Suit::Char(Number::Six)), "chars/6"),
        (Tile::Suit(Suit::Char(Number::Seven)), "chars/7"),
        (Tile::Suit(Suit::Char(Number::Eight)), "chars/8"),
        (Tile::Suit(Suit::Char(Number::Nine)), "chars/9"),
        (Tile::Honor(Honor::Wind(Wind::East)), "winds/east"),
        (Tile::Honor(Honor::Wind(Wind::South)), "winds/south"),
        (Tile::Honor(Honor::Wind(Wind::West)), "winds/west"),
        (Tile::Honor(Honor::Wind(Wind::North)), "winds/north"),
        (Tile::Honor(Honor::Dragon(Dragon::White)), "dragons/white"),
        (Tile::Honor(Honor::Dragon(Dragon::Green)), "dragons/green"),
        (Tile::Honor(Honor::Dragon(Dragon::Red)), "dragons/red"),
        (Tile::Bonus(Bonus::Season(Season::Spring)), "seasons/spring"),
        (Tile::Bonus(Bonus::Season(Season::Summer)), "seasons/summer"),
        (Tile::Bonus(Bonus::Season(Season::Fall)), "seasons/fall"),
        (Tile::Bonus(Bonus::Season(Season::Winter)), "seasons/winter"),
        (Tile::Bonus(Bonus::Plant(Plant::Plum)), "plants/plum"),
        (Tile::Bonus(Bonus::Plant(Plant::Orchid)), "plants/orchid"),
        (
            Tile::Bonus(Bonus::Plant(Plant::Chrysanthemum)),
            "plants/chrysanthemum",
        ),
        (Tile::Bonus(Bonus::Plant(Plant::Bamboo)), "plants/bamboo"),
    ]
    .iter()
    .map(|(tile, path)| {
        (
            *tile,
            asset_server.load(format!("textures/{}.png", path).as_str()),
        )
    })
    .collect::<HashMap<_, _>>();

    commands.insert_resource(TileAssetData {
        mesh,
        mesh_texture,
        covers,
        textures: HashMap::new(),
    });
}

pub fn blend_tile_textures_system(
    mut state: ResMut<State<GameState>>,
    mut tile_asset_data: ResMut<TileAssetData>,
    mut textures: ResMut<Assets<Texture>>,
) {
    if tile_asset_data
        .covers
        .iter()
        .map(|(_, handle)| handle)
        .chain(std::iter::once(&tile_asset_data.mesh_texture))
        .any(|handle| textures.get(handle).is_none())
    {
        return;
    }

    tile_asset_data.textures = tile_asset_data
        .covers
        .iter()
        .map(|(tile, texture_handle)| {
            let new_texture = {
                let mesh_texture = textures.get(&tile_asset_data.mesh_texture).unwrap();
                let tile_texture = textures.get(texture_handle.clone()).unwrap();
                alpha_blend_textures(mesh_texture, tile_texture)
            };

            (*tile, textures.add(new_texture))
        })
        .collect();

    state.set_next(GameState::Play).unwrap();
}

fn alpha_blend_textures(mesh_texture: &Texture, tile_texture: &Texture) -> Texture {
    let mut mesh_texture = mesh_texture.clone();

    const COLOR_CHANNELS: usize = 4;
    let tile_height = tile_texture.size.height as usize;
    let tile_width = tile_texture.size.width as usize;

    let to_index = |x, y, width| y * width * COLOR_CHANNELS + x * COLOR_CHANNELS;

    for y in 0..tile_height {
        for x in 0..tile_width {
            let tile_index = to_index(x, y, tile_width);
            let mesh_index = to_index(x, y, mesh_texture.size.width as usize);

            let tile_alpha = tile_texture.data[tile_index + COLOR_CHANNELS - 1] as u32 + 1;
            let tile_alpha_inverse = 256 - tile_alpha;

            for channel in 0..3 {
                let tile_color = tile_texture.data[tile_index + channel] as u32;
                let mesh_color = mesh_texture.data[mesh_index + channel] as u32;
                let new_color = tile_alpha * tile_color + tile_alpha_inverse * mesh_color;

                mesh_texture.data[mesh_index + channel] = (new_color >> 8) as u8;
            }
        }
    }

    mesh_texture
}
