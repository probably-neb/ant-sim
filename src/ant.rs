use std::f32::consts::{PI, TAU};

use crate::{
    pheromones::{Pheromone, PheromoneManager, self},
    ANT_ANIMATION_SPEED, ANT_SCALE, ANT_SPEED, BORDER_PADDING, nest::{NestColors, Nest}
};
use bevy::{ecs::component::Component, prelude::*, render::color::Color, log};
use rand::{seq::IteratorRandom, thread_rng, Rng};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Component)]
pub struct Ant {
    pub target_color: usize,
    pub parent_color: usize,
    pub carrying_food: bool,
    pub orientation: f32,
    pub target_orientation: f32,
    pub turn_around: bool,
    // pub following_trail: bool,
}

impl Ant {
    fn new(target_color: usize, parent_color: usize) -> Self {
        let mut rng = thread_rng();
        let angle = rng.gen_range(0.0..TAU);
        Self {
            target_color,
            parent_color,
            carrying_food: false,
            turn_around: false,
            orientation: angle,
            target_orientation: angle, 
        }
    }
    pub fn set_target_orientation(&mut self, angle: f32) {
        self.target_orientation = angle % TAU;
    }

    pub fn set_orientation(&mut self, angle: f32) {
        self.orientation = angle % TAU;
    }
}

#[derive(Bundle)]
pub struct AntBundle {
    ant: Ant,
    animation_timer: AntAnimationTimer,
    sprite_sheet: SpriteSheetBundle,
}

impl AntBundle {
    pub fn new(
        transform: &Transform,
        target: usize,
        parent: usize,
        ant_texture: &Handle<TextureAtlas>,
    ) -> Self {
        let ant = Ant::new(target, parent);
        return Self {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: ant_texture.clone(),
                transform: transform.with_scale(ANT_SCALE).with_rotation(Quat::from_rotation_z(-ant.orientation)),
                ..default()
            },
            animation_timer: AntAnimationTimer(Timer::from_seconds(
                ANT_ANIMATION_SPEED,
                TimerMode::Repeating,
            )),
            ant,
        };
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AntAnimationTimer(Timer);

#[derive(Resource, Deref)]
pub struct AntTexture(pub Handle<TextureAtlas>);

pub fn animate_ant(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AntAnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

// contains bound checking logic
#[derive(PartialEq, Eq)]
enum Bounds {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

struct Recommendation {
    where_to_point_now: f32,
    where_to_try_and_point: f32,
}

// one could argue this is overly verbose and to that I say "and?"
impl Bounds {
    fn check(pos: Vec3, logical_bounds: Vec2) -> Option<Self> {
        // because 2d is just 3d with z = what should I be on top of in bevy
        let pos: Vec2 = Vec2 { x: pos.x, y: pos.y };
        let diff = logical_bounds - pos.abs();
        let mut collision: Option<Bounds> = None;
        if diff.x < BORDER_PADDING {
            if pos.x < 0.0 {
                collision = Some(Self::LEFT);
            } else {
                collision = Some(Self::RIGHT);
            }
        } else if diff.y < BORDER_PADDING {
            if pos.y < 0.0 {
                collision = Some(Self::UP);
            } else {
                collision = Some(Self::DOWN);
            }
        }
        return collision;
    }
    fn rad(self) -> f32 {
        match self {
            Self::LEFT => PI,
            Self::RIGHT => 0.0,
            Self::UP => PI / 2.0,
            Self::DOWN => (3.0 * PI) / 2.0,
        }
    }

    fn where_should_i_go_instead(self, ant_orientation: f32) -> Recommendation {
        // recommends pointing clockwise with target orientation away from the wall
        // only recommends an immediate change in orientation (as opposed to a target orientation)
        // if the ant is pointing towards the wall
        // FIXME: LEFT match arm has incorrect min/max
        // FIXME: bc bin panics every time LEFT is matched we know the bounds we're checking are
        // way off
        match self {
            Self::LEFT => Recommendation {
                // where_to_point_now: {
                //     if ant_orientation > 3.0*PI/2.0 {
                //         // ant_orientation.clamp(3.0*PI/2.0, TAU)
                //         ant_orientation + PI / 2.
                //     } else {
                //         // ant_orientation.clamp(0.0, PI/2.0)
                //         ant_orientation + PI / 2.
                //     }
                // },
                where_to_point_now: 0.0,
                where_to_try_and_point: 0.0,
            },
            Self::DOWN => Recommendation {
                // point down now
                where_to_point_now: //ant_orientation.clamp(PI,TAU),
                        3.0*PI / 2.,
                where_to_try_and_point: 3.0*PI/2.0,
            },
            Self::RIGHT => Recommendation {
                // point left now
                where_to_point_now: //ant_orientation.clamp(PI/2.0, 3.0*PI/2.0),
                        PI,
                where_to_try_and_point: PI,
            },
            Self::UP => Recommendation {
                // point up now
                where_to_point_now: //ant_orientation.clamp(0.0, PI),
                        PI / 2.,
                where_to_try_and_point: PI/2.0,
            },
        }
    }
}

// the angles ants can set as their target_orientation
const ANG: f32 = PI / 6.;
const CCW_ANG: f32 = ANG;
const CW_ANG: f32 = -ANG;
const STRAIGHT: f32 = 0.;
const OPTS: [f32; 5] = [CCW_ANG, STRAIGHT, STRAIGHT, STRAIGHT, CW_ANG];
const STEP_PERCENT: f32 = 0.4;
// if the ants orientation is within this much of its target orientation it is considered to have
// reached its target orientation
const ACCEPTABLE_ORIENTATION_BOUND: f32 = PI / 48.;
const ANT_NEST_SCAN_RANGE: f32 = 150.;

pub fn move_ant(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Ant)>,
    windows: Res<Windows>,
    pheromone_manager: Query<&PheromoneManager>,
    mut pheromones: Query<&mut Pheromone>,
    time: Res<Time>,
    nests: Query<(Entity,&Nest)>,
    nest_ids: Res<NestColors>
) {
    let mut rng = thread_rng();
    let window = windows.primary();
    let bounds = Vec2 {
        x: window.width() / 2.0,
        y: window.height() / 2.0,
    };
    let pheromone_manager = pheromone_manager
        .get_single()
        .expect("there should be pheromones");

    for (mut transform, mut ant) in &mut query {
        if ant.turn_around {
            ant.orientation += PI;
            ant.target_orientation = ant.orientation;
            transform.rotate_z(PI);
            ant.turn_around = false;
            continue;
        }
        // add pheromones to tile
        let ant_loc = transform.translation.truncate();

        let bounds_situation = Bounds::check(transform.translation, bounds);
        if let Some(bounds_problem) = bounds_situation {
            let recommendation = bounds_problem.where_should_i_go_instead(ant.orientation);
            let old_orientation = ant.orientation;
            ant.set_orientation(recommendation.where_to_point_now);
            // this happens after we use the orientation setter so ant.orientation is modulo 2xPi
            let delta = ant.orientation - old_orientation;
            transform.rotate_z(delta);
            ant.set_target_orientation(recommendation.where_to_try_and_point);
        }

        let mut wander_angle = ANG;
        let nest_id = nest_ids.nests[ant.target_color];
        let (_, nest) = nests.get(nest_id).expect("ant shouldn't target non-existent nest");
        let nest_loc = nest.loc;
        if ant_loc.distance(nest_loc) <= ANT_NEST_SCAN_RANGE {
            let dest_trajectory = -(nest_loc - ant_loc);
            let curr_trajectory = Vec2::from_angle(ant.orientation);
            let delta = -curr_trajectory.angle_between(dest_trajectory);
            let new_orientation = ant.orientation + delta;
            ant.set_target_orientation(new_orientation);
            log::info!("ant within range of nest");
            wander_angle = 0.0;
        }


        let adjacent_pheromones =
            pheromone_manager.ids_of_adjacent_pheromones(ant.orientation, ant_loc);
        let strongest_pheromone = adjacent_pheromones
            .iter()
            .map(|(e,v)| (pheromones.get(*e).expect("adjacent_pheromones should exist"),v))
            .filter_map(|(p,v)| {
                let weight = p.weights[ant.target_color as usize];
                if weight <= f32::EPSILON {
                    return None;
                } else {
                    return Some((weight, v));
                }
            })
            .max_by_key(|(w,v)| (*w*100.0) as usize)
            .map(|(_,v)| v);

        let mut diff = ant.target_orientation - ant.orientation;
        if let Some(destination) = strongest_pheromone {
            let dest_trajectory = -(*destination - ant_loc);
            let curr_trajectory = Vec2::from_angle(ant.orientation);
            let delta = curr_trajectory.angle_between(dest_trajectory);
            // let mut new_orientation = ant.orientation;
            if !delta.is_nan() && delta.abs() != 0.0 {
                let new_orientation = ant.orientation+delta;
                // transform.rotate_z(delta);
                // ant.set_orientation(new_orientation);
                ant.set_target_orientation(new_orientation);
                // log::info!("redirected ant with target {} by {}deg", ant.target_color, delta.to_degrees());
            }
            // current trajectory is correct?
            // else {
                // ant.set_target_orientation(new_orientation);
            // }
        }
        if diff.abs() <= ACCEPTABLE_ORIENTATION_BOUND && wander_angle != 0.0 {
            let new_target = rng.gen_range(-wander_angle..wander_angle);
            let new_orientation = ant.orientation + new_target;
            ant.set_target_orientation(new_orientation);
            diff = ant.target_orientation - ant.orientation;
        }

        let pheromone_tile =
            pheromone_manager.id_of_pheromone_at(ant_loc, window.width(), window.height());
        let mut pheromone = pheromones
            .get_mut(pheromone_tile)
            .expect("pheromone manager shouldn't have bastards");
        // to find our way home
        pheromone.add_trail(ant.target_color, ant.target_color);
        commands.entity(pheromone_tile).insert(pheromones::NonEmptyTrail);
        // pheromone.add_trail(ant.parent_color);
        // TODO: system to colorize pheromones

        let rotation = diff * STEP_PERCENT;
        ant.orientation += rotation;
        ant.orientation %= 2.0 * PI;
        transform.rotate_z(rotation);
        let delta_time = f32::min(0.2, time.delta_seconds());
        transform.translation.x += delta_time*ANT_SPEED * ant.orientation.cos();
        transform.translation.y += delta_time*ANT_SPEED * ant.orientation.sin();
    }
}

pub fn load_ant_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("ant.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(202.0, 250.0), 8, 7, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(AntTexture(texture_atlas_handle));
}
