use std::{
    f32::consts::{FRAC_PI_8, TAU},
    ops::{Index, IndexMut},
};

use crate::{Colors, BOARD_HEIGHT, NUM_NESTS, HexagonMesh};
use bevy::{log, prelude::*, sprite::MaterialMesh2dBundle};

use super::{ant::Ant, pheromones, PheromoneParams};

const PHEROMONE_GRANULARITY: f32 = 4.0;
const PHEROMONE_SCALE: f32 = 4.0;
// const PHEROMONE_FADE_PERCENTAGE: f32 = 1.0 - PHEROMONE_FADE_RATE;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Component)]
pub struct Pheromone {
    // TODO: NUM_COLORS not num nests
    pub weights: Vec<f32>,
    loc: Vec2,
}

#[derive(Component, Default)]
pub struct NonEmptyTrail;

impl Pheromone {
    pub fn new(_num_colors: usize, loc: Vec2) -> Self {
        let weights = vec![0.; NUM_NESTS];
        return Self { weights, loc };
    }
    pub fn add_trail(&mut self, color: usize, step: f32) {
        self.weights[color] += step;
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
    pub fn fade(&mut self, rate: f32) {
        for w in &mut self.weights {
            *w = (*w - rate).max(0.0);
        }
    }
}

fn contained(target: f32, min: f32, max: f32) -> bool {
    return target <= max && target >= min;
}

// #[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Component)]
pub struct PheromoneManager {
    //TODO: add field for window dims instead of passing them around constantly
    grid_dims: Vec2,
    child_ids: Vec<Option<Entity>>,
    pub win: Vec2,
}

impl PheromoneManager {
    fn get_grid_dims_from_window_size(width: f32, height: f32) -> Vec2 {
        let width = width / PHEROMONE_GRANULARITY;
        let height = height / PHEROMONE_GRANULARITY;
        let grid_dims = Vec2 {
            x: width,
            y: height,
        };
        return grid_dims;
    }
    pub fn new(w_width: f32, w_height: f32) -> Self {
        let grid_dims = Self::get_grid_dims_from_window_size(w_width, w_height);
        let grid_size = (grid_dims.x * grid_dims.y) as usize;
        let mut child_ids = vec![None; grid_size];
        // for i in 0..grid_size {
        //     // temp value
        //     let e = Entity::from_raw(i as u32);
        //     child_ids.push(e);
        // }
        let win = Vec2 {
            x: w_width,
            y: w_height,
        };
        return Self {
            grid_dims,
            child_ids,
            win,
        };
    }

    /// returns the pheromone grid coordinates of the cell containing loc
    pub fn get_grid_loc(loc: Vec2, window_width: f32, window_height: f32) -> Vec2 {
        let rel_x = loc.x + 0.5 * window_width;
        let rel_y = loc.y + 0.5 * window_height;
        let x = rel_x / PHEROMONE_GRANULARITY; //.floor();
        let y = rel_y / PHEROMONE_GRANULARITY; //.floor();
        let grid_loc = Vec2 { x, y };
        return grid_loc;
    }

    pub fn id_of_pheromone_at(
        &self,
        ant_loc: Vec2,
        window_width: f32,
        window_height: f32,
    ) -> Option<Entity> {
        let grid_loc = Self::get_grid_loc(ant_loc, window_width, window_height);
        let id = self[grid_loc];
        // println!(
        //     "ant: {:?}\ngrid: {:?}\nw:w/h {:?}\nid: {:?}\n",
        //     loc,
        //     grid_loc,
        //     (window_width, window_height),
        //     id
        // );
        return id;
    }

    fn index_grid(&self, index: Vec2) -> f32 {
        // A[i][j] = W*j + i
        return self.grid_dims.x * index.y.floor() + index.x.floor();
    }

    pub fn ids_of_adjacent_pheromones(&self, angle: f32, ant_loc: Vec2) -> Vec<(Option<Entity>, Vec2)> {
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
            return target < max && target >= min;
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

        let mut ids: Vec<(Option<Entity>, Vec2)> = Vec::with_capacity(3);
        for tile_loc in locs.iter() {
            let world_loc = ant_loc + (PHEROMONE_GRANULARITY * (current_tile - *tile_loc));
            ids.push((
                self.id_of_pheromone_at(*tile_loc, self.win.x, self.win.y),
                world_loc,
            ));
        }
        return ids;
    }
}

impl Index<Vec2> for PheromoneManager {
    type Output = Option<Entity>;
    fn index(&self, index: Vec2) -> &Self::Output {
        let idx = self.index_grid(index);
        // println!(
        //     "idx: {:?} | x,y: {:?} / dims: {:?}",
        //     idx, index, self.grid_dims
        // );
        let id = &self.child_ids[idx.floor() as usize];
        return id;
    }
}

impl IndexMut<Vec2> for PheromoneManager {
    fn index_mut(&mut self, index: Vec2) -> &mut Self::Output {
        let idx = self.index_grid(index);
        let id = &mut self.child_ids[idx.floor() as usize];
        return id;
    }
}

pub fn create_pheromone_manager(
    mut commands: Commands,
    windows: Res<Windows>,
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
    //entity_commands.with_children(|builder| {
    //    for x in (0..manager.grid_dims.x as u32).rev() {
    //        for y in (0..manager.grid_dims.y as u32).rev() {
    //            let x = x as f32;
    //            let y = y as f32;
    //            let dim_x = x * PHEROMONE_GRANULARITY;
    //            let dim_y = y * PHEROMONE_GRANULARITY;
    //            let id = builder
    //                .spawn((
    //                    MaterialMesh2dBundle {
    //                        // mesh: meshes.add(shape::Circle::default().into()).into(),
    //                        mesh: meshes.add((shape::RegularPolygon{sides: 6, ..default()}).into()).into(),
    //                        //FIXME: no color here
    //                        material: default_handle.clone(),
    //                        transform: Transform::from_xyz(dim_x, dim_y, BOARD_HEIGHT as f32)
    //                            .with_scale(Vec3::splat(PHEROMONE_GRANULARITY)),
    //                        visibility: Visibility { is_visible: false },
    //                        ..default()
    //                    },
    //                    Pheromone::new(5, Vec2 { x, y }),
    //                ))
    //                .id();
    //            manager[Vec2 { x, y }] = id;
    //        }
    //    }
    //});
    // manager.child_ids.reverse();
    entity_commands.insert(manager);
}

pub fn fade_pheromones(
    mut commands: Commands,
    mut pheromone_manager: Query<&mut PheromoneManager>,
    mut pheromones: Query<
        (
            Entity,
            &mut Pheromone,
            &mut Visibility,
        ),
        With<NonEmptyTrail>,
    >,
    params: Res<PheromoneParams>,
    time: Res<Time>,
) {
    let pheromone_manager = &mut pheromone_manager
        .get_single_mut()
        .expect("there should be pheromones");
    let rate = params.trail_fade_rate * (1.0 + time.delta_seconds());
    for (id, mut pheromone, mut visibility) in &mut pheromones {
        pheromone.fade(rate);
        visibility.is_visible = !pheromone.is_empty();
        if !visibility.is_visible {
            // will prevent this pheromone from being looped over until another ant steps on it
            // log::info!("removed pheromone at {:?}", id);
            // commands.entity(id).remove::<NonEmptyTrail>();
            let pheromone_loc = pheromone.loc;
            commands.entity(id).despawn();
            pheromone_manager[pheromone_loc].take(); // = None;
        }
        // log::info!("pheromone visible: {}", visibility.is_visible);
    }
}

pub fn leave_pheromone_trails(
    mut commands: Commands,
    ants: Query<(&Ant, &Transform)>,
    mut pheromone_manager: Query<(Entity, &mut PheromoneManager)>,
    mut pheromones: Query<
        (
            Entity,
            &mut Pheromone,
            &mut Handle<ColorMaterial>,
        ),
    >,
    colors: Res<Colors>,
    pher_params: Res<PheromoneParams>,
    hex_mesh: Res<HexagonMesh>,
) {
    let (manager_id, mut pheromone_manager) = pheromone_manager
        .get_single_mut()
        .expect("there should be pheromones");
    let bounds = pheromone_manager.win;
    for (ant, transform) in &ants {
        let ant_loc = transform.translation.truncate();

        let pheromone_loc = PheromoneManager::get_grid_loc(ant_loc, bounds.x, bounds.y);
        let opt_pheromone_tile = pheromone_manager[pheromone_loc];

        let trail_color = ant.parent_color;

        match opt_pheromone_tile.and_then( |pheromone_tile| 
                                      pheromones
                                      .get_mut(pheromone_tile)
                                      .ok()
                                      ) {
            Some((pheromone_tile, mut pheromone, mut color_handle)) => {

                // to find our way home
                pheromone.add_trail(trail_color, pher_params.trail_step);

                // pheromone_timer.tick(time.delta());
                // if pheromone_timer.just_finished() {
                    commands
                        .entity(pheromone_tile)
                        .insert(pheromones::NonEmptyTrail);
                // }

                // color trail
                let color_id = pheromone.most_prominent();
                let cur_color_handle: &Handle<ColorMaterial> = &colors.color_handles[color_id];
                if cur_color_handle.id() != color_handle.id() {
                    *color_handle = Handle::weak(cur_color_handle.id());
                }
            },
            None => {
                let mut pheromone = Pheromone::new(colors.colors.len(), pheromone_loc);
                let color_handle = &colors.color_handles[trail_color];

                pheromone.add_trail(trail_color,pher_params.trail_step);

                let scaled_loc = pheromone_loc * PHEROMONE_GRANULARITY;
                commands.entity(manager_id).with_children(
                    |builder| {
                        let new_pher_id = builder.spawn((
                            MaterialMesh2dBundle {
                                // mesh: meshes.add(shape::Circle::default().into()).into(),
                                mesh: hex_mesh.clone_weak().into(),
                                //FIXME: no color here
                                material: color_handle.clone_weak(),
                                transform: Transform::from_xyz(scaled_loc.x, scaled_loc.y, BOARD_HEIGHT as f32)
                                    .with_scale(Vec3::splat(PHEROMONE_SCALE)),
                                    ..default()
                            },
                            pheromone,
                            pheromones::NonEmptyTrail,
                            )).id();
                        pheromone_manager[pheromone_loc] = Some(new_pher_id);
                    }

                 );
            },
        }

        // make transparent based on weight
        // ###############################
        // let weight = pheromone.weights[color_id];
        // log::info!("coloring pher with clr: {}", color_id);
        // let color = colors.colors[color_id];
        // let material = materials
        //     .get_mut(&color_handle)
        //     .expect("pheromones should have a color");
        //
        // material.color = *color.clone().set_a(weight);
    }
}

// pub fn print_angle(
//     mut pheromones: Query<(
//         Entity,
//         &Pheromone,
//         &mut Visibility,
//         &mut Handle<ColorMaterial>,
//     )>,
//     manager: Query<&PheromoneManager>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     let manager = manager.get_single().unwrap();
//     let max_loc = manager.grid_dims;
//     let m = max_loc.y / max_loc.x;
//     for x in 0..max_loc.x as usize {
//         let x = x as f32;
//         let y = m * x;
//         let loc = Vec2 { x, y };
//         let id = manager[loc];
//         let mut tup = pheromones.get_mut(id).unwrap();
//         tup.2.is_visible = true;
//         materials.get_mut(&tup.3).unwrap().color = Color::RED;
//     }
//     let mid = pheromones
//         .get_mut(manager.id_of_pheromone_at(Vec2::splat(0.), 736., 955.5))
//         .unwrap();
//     materials.get_mut(&mid.3).unwrap().color = Color::AQUAMARINE;
// }

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
