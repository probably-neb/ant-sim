use ant_sim::*;
use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            title: "Ant sim".to_string(),
            width: WINDOW_SIZE_X,
            height: WINDOW_SIZE_Y,
            ..default()
        },
        ..default()
    }));

    app.add_startup_system(setup_camera)
        .add_loopless_state(GameMode::AntNetwork)
        .add_loopless_state(GameState::Play)
        .init_resource::<HexagonMesh>()
        .add_plugin(network::AntNetworkPlugin)
        .add_system(toggle_playing)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
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

fn setup_camera(mut commands: Commands) {
    // Camera
    let camera = Camera2dBundle::default();
    commands.spawn(camera);
}
