pub use bevy::{prelude::*, render::color::Color};
pub mod food;
pub use food::*;
pub mod ant;
pub use ant::*;
pub mod nest;
pub use nest::*;

pub const BOARD_HEIGHT: usize = 0;
pub const NEST_HEIGHT: usize = 1;
pub const ANT_HEIGHT: usize = 2;
pub const FOOD_HEIGHT: usize = 3;

pub const FOOD_SIZE: f32 = 4.;
pub const FOOD_SIZE_V3: Vec3 = Vec3::new(FOOD_SIZE, FOOD_SIZE, FOOD_HEIGHT as f32);

pub const ANT_SPEED: f32 = 5.0;
pub const ANT_SIZE: f32 = 0.0625;
pub const ANT_SCALE: Vec3 = Vec3::new(ANT_SIZE, ANT_SIZE, ANT_HEIGHT as f32);
pub const ANT_ANIMATION_SPEED: f32 = 0.06;

pub const NUM_NESTS: usize = 5;
pub const WINDOW_SIZE: f32 = 800.;
pub const WINDOW_SIZE_X: f32 = WINDOW_SIZE;
pub const WINDOW_SIZE_Y: f32 = WINDOW_SIZE;

pub const NEST_SIZE: f32 = 32.;
pub const NEST_FOOD_REQUEST_PROB: f32 = 0.05;
pub const NEST_COLORS: [Color; NUM_NESTS] = [
    Color::rgb(1.0, 0.745, 0.0431),
    Color::rgb(0.984, 0.337, 0.027),
    Color::rgb(1.0, 0.0, 0.431),
    Color::rgb(0.513, 0.219, 0.925),
    Color::rgb(0.227, 0.525, 1.0),
];
