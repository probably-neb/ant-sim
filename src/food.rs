use bevy::{
    render::color::Color,
    ecs::component::Component,
    prelude::Commands,
};
use rand::{thread_rng,Rng};
use crate::NEST_FOOD_REQUEST_PROB;

#[derive(Debug, Clone, Copy, Component)]
pub struct Food {
    pub color: Color
}

impl Food {
    pub fn new(color: Color) -> Self {
        return Self { color };
    }
}
