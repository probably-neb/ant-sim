use crate::{NEST_COLORS, NEST_SIZE, NEST_HEIGHT, ant, food::Food, NEST_FOOD_REQUEST_PROB, FOOD_SIZE_V3, WINDOW_SIZE_X, WINDOW_SIZE_Y};
use bevy::{
    ecs::{component::Component, system::Query},
    prelude::{*,shape::Circle},
    render::color::Color,
    sprite::{ColorMaterial, MaterialMesh2dBundle},
    utils::default,
};
use rand::{Rng, seq::IteratorRandom};

#[derive(Debug, Clone, Copy, Component)]
pub struct Nest {
    pub color: Color,
}

impl Nest {
    pub fn new(color: Color) -> Nest {
        return Self { color };
    }
    pub fn take_food() {
        todo!()
    }
}

pub fn food_request_system(
    mut commands: Commands,
    query: Query<(&Nest, &Transform, &GlobalTransform)>,
    ant_texture: Res<ant::AntTexture>,
) {
    let mut rng = rand::thread_rng();
    for (&nest, transform, global_transform) in query.iter() {
        // let color = nest.color;
        // PERF: Bernoulli distribution resource will be more efficien
        let should_ask_for_food: bool = rng.gen_bool(NEST_FOOD_REQUEST_PROB as f64);
        if should_ask_for_food {
            let target_color = NEST_COLORS.iter().filter(|c| **c != nest.color).choose(&mut rng).unwrap();
            commands.spawn(ant::AntBundle::new(transform, *target_color, nest.color, &ant_texture));
        }
    }
}

pub fn spawn_nests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    _: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();
    let sprite_size = Vec3::new(NEST_SIZE, NEST_SIZE, 0.);
    // let color = Color::rgba(0., 0., 0., 0.);
    for color in NEST_COLORS {
        let size_x = WINDOW_SIZE_X / 2.0 * 0.95;
        let size_y = WINDOW_SIZE_Y / 2.0 * 0.95;
        let x: f32 = rng.gen_range(-size_x..size_x);
        let y: f32 = rng.gen_range(-size_y..size_y);
        let nest_loc = Vec3::new(x, y, NEST_HEIGHT as f32);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_translation(nest_loc).with_scale(sprite_size),
                ..default()
            },
            Nest::new(color),
        ));
    }

}

