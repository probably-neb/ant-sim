pub mod ant;
pub mod nest;
pub mod pheromones;
pub mod food;

use crate::{Colors, GameState, GameMode, NumAnts};
use bevy::prelude::*;
use iyes_loopless::prelude::*;

pub struct AntNetworkPlugin;

impl Plugin for AntNetworkPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<Colors>()
        .init_resource::<NumAnts>()
        .add_startup_system(pheromones::create_pheromone_manager)
        .add_startup_system(nest::spawn_nests)
        .add_startup_system(ant::load_ant_texture)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Play)
                .run_in_state(GameMode::AntNetwork)
                .with_system(nest::food_request_system)
                // .with_system(ant::move_ant_network.label("move ants"))
                // .with_system(pheromones::color_and_fade_pheromones.label("color pheromones").after("move ants"))
                .with_system(ant::move_ant)
                .with_system(pheromones::color_and_fade_pheromones)
                .with_system(nest::ant_nest_network_interactions)
                .with_system(nest::fade_nest_network_pheremones)
                .into()
            )
        // .add_system_set(
        //     ConditionSet::new()
            // .run_in_state(GameState::Paused)
        //     .run_in_state(GameMode::AntNetwork)
        //     .with_system(ant::animate_ant)
        //     .into()
                       // )
                       ;
        // .add_system(ant::ant_wander)
        // .add_system(pheromones::print_angle)
        // .add_system(print_camera)
    }
}
