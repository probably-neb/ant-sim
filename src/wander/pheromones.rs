use std::{
    f32::consts::{FRAC_PI_8, TAU},
    ops::{Index, IndexMut},
};

use crate::{Colors, BOARD_HEIGHT};

#[allow(unused_imports)]
use bevy::log;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

const PHEROMONE_STEP: f32 = 0.10;
const PHEROMONE_GRANULARITY: f32 = 4.0;
const PHEROMONE_FADE_RATE: f32 = 0.001;
// const PHEROMONE_FADE_PERCENTAGE: f32 = 1.0 - PHEROMONE_FADE_RATE;

#[derive(Debug, Component)]
pub struct Pheromone {
    // TODO: NUM_COLORS not num nests
    pub weights: Vec<f32>,
}

#[derive(Component, Default)]
pub struct NonEmptyTrail;

impl Pheromone {
    pub fn new(num_colors: usize) -> Self {
        let weights = vec![0.; num_colors];
        Self { weights }
    }
    pub fn add_trail(&mut self, color: usize) {
        self.weights[color] += PHEROMONE_STEP;
    }

    pub fn is_empty(&self) -> bool {
        return self.weights.iter().sum::<f32>() <= f32::EPSILON;
    }

    pub fn most_prominent(&self) -> usize {
        return self
            .weights
            .iter()
            .enumerate()
            .max_by_key(|tup| (*tup.1 * 100.0) as usize)
            .expect("pheromone weights shouldn't be empty")
            .0;
    }
    pub fn fade(&mut self) {
        for w in &mut self.weights {
            *w = (*w - PHEROMONE_FADE_RATE).max(0.0);
        }
    }
}

#[derive(Debug, Component)]
pub struct PheromoneManager {
    //TODO: add field for window dims instead of passing them around constantly
    grid_dims: Vec2,
    child_ids: Vec<Entity>,
    pub win: Vec2,
}

impl PheromoneManager {
    fn get_grid_dims_from_window_size(width: f32, height: f32) -> Vec2 {
        let width = width / PHEROMONE_GRANULARITY;
        let height = height / PHEROMONE_GRANULARITY;

        Vec2 {
            x: width,
            y: height,
        }
    }
    pub fn new(w_width: f32, w_height: f32) -> Self {
        let grid_dims = Self::get_grid_dims_from_window_size(w_width, w_height);
        let grid_size = (grid_dims.x * grid_dims.y) as usize;
        let mut child_ids = Vec::with_capacity(grid_size);
        for i in 0..grid_size {
            // temp value
            let e = Entity::from_raw(i as u32);
            child_ids.push(e);
        }
        let win = Vec2 {
            x: w_width,
            y: w_height,
        };
        Self {
            grid_dims,
            child_ids,
            win,
        }
    }

    /// returns the pheromone grid coordinates of the cell containing loc
    fn get_grid_loc(loc: Vec2, window_width: f32, window_height: f32) -> Vec2 {
        let rel_x = loc.x + 0.5 * window_width;
        let rel_y = loc.y + 0.5 * window_height;
        let x = rel_x / PHEROMONE_GRANULARITY; //.floor();
        let y = rel_y / PHEROMONE_GRANULARITY; //.floor();

        Vec2 { x, y }
    }

    pub fn id_of_pheromone_at(
        &self,
        ant_loc: Vec2,
        window_width: f32,
        window_height: f32,
    ) -> Entity {
        let grid_loc = Self::get_grid_loc(ant_loc, window_width, window_height);

        self[grid_loc]
    }

    fn index_grid(&self, index: Vec2) -> f32 {
        // A[i][j] = W*j + i
        self.grid_dims.x * index.y.floor() + index.x.floor()
    }

    pub fn ids_of_adjacent_pheromones(&self, angle: f32, ant_loc: Vec2) -> Vec<(Entity, Vec2)> {
        // let ul = Vec2 { x: -1.0, y: 1.0 };
        // let uu = Vec2 { x: 0.0, y: 1.0 };
        // let ur = Vec2 { x: 1.0, y: 1.0 };
        // let rr = Vec2 { x: 1.0, y: 0.0 };
        // let dr = Vec2 { x: 1.0, y: -1.0 };
        // let dd = Vec2 { x: 0.0, y: -1.0 };
        // let dl = Vec2 { x: -1.0, y: -1.0 };
        // let ll = Vec2 { x: -1.0, y: 0.0 };
        let current_tile = Self::get_grid_loc(ant_loc, self.win.x, self.win.y);
        // let mut locs: [Option<Vec2>; 3] = [Some(current_tile); 3];

        // let NE = FRAC_PI_4;
        // let NW = 3. * FRAC_PI_4;
        // let SW = 5. * FRAC_PI_4;
        // let SE = 7. * FRAC_PI_4;

        // for checking if locs are in bounds
        fn within_bounds(target: f32, min: f32, max: f32) -> bool {
            target < max && target >= min
        }
        let mut angle = angle;

        if angle < 0.0 {
            angle = TAU - angle;
        }

        let mut locs = Vec::with_capacity(16);
        let (ax, ay) = (ant_loc.x as isize, ant_loc.y as isize);
        let mut min_x = ax;
        let mut max_x = ax;
        let mut min_y = ay;
        let mut max_y = ay;

        let range = 4;
        let hrange = 2;
        let dir: u8 = (angle / FRAC_PI_8).floor() as u8;
        if dir <= 2 || dir <= 14 {
            // E
            min_x += hrange;
            max_x += range;
            min_y -= range;
            max_y += range;
        } else if dir <= 6 {
            // N
            max_x += range;
            min_x -= range;
            min_y += hrange;
            max_y += range;
        } else if dir <= 10 {
            // W
            min_x -= hrange;
            max_x -= range;
            max_y += range;
            min_y -= range;
        } else if dir <= 14 {
            // S
            max_x += range;
            min_x -= range;
            min_y -= hrange;
            max_y -= range;
        }

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let x = x as f32;
                let y = y as f32;
                if within_bounds(x, 0.0, self.grid_dims.x)
                    && within_bounds(y, 0.0, self.grid_dims.y)
                    && (x != ant_loc.x && y != ant_loc.y)
                {
                    let loc = Vec2 { x, y };
                    locs.push(loc)
                } else {
                    continue;
                };
            }
        }

        // // NORTH
        // if contained(angle, NE, NW) {
        //     for (i, v) in [ul, uu, ur].iter().enumerate() {
        //         let grid_loc = locs[i].unwrap() + *v;
        //         if within_bounds(grid_loc.x, 0.0, self.grid_dims.x)
        //             && within_bounds(grid_loc.y, 0.0, self.grid_dims.y)
        //         {
        //             locs[i] = Some(grid_loc);
        //         } else {
        //             locs[i] = None;
        //         }
        //     }
        // }
        // // EAST
        // if contained(angle, 0.0, NE) || contained(angle, SE, TAU) {
        //     for (i, v) in [ur, rr, dr].iter().enumerate() {
        //         let grid_loc = locs[i].unwrap() + *v;
        //         if within_bounds(grid_loc.x, 0.0, self.grid_dims.x)
        //             && within_bounds(grid_loc.y, 0.0, self.grid_dims.y)
        //         {
        //             locs[i] = Some(grid_loc);
        //         } else {
        //             locs[i] = None;
        //         }
        //     }
        // }
        // // SOUTH
        // if contained(angle, SW, SE) {
        //     for (i, v) in [dl, dd, dr].iter().enumerate() {
        //         let grid_loc = locs[i].unwrap() + *v;
        //         if within_bounds(grid_loc.x, 0.0, self.grid_dims.x)
        //             && within_bounds(grid_loc.y, 0.0, self.grid_dims.y)
        //         {
        //             locs[i] = Some(grid_loc);
        //         } else {
        //             locs[i] = None;
        //         }
        //     }
        // }
        // // WEST
        // if contained(angle, NW, SW) {
        //     for (i, v) in [dl, ll, ul].iter().enumerate() {
        //         let grid_loc = locs[i].unwrap() + *v;
        //         if within_bounds(grid_loc.x, 0.0, self.grid_dims.x)
        //             && within_bounds(grid_loc.y, 0.0, self.grid_dims.y)
        //         {
        //             locs[i] = Some(grid_loc);
        //         } else {
        //             locs[i] = None;
        //         }
        //     }
        // }

        let mut ids: Vec<(Entity, Vec2)> = Vec::with_capacity(3);
        for tile_loc in locs.iter() {
            let world_loc = ant_loc + (PHEROMONE_GRANULARITY * (current_tile - *tile_loc));
            ids.push((
                self.id_of_pheromone_at(*tile_loc, self.win.x, self.win.y),
                world_loc,
            ));
        }
        ids
    }
}

impl Index<Vec2> for PheromoneManager {
    type Output = Entity;
    fn index(&self, index: Vec2) -> &Self::Output {
        let idx = self.index_grid(index);
        // println!(
        //     "idx: {:?} | x,y: {:?} / dims: {:?}",
        //     idx, index, self.grid_dims
        // );

        &self.child_ids[idx.floor() as usize] as _
    }
}

impl IndexMut<Vec2> for PheromoneManager {
    fn index_mut(&mut self, index: Vec2) -> &mut Self::Output {
        let idx = self.index_grid(index);

        &mut self.child_ids[idx.floor() as usize] as _
    }
}

pub fn create_pheromone_manager(
    mut commands: Commands,
    windows: Res<Windows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.primary();
    let (height, width) = (window.height(), window.width());
    let mut manager = PheromoneManager::new(width, height);
    let mut entity_commands = commands.spawn((SpatialBundle {
        transform: Transform::from_xyz(-(width / 2.0), -(height / 2.0), BOARD_HEIGHT as f32),
        ..default()
    },));
    let default_handle = materials.add(ColorMaterial::default());
    entity_commands.with_children(|builder| {
        for x in (0..manager.grid_dims.x as u32).rev() {
            for y in (0..manager.grid_dims.y as u32).rev() {
                let x = x as f32;
                let y = y as f32;
                let dim_x = x * PHEROMONE_GRANULARITY;
                let dim_y = y * PHEROMONE_GRANULARITY;
                let id = builder
                    .spawn((
                        MaterialMesh2dBundle {
                            // mesh: meshes.add(shape::Circle::default().into()).into(),
                            mesh: meshes
                                .add(
                                    (shape::RegularPolygon {
                                        sides: 6,
                                        ..default()
                                    })
                                    .into(),
                                )
                                .into(),
                            //FIXME: no color here
                            material: default_handle.clone(),
                            transform: Transform::from_xyz(dim_x, dim_y, BOARD_HEIGHT as f32)
                                .with_scale(Vec3::splat(PHEROMONE_GRANULARITY)),
                            visibility: Visibility { is_visible: false },
                            ..default()
                        },
                        Pheromone::new(5),
                    ))
                    .id();
                manager[Vec2 { x, y }] = id;
            }
        }
    });
    // manager.child_ids.reverse();
    entity_commands.insert(manager);
}

pub fn color_and_fade_pheromones(
    mut commands: Commands,
    mut pheromones: Query<
        (
            Entity,
            &mut Pheromone,
            &mut Visibility,
            &mut Handle<ColorMaterial>,
        ),
        With<NonEmptyTrail>,
    >,
    colors: Res<Colors>,
) {
    for (id, mut pheromone, mut visibility, mut color_handle) in &mut pheromones {
        // color trail
        let color_id = pheromone.most_prominent();
        let cur_color_handle: &Handle<ColorMaterial> = &colors.color_handles[color_id];
        if cur_color_handle.id() != color_handle.id() {
            *color_handle = Handle::weak(cur_color_handle.id());
        }

        pheromone.fade();
        visibility.is_visible = !pheromone.is_empty();
        if !visibility.is_visible {
            // will prevent this pheromone from being looped over until another ant steps on it
            commands.entity(id).remove::<NonEmptyTrail>();
        }
        // log::info!("pheromone visible: {}", visibility.is_visible);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_grid_loc_0_0_should_be_win_dims_div_8() {
        let (x, y) = (8., 8.);
        let loc = PheromoneManager::get_grid_loc(Vec2::splat(0.), x, y);
        assert_eq!(
            loc,
            Vec2 {
                x: x / PHEROMONE_GRANULARITY,
                y: y / PHEROMONE_GRANULARITY
            }
        );
        let (x, y) = (8., 16.);
        let loc = PheromoneManager::get_grid_loc(Vec2::splat(0.), x, y);
        assert_eq!(
            loc,
            Vec2 {
                x: x / PHEROMONE_GRANULARITY,
                y: y / PHEROMONE_GRANULARITY
            }
        );
        let (x, y) = (16., 16.);
        let loc = PheromoneManager::get_grid_loc(Vec2::splat(0.), x, y);
        assert_eq!(
            loc,
            Vec2 {
                x: x / PHEROMONE_GRANULARITY,
                y: y / PHEROMONE_GRANULARITY
            }
        );
        let (x, y) = (2. * PHEROMONE_GRANULARITY, 2. * PHEROMONE_GRANULARITY);
        let loc = PheromoneManager::get_grid_loc(Vec2::splat(-PHEROMONE_GRANULARITY), x, y);
        assert_eq!(loc, Vec2 { x: 1., y: 1. });
    }
}
