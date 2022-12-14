use std::{
    collections::VecDeque,
    f32::consts::{FRAC_PI_2, TAU},
};

use crate::{ANT_ANIMATION_SPEED, ANT_SCALE, ANT_SPEED, BORDER_PADDING, NUM_NESTS};

use super::{
    nest::{Nest, NestColors},
    DecisionWeights, PheromoneParams,
};

use bevy::{ecs::component::Component, log, prelude::*};
use rand::{distributions::WeightedIndex, prelude::*, thread_rng, Rng};

#[derive(Debug, Component)]
pub struct Ant {
    pub target_color: usize,
    pub parent_color: usize,
    pub carrying_food: bool,
    pub orientation: f32,
    pub target_orientation: f32,
    pub turn_around: bool,
    // pub has_target: bool,
    pub current_nest: Option<usize>,
    pub prev_nests: VecDeque<usize>,
    pub steps: usize,
}

impl Ant {
    fn new(target_color: usize, parent_color: usize) -> Self {
        let mut rng = thread_rng();
        let angle = rng.gen_range(0.0..TAU);
        // let angle = FRAC_PI_2;
        let mut prev_nests = VecDeque::with_capacity(NUM_NESTS);
        prev_nests.push_front(parent_color);
        Self {
            target_color,
            parent_color,
            carrying_food: false,
            turn_around: false,
            orientation: angle,
            target_orientation: angle,
            current_nest: Some(parent_color),
            prev_nests,
            steps: 0,
        }
    }

    #[inline]
    pub fn set_target_orientation(&mut self, angle: f32) {
        self.target_orientation = angle % TAU;
    }

    #[inline]
    pub fn set_orientation(&mut self, angle: f32) {
        self.orientation = angle % TAU;
    }

    #[inline]
    fn rotate_hard(&mut self, t: &mut Transform, delta: f32) {
        t.rotate_z(delta);
        let new_orientation = self.orientation + delta;
        self.set_orientation(new_orientation);
        self.set_target_orientation(new_orientation);
    }

    #[inline]
    pub fn visit_nest(&mut self, color: usize) {
        assert!(self.current_nest.is_none());
        self.prev_nests.push_front(color);
        self.current_nest = Some(color);
    }

    #[inline]
    pub fn leave_nest(&mut self) {
        self.prev_nests.truncate(NUM_NESTS);
        self.current_nest = None;
        self.steps += 1;
    }

    pub fn prev_nest(&self) -> usize {
        return *self
            .prev_nests
            .front()
            .expect("prev nests shouldn't be empty");
    }

    pub fn pop_prev_nest(&mut self) {
        self.current_nest = Some(self.prev_nest());
    }

    pub fn wipe_mem(&mut self) {
        self.steps = 0;
        self.prev_nests.truncate(1);
    }
}

#[derive(Bundle)]
pub struct AntBundle {
    ant: Ant,
    animation_timer: AntAnimationTimer,
    sprite_sheet: SpriteSheetBundle,
    pheromone_timer: AntPheromoneTimer,
}

const ANT_DROP_VISIBLE_PHEROMONE_SPEED: f32 = 0.2;
impl AntBundle {
    pub fn new(
        transform: &Transform,
        target: usize,
        parent: usize,
        ant_texture: &Handle<TextureAtlas>,
    ) -> Self {
        let ant = Ant::new(target, parent);
        let q = Quat::from_rotation_z(ant.orientation - FRAC_PI_2);
        // log::info!(
        //     "Quat {:?} going from {} to {}",
        //     q.to_axis_angle(),
        //     FRAC_PI_2,
        //     ant.orientation
        // );
        Self {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: ant_texture.clone_weak(),
                transform: transform.with_scale(ANT_SCALE).with_rotation(q),
                ..default()
            },
            animation_timer: AntAnimationTimer(Timer::from_seconds(
                ANT_ANIMATION_SPEED,
                TimerMode::Repeating,
            )),
            pheromone_timer: AntPheromoneTimer(Timer::from_seconds(
                ANT_DROP_VISIBLE_PHEROMONE_SPEED,
                TimerMode::Repeating,
            )),
            ant,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AntAnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct AntPheromoneTimer(Timer);

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
    Left,
    Right,
    Up,
    Down,
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
                collision = Some(Self::Left);
            } else {
                collision = Some(Self::Right);
            }
        } else if diff.y < BORDER_PADDING {
            if pos.y < 0.0 {
                collision = Some(Self::Up);
            } else {
                collision = Some(Self::Down);
            }
        }
        collision
    }
}

pub fn move_ant(
    mut ants: Query<(&mut Transform, &mut Ant)>,
    time: Res<Time>,
    mut nests: Query<(Entity, &mut Nest)>,
    nest_ids: Res<NestColors>,
    decision_weights: Res<DecisionWeights>,
    windows: Res<Windows>,
    pher_params: Res<PheromoneParams>,
) {
    let mut rng = thread_rng();

    let win = windows.primary();
    let bounds = Vec2 {
        x: win.width(),
        y: win.height(),
    };

    for (mut transform, mut ant) in &mut ants {
        let ant_loc = transform.translation.truncate();
        let bounds_situation = Bounds::check(transform.translation, bounds / 2.0);

        if let Some(_bounds_problem) = bounds_situation {
            //
            ant.pop_prev_nest();
        }
        if let Some(current_nest_color) = ant.current_nest {
            let mut weights = vec![0.0; NUM_NESTS];
            let mut cur_id = None;
            for (id, nest) in &nests {
                if nest.color == current_nest_color {
                    cur_id = Some(id);
                    continue;
                }
                let mut distance_factor = ant_loc.distance(nest.loc);

                let mut pheromone_factor = nest.color_weights[ant.target_color].max(1.0);

                // how recently we visited this nest
                // TODO:
                let mut visited_factor = ant
                    .prev_nests
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &c)| {
                        if c == nest.color {
                            // weight by index
                            Some(1. / (i as f32 + 1.0))
                        } else {
                            None
                        }
                    })
                    .sum::<f32>()
                    // ensure no div by 0
                    .max(1.0);

                distance_factor = (1.0 / distance_factor).powf(decision_weights.distance_pow);
                pheromone_factor = pheromone_factor.powf(decision_weights.pheromone_pow);
                // make it less likely to visit ones we've been too recently
                visited_factor = (1.0 / visited_factor).powf(decision_weights.visited_pow);
                // log::info!("factors: dist: {} pher: {} visit: {}", distance_factor, pheromone_factor, visited_factor);
                let mut factors = vec![distance_factor, pheromone_factor, visited_factor];

                for factor in &mut factors {
                    if !factor.is_finite() {
                        *factor = f32::MAX;
                    }
                }
                weights[nest.color] = factors.iter().product();
                weights[current_nest_color] = 0.0;
            }
            let tot: f32 = weights.iter().sum();
            weights = weights.iter().map(|v| v / tot).collect();
            let dist = WeightedIndex::new(&weights).unwrap();
            let next_nest_color = dist.sample(&mut rng);
            if next_nest_color == current_nest_color {
                log::warn!("chose same nest");
                continue;
            }
            let next_nest_id = nest_ids.nests[next_nest_color];
            let next_nest = nests.get(next_nest_id).unwrap().1;
            let next_nest_loc: Vec2 = next_nest.loc;
            let curr_trajectory = Vec2::from_angle(ant.orientation);
            let new_trajectory = next_nest_loc - ant_loc;
            let delta = curr_trajectory.angle_between(new_trajectory);

            // let new_orientation = ant.orientation + delta;
            ant.rotate_hard(&mut transform, delta);
            // transform.rotate_z(delta);
            // ant.set_orientation(new_orientation);
            // ant.set_target_orientation(new_orientation);

            let mut nest_component = nests.get_mut(cur_id.unwrap()).unwrap().1;
            // leave memory of where we were going and where we came from
            nest_component.step_pheromone(ant.parent_color, pher_params.nest_step);
            ant.leave_nest();
        }

        let delta_time = f32::min(0.2, time.delta_seconds());
        transform.translation.x += delta_time * ANT_SPEED * ant.orientation.cos();
        transform.translation.y += delta_time * ANT_SPEED * ant.orientation.sin();
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
