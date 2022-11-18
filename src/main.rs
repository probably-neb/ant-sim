use bevy::sprite::MaterialMesh2dBundle;
use rand::Rng;
use bevy::prelude::*;
use ant_sim::*;
use ant_sim::nest::Nest;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Ant sim".to_string(),
                width: WINDOW_SIZE_X,
                height: WINDOW_SIZE_Y,
                ..Default::default()
            },
            ..default()
        }))
        .add_startup_system(setup_camera)
        .add_startup_system(setup_map)
        .add_startup_system(nest::spawn_nests)
        .add_startup_system(ant::load_ant_texture)
        .add_system(nest::food_request_system)
        .add_system(ant::animate_ant)
        .add_system(ant::move_ant)
        .run();
}

fn setup_camera(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());
}

fn setup_map(mut commands: Commands) {
    spawn_map(&mut commands);
}

#[derive(Debug, Clone, Component, Deref)]
struct NeumannCell {
    coords: IVec2,
}

impl NeumannCell {
    fn new(coords: IVec2) -> Self {
        Self { coords }
    }
}

fn spawn_map(commands: &mut Commands) {
    let mut rng = rand::thread_rng();
    let (size_x, size_y) = (300, 200);
    let sprite_size = 4.;
    let color = Color::rgba(0., 0., 0., 0.);

    commands
        .spawn(SpatialBundle::from_transform(Transform::from_xyz(
            -(size_x as f32 * sprite_size) / 2.,
            -(size_y as f32 * sprite_size) / 2.,
            BOARD_HEIGHT as f32,
        )))
        .with_children(|builder| {
            for y in 0..=size_y {
                for x in 0..=size_x {
                    builder.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(sprite_size)),
                                color,
                                ..Default::default()
                            },
                            transform: Transform::from_xyz(
                                sprite_size * x as f32,
                                sprite_size * y as f32,
                                BOARD_HEIGHT as f32,
                            ),
                            ..Default::default()
                        },
                        NeumannCell::new(IVec2::new(x, y)),
                    ));
                }
            }
        });
    println!("map generated");
}

// pub fn color_sprites(
//     mut query: Query<(&RockPaperScissor, &mut Sprite), Changed<RockPaperScissor>>,
// ) {
//     for (state, mut sprite) in query.iter_mut() {
//         match state {
//             RockPaperScissor::Rock => sprite.color = Color::BLUE,
//             RockPaperScissor::Paper => sprite.color = Color::BEIGE,
//             RockPaperScissor::Scissor => sprite.color = Color::RED,
//         }
//     }
// }
