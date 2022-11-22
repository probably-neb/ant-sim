use std::f32::consts::PI;

use crate::{
    ant, food::Food, pheromones::PheromoneManager, Colors, NumAnts, BORDER_PADDING, FOOD_HEIGHT,
    FOOD_SIZE_V3, MAX_ANTS, NEST_COLORS, NEST_FOOD_REQUEST_PROB, NEST_HEIGHT, NEST_SIZE, NUM_NESTS,
    WINDOW_SIZE_X, WINDOW_SIZE_Y,
};
use bevy::{
    ecs::{component::Component, system::Query},
    log,
    prelude::{shape::Circle, *},
    sprite::{collide_aabb::collide, ColorMaterial, MaterialMesh2dBundle},
    utils::default,
};
use rand::{seq::IteratorRandom, Rng};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, Component)]
pub struct Nest {
    pub color: usize,
    pub loc: Vec2,
}

impl Nest {
    pub fn new(color: usize, loc: Vec2) -> Nest {
        return Self { color, loc };
    }
    pub fn take_food() {
        todo!()
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
    for (&nest, transform) in query.iter().take(1) {
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

pub fn spawn_nests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    colors: Res<Colors>,
) {
    let mut rng = rand::thread_rng();
    let sprite_size = Vec3::new(NEST_SIZE, NEST_SIZE, 0.);
    let mut nests = Vec::with_capacity(NUM_NESTS);
    for i in 0..NUM_NESTS {
        // temp value
        let e = Entity::from_raw(i as u32);
        nests.push(e);
    }
    // let color = Color::rgba(0., 0., 0., 0.);
    // FIXME:
    for (color, color_id) in colors.iter() {
        let size_x = WINDOW_SIZE_X / 2.0 - BORDER_PADDING;
        let size_y = WINDOW_SIZE_Y / 2.0 - BORDER_PADDING;
        let x: f32 = rng.gen_range(-size_x..size_x);
        let y: f32 = rng.gen_range(-size_y..size_y);
        let nest_loc = Vec3::new(x, y, NEST_HEIGHT as f32);
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

pub fn ant_nest_interactions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
                                        material: materials.add(ColorMaterial::from(
                                            colors.colors[nest.color as usize],
                                        )),
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
                            log::info!("Ant reached final destination {}. despawning", nest.color);
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
