pub mod ant;
pub mod nest;
pub mod pheromones;
pub mod food;

use std::time::Duration;

use crate::{Colors, GameState, GameMode, NumAnts};
use bevy::prelude::*;
use iyes_loopless::prelude::*;

#[cfg(feature = "debug")]
use bevy_inspector_egui::{InspectorPlugin, WorldInspectorPlugin, RegisterInspectable};

pub struct AntNetworkPlugin;

impl Plugin for AntNetworkPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<Colors>()
        .init_resource::<NumAnts>()
        .init_resource::<DecisionWeights>()
        .init_resource::<PheromoneParams>()
        .add_startup_system(pheromones::create_pheromone_manager)
        .add_startup_system(nest::spawn_nests)
        .add_startup_system(ant::load_ant_texture)
        .add_fixed_timestep(
            Duration::from_millis(250),
            "color timestep"
       )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Play)
                .run_in_state(GameMode::AntNetwork)
                .with_system(nest::food_request_system)
                // .with_system(ant::move_ant_network.label("move ants"))
                // .with_system(pheromones::color_and_fade_pheromones.label("color pheromones").after("move ants"))
                // .with_system(pheromones::color_and_fade_pheromones)
                .with_system(nest::fade_nest_network_pheremones)
                .with_system(pheromones::fade_pheromones)
                .into()
            )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Play)
                .run_in_state(GameMode::AntNetwork)
                .label("collisions")
                .before("move ants")
                .with_system(nest::ant_nest_network_interactions)
                .into()
            )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Play)
                .run_in_state(GameMode::AntNetwork)
                .label("move ants")
                .after("collisions")
                .with_system(ant::move_ant)
                .into()
            )
        .add_fixed_timestep_system(
            "color timestep", 0,
            pheromones::create_required_pheromones
            .run_in_state(GameState::Play)
            .run_in_state(GameMode::AntNetwork)
        )
        .add_fixed_timestep_child_stage("color timestep")
        .add_fixed_timestep_system(
            "color timestep", 1,
            pheromones::leave_pheromone_trails
            .run_in_state(GameState::Play)
            .run_in_state(GameMode::AntNetwork)
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
        use crate::network::pheromones::{PheromoneManager, PheromoneGrid};
        #[cfg(feature = "debug")]
        app.add_plugin(WorldInspectorPlugin::new())
            .register_inspectable::<PheromoneManager>()
            .register_inspectable::<PheromoneGrid>();

        #[cfg(feature = "debug")]
        app.add_plugin(InspectorPlugin::<DecisionWeights>::new());

        #[cfg(feature = "debug")]
        app.add_plugin(InspectorPlugin::<PheromoneParams>::new());

        #[cfg(feature = "debug")]
        let mut registry = app
            .world
            .get_resource_mut::<InspectableRegistry
    }
}

const DISTANCE_POW: f32 = 1.2;
const PHEROMONE_POW: f32 = 4.;
const VISITED_POW: f32 = 2.;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug,Clone,Resource,)]
pub struct DecisionWeights {
    pub distance_pow: f32,
    pub pheromone_pow: f32,
    pub visited_pow: f32,
}

impl Default for DecisionWeights {
    fn default() -> Self {
        return Self {
            distance_pow: DISTANCE_POW,
            pheromone_pow: PHEROMONE_POW,
            visited_pow: VISITED_POW,
        }
    }
}


const TRAIL_PHEROMONE_STEP: f32 = 0.10;
const TRAIL_PHEROMONE_FADE_RATE: f32 = 0.001;
const NEST_PHEROMONE_FADE_SPEED: f32 = 0.03;
const NEST_PHEROMONE_STEP: f32 = 0.1;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug,Clone,Resource,)]
pub struct PheromoneParams {
    pub trail_step: f32,
    pub nest_step: f32,
    pub trail_fade_rate: f32,
    pub nest_fade_rate: f32,
}

impl Default for PheromoneParams {
    fn default() -> Self {
        return Self {
    trail_step: TRAIL_PHEROMONE_STEP,
    nest_step: NEST_PHEROMONE_STEP,
    trail_fade_rate: TRAIL_PHEROMONE_FADE_RATE,
    nest_fade_rate: NEST_PHEROMONE_FADE_SPEED,
        }
    }
}
