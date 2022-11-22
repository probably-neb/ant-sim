pub use bevy::{prelude::*, render::color::Color};
pub mod food;
pub mod ant;
pub mod nest;
pub mod pheromones;

pub const BOARD_HEIGHT: usize = 0;
pub const NEST_HEIGHT: usize = 1;
pub const ANT_HEIGHT: usize = 2;
pub const FOOD_HEIGHT: usize = 3;

pub const FOOD_SIZE: f32 = 64.;
pub const FOOD_SIZE_V3: Vec3 = Vec3::new(FOOD_SIZE, FOOD_SIZE, FOOD_HEIGHT as f32);

pub const ANT_SPEED: f32 = 64.;
pub const ANT_SIZE: f32 = 0.0625;
pub const ANT_SCALE: Vec3 = Vec3::new(ANT_SIZE, ANT_SIZE, ANT_HEIGHT as f32);
pub const ANT_ANIMATION_SPEED: f32 = 0.1;

pub const WINDOW_SIZE: f32 = 800.;
pub const WINDOW_SIZE_X: f32 = WINDOW_SIZE;
pub const WINDOW_SIZE_Y: f32 = WINDOW_SIZE;

pub const NUM_NESTS: usize = 2;
pub const NEST_SIZE: f32 = 64.;
pub const NEST_FOOD_REQUEST_PROB: f32 = 0.05;
// TODO: make colors a resource
pub const NEST_COLORS: [Color; 5] = [
    Color::rgb(1.0, 0.745, 0.0431),
    Color::rgb(0.984, 0.337, 0.027),
    Color::rgb(1.0, 0.0, 0.431),
    Color::rgb(0.513, 0.219, 0.925),
    Color::rgb(0.227, 0.525, 1.0),
];

// allowed distance to edge of screen
const BORDER_PADDING: f32 = 50.0;


#[derive(Resource)]
pub struct Colors {
    pub colors: Vec<Color>,
    pub color_ids: Vec<usize>
}

use std::iter::{zip, Zip};
use std::vec::IntoIter;

impl Colors {
    pub fn iter(&self) -> Zip<IntoIter<Color>, IntoIter<usize>> {
        return zip(self.colors.clone(), self.color_ids.clone());
    }
}

impl Default for Colors {
    fn default() -> Self {
        // TODO: turn this into a vec and make it dynamic
        let mut colors = NEST_COLORS.to_vec();
        colors.truncate(NUM_NESTS);
        let color_ids = (0..NUM_NESTS).collect();
        Colors {colors,color_ids}
    }
}


const MAX_ANTS:u32 = 25;

#[derive(Resource,Deref,DerefMut,Default)]
pub struct NumAnts(u32);

#[derive(Resource)]
pub struct BoundingBox {
    pub w: f32,
    pub h: f32
}

// TODO: impl FromWorld for dynamic BoundingBox
impl Default for BoundingBox {
    fn default() -> Self {
        return Self {
            w: 200.0,
            h: 200.0,
        }
    }
}
