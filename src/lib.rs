pub use bevy::{prelude::*, render::color::Color};
pub use iyes_loopless::prelude::*;
// pub mod ant;
// pub mod food;
// pub mod nest;
// pub mod pheromones;
pub mod network;
pub mod wander;

const NEST_SPREAD: f32 = BORDER_PADDING;

pub const BOARD_HEIGHT: usize = 0;
pub const NEST_HEIGHT: usize = 1;
pub const ANT_HEIGHT: usize = 2;
pub const FOOD_HEIGHT: usize = 3;

pub const FOOD_SIZE: f32 = 128.;
pub const FOOD_SIZE_V3: Vec3 = Vec3::new(FOOD_SIZE, FOOD_SIZE, FOOD_HEIGHT as f32);

pub const ANT_SPEED: f32 = 128.;
pub const ANT_SIZE: f32 = 0.0625;
pub const ANT_SCALE: Vec3 = Vec3::new(ANT_SIZE, ANT_SIZE, ANT_HEIGHT as f32);
pub const ANT_ANIMATION_SPEED: f32 = 10.0;

pub const WINDOW_SIZE: f32 = 800.;
pub const WINDOW_SIZE_X: f32 = WINDOW_SIZE;
pub const WINDOW_SIZE_Y: f32 = WINDOW_SIZE;

pub const NUM_NESTS: usize = 15;
pub const NEST_SIZE: f32 = 16.;
pub const NEST_FOOD_REQUEST_PROB: f32 = 0.01;

// TODO: make colors a resource
// TODO: make colors have unique id so mutltiple nests of same color can exist
// pub const NEST_COLORS: [Color; 10] = [
//     Color::rgb(1.0, 0.745, 0.0431),
//     Color::rgb(0.984, 0.337, 0.027),
//     Color::rgb(1.0, 0.0, 0.431),
//     Color::rgb(0.513, 0.219, 0.925),
//     Color::rgb(0.227, 0.525, 1.0),
//     Color::rgb(0.1059, 0.9059, 1.0000),
//     Color::rgb(0.4314, 0.9216, 0.5137),
//     Color::rgb(0.8941, 1.0000, 0.1020),
//     Color::rgb(1.0000, 0.7216, 0.0000),
//     Color::rgb(1.0000, 0.3412, 0.0784),
// ];

pub const NEST_COLORS: [Color; 20] = [
    Color::rgb(0.3059, 0.4745, 0.6549),
    Color::rgb(0.6275, 0.7961, 0.9098),
    Color::rgb(0.949, 0.5569, 0.1686),
    Color::rgb(1.0, 0.7451, 0.4902),
    Color::rgb(0.349, 0.6314, 0.3098),
    Color::rgb(0.549, 0.8196, 0.4902),
    Color::rgb(0.7137, 0.6, 0.1765),
    Color::rgb(0.9451, 0.8078, 0.3882),
    Color::rgb(0.2863, 0.5961, 0.5804),
    Color::rgb(0.5255, 0.7373, 0.7137),
    Color::rgb(0.8824, 0.3412, 0.349),
    Color::rgb(1.0, 0.6157, 0.6039),
    Color::rgb(0.4745, 0.4392, 0.4314),
    Color::rgb(0.7294, 0.6902, 0.6745),
    Color::rgb(0.8275, 0.4471, 0.5843),
    Color::rgb(0.9804, 0.749, 0.8235),
    Color::rgb(0.6902, 0.4784, 0.6314),
    Color::rgb(0.8314, 0.651, 0.7843),
    Color::rgb(0.6157, 0.4627, 0.3765),
    Color::rgb(0.8431, 0.7098, 0.651),
];
// 1BE7FF
// 6EEB83
// E4FF1A
// FFB800
// FF5714

// allowed distance to edge of screen
const BORDER_PADDING: f32 = 50.0;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Colors {
    pub colors: Vec<Color>,
    pub color_ids: Vec<usize>,
    pub color_handles: Vec<Handle<ColorMaterial>>,
}

use std::iter::{zip, Zip};
use std::vec::IntoIter;

impl Colors {
    pub fn iter(&self) -> Zip<IntoIter<Color>, IntoIter<usize>> {
        zip(self.colors.clone(), self.color_ids.clone())
    }
}

impl FromWorld for Colors {
    fn from_world(world: &mut World) -> Self {
        let assets: &mut Mut<Assets<ColorMaterial>> = &mut world.resource_mut();
        // TODO: turn this into a vec and make it dynamic
        let mut colors: Vec<Color> = NEST_COLORS
            .iter()
            .cycle()
            .take(NUM_NESTS)
            .cloned()
            .collect();
        colors.truncate(NUM_NESTS);
        let color_ids = (0..NUM_NESTS).collect();
        let color_handles = colors
            .iter()
            .copied()
            .map(|c| assets.add(ColorMaterial::from(c)))
            .collect();
        Colors {
            colors,
            color_ids,
            color_handles,
        }
    }
}

const MAX_ANTS: u32 = 50;

#[derive(Resource, Deref, DerefMut, Default, Reflect)]
#[reflect(Resource)]
pub struct NumAnts(u32);

#[derive(Resource)]
pub struct BoundingBox {
    pub w: f32,
    pub h: f32,
}

// TODO: impl FromWorld for dynamic BoundingBox
impl Default for BoundingBox {
    fn default() -> Self {
        Self { w: 200.0, h: 200.0 }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Paused,
    Play,
}

pub fn toggle_playing(
    mut commands: Commands,
    mut keys: ResMut<Input<KeyCode>>,
    state: Res<CurrentState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        let new_state = match state.0 {
            GameState::Paused => GameState::Play,
            GameState::Play => GameState::Paused,
        };
        commands.insert_resource(NextState(new_state));
        keys.reset(KeyCode::Space);
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameMode {
    Menu,
    AntNetwork,
    AntWander,
}

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
pub struct HexagonMesh(Handle<Mesh>);

impl FromWorld for HexagonMesh {
    fn from_world(world: &mut World) -> Self {
        let meshes: &mut Mut<Assets<Mesh>> = &mut world.resource_mut();
        let hexagon = meshes.add(shape::Circle::default().into());
        Self(hexagon)
    }
}
