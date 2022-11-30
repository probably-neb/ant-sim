use ant_sim::*;
#[cfg(feature = "debug")]
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
    let mut app = App::new();
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Ant sim".to_string(),
                width: WINDOW_SIZE_X,
                height: WINDOW_SIZE_Y,
                ..default()
            },
            ..default()
        }));

        #[cfg(feature = "debug")]
        app.add_plugin(WorldInspectorPlugin::new());
            // .register_inspectable::<PheromoneManager>();

        app.add_startup_system(setup_camera)
        .add_loopless_state(GameMode::AntNetwork)
        .add_loopless_state(GameState::Play)
        // .init_resource::<Colors>()
        // .init_resource::<BoundingBox>()
        // .init_resource::<NumAnts>()
        .add_plugin(network::AntNetworkPlugin)
        // .add_startup_system(pheromones::create_pheromone_manager)
        // .add_startup_system(nest::spawn_nests)
        // .add_startup_system(ant::load_ant_texture)
        .add_system(toggle_playing)
        // .add_system_set(
        //     SystemSet::on_update(GameState::Play)
        //         .with_system(nest::food_request_system)
        //         .with_system(ant::move_ant_network.label("move ants"))
        //         .with_system(pheromones::color_and_fade_pheromones.label("color pheromones").after("move ants"))
        //         .with_system(nest::ant_nest_network_interactions)
        //         .with_system(nest::fade_nest_network_pheremones)
        //     )
        // .add_system_set(
        //     SystemSet::on_update(GameState::Paused)
        //         .with_system(ant::animate_ant)
        //                )
        // .add_system(ant::ant_wander)
        // .add_system(pheromones::print_angle)
        // .add_system(print_camera)
        .run();
}

fn print_camera(query: Query<&Camera>, windows: Res<Windows>) {
    // let camera = query.get_single().unwrap();
    // println!("Physical: {:?}", camera.physical_viewport_rect());
    // println!("Logical: {:?}", camera.logical_viewport_rect());
    // let window = windows.primary();
    // println!("window size: ({:?}, {:?})", window.width(), window.height())
}

fn setup_camera(mut commands: Commands) {
    // Camera
    let camera = Camera2dBundle::default();
    commands.spawn(camera);
}
