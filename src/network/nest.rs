use crate::{
    Colors, NumAnts, BORDER_PADDING, FOOD_HEIGHT, FOOD_SIZE_V3, MAX_ANTS,
    NEST_FOOD_REQUEST_PROB, NEST_HEIGHT, NEST_SIZE, NUM_NESTS, NEST_SPREAD,
};

use bevy::{
    ecs::{component::Component, system::Query},
    log,
    prelude::*,
    sprite::{collide_aabb::collide, ColorMaterial, MaterialMesh2dBundle},
    utils::default,
};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};

use super::{ant, food::Food};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Component, Clone)]
pub struct Nest {
    pub color: usize,
    pub color_weights: Vec<f32>,
    pub loc: Vec2,
}

impl Nest {
    pub fn new(color: usize, loc: Vec2) -> Nest {
        let mut color_weights = Vec::with_capacity(NUM_NESTS);
        for _i in 0..NUM_NESTS {
            color_weights.push(0.0);
        }
        return Self {
            color,
            loc,
            color_weights,
        };
    }
    #[inline]
    pub fn step_pheromone(&mut self, color: usize) {
        let mut weight = self.color_weights[color];
        weight += ant::SYSTEM_PHEROMONE_GROW_SPEED;
        weight = weight.min(1.0);
        self.color_weights[color] = weight;
    }
    // TODO: pheromone component
    #[inline]
    pub fn step_pheromones(&mut self, target_color: usize, parent_color: usize) {
        self.step_pheromone(target_color);
        self.step_pheromone(parent_color);
    }

    pub fn fade(&mut self) {
        for w in self.color_weights.iter_mut() {
            *w -= ant::SYSTEM_PHEROMONE_FADE_SPEED;
            *w = w.max(0.0);
        }
    }
}

pub fn food_request_system(
    mut commands: Commands,
    query: Query<(&Nest, &Transform)>,
    ant_texture: Res<ant::AntTexture>,
    mut num_ants: ResMut<NumAnts>,
    colors: Res<Colors>,
) {
    let mut rng = rand::thread_rng();
    for (nest, transform) in &query {
        // let color = nest.color;
        // PERF: Bernoulli distribution resource will be more efficien
        let should_ask_for_food: bool =
            num_ants.0 != MAX_ANTS && rng.gen_bool(NEST_FOOD_REQUEST_PROB as f64);
        if should_ask_for_food {
            let target_color = colors
                .color_ids
                .iter()
                .filter(|c| **c != nest.color)
                .choose(&mut rng)
                .unwrap();
            commands.spawn(ant::AntBundle::new(
                transform,
                *target_color,
                nest.color,
                &ant_texture,
            ));
            log::info!(
                "generated ant: nest {:?} target: {:?}",
                nest.color,
                target_color
            );
            num_ants.0 += 1;
        }
    }
}

#[derive(Resource)]
pub struct NestColors {
    pub nests: Vec<Entity>,
}

fn gen_hex_coords(w: u32, h: u32) -> Vec<Vec2> {
    let mut coords = Vec::with_capacity((w * h) as usize);
    // step by 2
    for x in (0..w).step_by(2) {
        // step by 1
        for y in 0..h {
            let x = x as f32;
            let y = y as f32;
            coords.push(Vec2 { x, y });
        }
    }
    return coords;
}


pub fn spawn_nests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    colors: Res<Colors>,
    windows: Res<Windows>,
) {
    let mut rng = rand::thread_rng();
    let window = windows.primary();
    let bounds = Vec2 {
        x: window.width(),
        y: window.height(),
    } - 2. * BORDER_PADDING;
    let sprite_size = Vec3::new(NEST_SIZE, NEST_SIZE, 0.);
    let mut nests = Vec::with_capacity(NUM_NESTS);
    for i in 0..NUM_NESTS {
        // temp value
        let e = Entity::from_raw(i as u32);
        nests.push(e);
    }
    let hex_bounds = (bounds / NEST_SPREAD).as_uvec2();
    let mut coords: Vec<Vec2> = gen_hex_coords(hex_bounds.x, hex_bounds.y)
        .iter()
        .map(|&v| -(bounds / 2.) + (v * NEST_SPREAD))
        .collect();
    coords.as_mut_slice().shuffle(&mut rng);
    println!("coords: {:?}", &coords);
    // let color = Color::rgba(0., 0., 0., 0.);
    // FIXME:
    for (color, color_id) in colors.iter() {
        let c = &coords
            .pop()
            .expect("NUM_NESTS should always be > num hex coords");
        let nest_loc = Vec3::new(c.x, c.y, NEST_HEIGHT as f32);
        let id = commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::default().into()).into(),
                    material: materials.add(ColorMaterial::from(color)),
                    transform: Transform::from_translation(nest_loc).with_scale(sprite_size),
                    ..default()
                },
                Nest::new(color_id, nest_loc.truncate()),
            ))
            .id();
        nests[color_id] = id;
    }
    commands.insert_resource(NestColors { nests });
}

#[inline]
fn pos_size(t: Transform) -> (Vec3, Vec2) {
    let pos = t.translation;
    let size = t.scale.truncate();
    return (pos, size);
}

pub fn fade_nest_network_pheremones(mut nests: Query<&mut Nest>) {
    for mut nest in &mut nests {
        nest.fade();
    }
}

const FOOD_OFFSET: Vec3 = Vec3 {x: 0., y: 80., z: FOOD_HEIGHT as f32};

pub fn ant_nest_network_interactions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    colors: Res<Colors>,
    mut ants: Query<(Entity, &mut ant::Ant, &Transform)>,
    mut nests: Query<(&Nest, &Transform)>,
) {
    for (nest, nest_transform) in &mut nests {
        for (ant_id, mut ant, ant_transform) in &mut ants {
            let (nest_pos, nest_size) = pos_size(*nest_transform);
            let (ant_pos, ant_size) = pos_size(*ant_transform);

            // skip ants we already updated
            if nest.color == ant.prev_nest() {
                continue;
            }
            let collision = collide(nest_pos, nest_size * 2., ant_pos, ant_size);
            match collision {
                Some(_) => {
                    if ant.target_color == nest.color {
                        if !ant.carrying_food {
                            // commands.entity(ant_id).add_child(food_id);
                            commands.entity(ant_id).with_children(|builder| {
                                builder.spawn((
                                    MaterialMesh2dBundle {
                                        mesh: meshes.add(shape::Circle::default().into()).into(),
                                        material: colors.color_handles[nest.color].clone(),
                                        transform: Transform::from_translation(FOOD_OFFSET)
                                        .with_scale(FOOD_SIZE_V3),
                                        visibility: Visibility { is_visible: true },
                                        ..default()
                                    },
                                    Food::new(nest.color),
                                ));
                            });
                            ant.target_color = ant.parent_color;
                            // not parent but this will cause to and from pheromone trails
                            // to be set on the way to target and on the way back
                            ant.parent_color = nest.color;
                            ant.carrying_food = true;
                            log::info!("Ant reached target nest {} after {} steps", nest.color,ant.prev_nests.len());
                        } else {
                            // despawn food
                            commands.entity(ant_id).despawn_descendants();
                            ant.target_color = ant.parent_color;
                            ant.parent_color = nest.color;
                            ant.carrying_food = false;
                            log::info!("Ant reached parent nest {} after {} steps", nest.color,ant.prev_nests.len());
                        }
                        // let orientation = ant.orientation + PI;
                        // ant.set_orientation(orientation);
                        // ant.set_target_orientation(orientation);
                        // ant.turn_around = true;
                    } else {
                        // log::info!(
                        //     "ant heading to {} taking pit stop at {}",
                        //     ant.target_color,
                        //     nest.color
                        // );
                        // figure out jump point
                        //
                        ant.visit_nest(nest.color);
                    }
                }
                None => continue,
            }
        }
    }
}

pub fn ant_nest_interactions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    colors: Res<Colors>,
    mut ants: Query<(Entity, &mut ant::Ant, &Transform)>,
    mut nests: Query<(&Nest, &Transform)>,
) {
    for (nest, nest_transform) in &mut nests {
        for (ant_id, mut ant, ant_transform) in &mut ants {
            if nest.color == ant.target_color {
                let (nest_pos, nest_size) = pos_size(*nest_transform);
                let (ant_pos, ant_size) = pos_size(*ant_transform);

                let collision = collide(nest_pos, nest_size * 2., ant_pos, ant_size);
                match collision {
                    Some(_) => {
                        if !ant.carrying_food {
                            let food_id = commands
                                .spawn((
                                    MaterialMesh2dBundle {
                                        mesh: meshes.add(shape::Circle::default().into()).into(),
                                        material: colors.color_handles[nest.color].clone(),
                                        transform: Transform::from_translation(Vec3 {
                                            x: ant_pos.x,
                                            y: ant_pos.y,
                                            z: FOOD_HEIGHT as f32,
                                        })
                                        .with_scale(FOOD_SIZE_V3),
                                        visibility: Visibility { is_visible: true },
                                        ..default()
                                    },
                                    Food::new(nest.color),
                                ))
                                .id();
                            commands.entity(ant_id).add_child(food_id);
                            ant.target_color = ant.parent_color;
                            // not parent but this will cause to and from pheromone trails
                            // to be set on the way to target and on the way back
                            ant.parent_color = nest.color;
                            ant.carrying_food = true;
                            // TODO: reverse ant orientation or something
                            log::info!(
                                "Ant reached nest {}. returning to {}",
                                nest.color,
                                ant.target_color
                            );
                            // let orientation = ant.orientation + PI;
                            // ant.set_orientation(orientation);
                            // ant.set_target_orientation(orientation);
                            ant.turn_around = true;
                        } else {
                            commands.entity(ant_id).despawn_descendants();
                            log::info!("Ant reached target {}", nest.color);
                            ant.target_color = ant.parent_color;
                            ant.parent_color = nest.color;
                            ant.carrying_food = false;
                            ant.turn_around = true;
                        }
                    }
                    None => continue,
                }
            }
        }
    }
}
