use crate::{ANT_ANIMATION_SPEED, ANT_SCALE, ANT_SPEED};
use bevy::{ecs::component::Component, prelude::*, render::color::Color};

#[derive(Debug, Component)]
pub struct Ant {
    pub target_color: Color,
    pub parent_color: Color,
    pub carrying_food: bool,
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
        target: Color,
        parent: Color,
        ant_texture: &Handle<TextureAtlas>,
    ) -> Self {
        return Self {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: ant_texture.clone(),
                transform: transform.with_scale(ANT_SCALE),
                ..default()
            },
            animation_timer: AntAnimationTimer(Timer::from_seconds(
                ANT_ANIMATION_SPEED,
                TimerMode::Repeating,
            )),
            ant: Ant {
                target_color: target,
                parent_color: parent,
                carrying_food: false,
            },
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

pub fn move_ant(mut query: Query<(&mut Transform, &Ant)>) {
    for (mut transform, _ant) in &mut query {
        transform.translation.y += ANT_SPEED;
        // transform.translation.y += ANT_SPEED;
    }
}

pub fn load_ant_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("ant.png");
    let mut texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(202.0, 250.0), 8, 7, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(AntTexture(texture_atlas_handle));
}

// impl Ant {
//     // if
//     fn new() -> Self {
//     }
// }
