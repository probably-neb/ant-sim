use bevy::ecs::component::Component;

#[derive(Debug, Clone, Copy, Component)]
pub struct Food {
    pub color: usize
}

impl Food {
    pub fn new(color: usize) -> Self {
        return Self { color };
    }
}
