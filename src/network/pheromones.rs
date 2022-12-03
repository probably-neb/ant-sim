use std::ops::{Index, IndexMut};

use crate::{Colors, HexagonMesh, BOARD_HEIGHT, NUM_NESTS};

#[allow(unused_imports)]
use bevy::log;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use super::{ant::Ant, PheromoneParams};

const PHEROMONE_GRANULARITY: u32 = 8;
const PHEROMONE_GRANULARITY_F: f32 = PHEROMONE_GRANULARITY as f32;
const PHEROMONE_SCALE: f32 = 4.0;
// const PHEROMONE_FADE_PERCENTAGE: f32 = 1.0 - PHEROMONE_FADE_RATE;

#[derive(Debug, Component)]
pub struct Pheromone {
    // TODO: NUM_COLORS not num nests
    pub weights: Vec<f32>,
    loc: UVec2,
}

impl Pheromone {
    pub fn new(_num_colors: usize, loc: UVec2) -> Self {
        let weights = vec![0.; NUM_NESTS];
        Self { weights, loc }
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

// fn contained(target: f32, min: f32, max: f32) -> bool {
//     target <= max && target >= min
// }

#[derive(Debug, Clone, Deref, DerefMut, Reflect)]
pub struct PheromoneGrid(Vec<Option<Entity>>);

// impl Inspectable for PheromoneGrid {
//     type Attributes = ();
//     fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
//         let total_len = self.len();
//         let num_filled = self.iter().filter(|c| c.is_some()).count();
//         ui.label(format!("Cells Filled: {}/{}", num_filled, total_len));
//         false
//     }
// }

#[derive(Debug, Component, Reflect)]
pub struct PheromoneManager {
    //TODO: add field for window dims instead of passing them around constantly
    grid_dims: UVec2,
    child_ids: PheromoneGrid,
    pub win: UVec2,

    // #[inspectable(ignore)]
    pub color_queue: Vec<(usize, Entity)>,
}

impl PheromoneManager {
    fn get_grid_dims_from_window_size(win: UVec2) -> UVec2 {
        win / PHEROMONE_GRANULARITY
    }

    pub fn new(width: f32, height: f32) -> Self {
        let width = width as u32;
        let height = height as u32;
        let win = UVec2 {
            x: width,
            y: height,
        };
        let grid_dims = Self::get_grid_dims_from_window_size(win);
        let grid_size = (grid_dims.x * grid_dims.y) as usize;
        let child_ids = PheromoneGrid(vec![None; grid_size]);
        // for i in 0..grid_size {
        //     // temp value
        //     let e = Entity::from_raw(i as u32);
        //     child_ids.push(e);
        // }
        Self {
            grid_dims,
            child_ids,
            win,
            color_queue: Vec::new(),
        }
    }

    /// returns the pheromone grid coordinates of the cell containing loc
    pub fn get_grid_loc(loc: Vec2, window_width: f32, window_height: f32) -> UVec2 {
        let rel_x = loc.x + 0.5 * window_width;
        let rel_y = loc.y + 0.5 * window_height;
        let x = (rel_x / PHEROMONE_GRANULARITY_F) as u32; //.floor();
        let y = (rel_y / PHEROMONE_GRANULARITY_F) as u32; //.floor();

        UVec2 { x, y }
    }

    pub fn cell_containing(&self, loc: Vec2) -> UVec2 {
        let relative_loc = (loc + self.win.as_vec2() / 2.0).as_uvec2();
        //.clamp(UVec2::ZERO, self.grid_dims);
        relative_loc / PHEROMONE_GRANULARITY
    }

    pub fn id_of_pheromone_at(&self, ant_loc: Vec2) -> Option<Entity> {
        let grid_loc = self.cell_containing(ant_loc);

        // println!(
        //     "ant: {:?}\ngrid: {:?}\nw:w/h {:?}\nid: {:?}\n",
        //     loc,
        //     grid_loc,
        //     (window_width, window_height),
        //     id
        // );
        self[grid_loc]
    }

    fn index_grid(&self, index: UVec2) -> u32 {
        // A[i][j] = W*j + i
        self.grid_dims.x * index.y + index.x
    }

    // pub fn ids_of_adjacent_pheromones(&self, angle: f32, ant_loc: Vec2) -> Vec<(Option<Entity>, Vec2)> {
    //     // let ul = Vec2 { x: -1.0, y: 1.0 };
    //     // let uu = Vec2 { x: 0.0, y: 1.0 };
    //     // let ur = Vec2 { x: 1.0, y: 1.0 };
    //     // let rr = Vec2 { x: 1.0, y: 0.0 };
    //     // let dr = Vec2 { x: 1.0, y: -1.0 };
    //     // let dd = Vec2 { x: 0.0, y: -1.0 };
    //     // let dl = Vec2 { x: -1.0, y: -1.0 };
    //     // let ll = Vec2 { x: -1.0, y: 0.0 };
    //     let current_tile = Self::get_grid_loc(ant_loc, self.win.x, self.win.y);
    //     // let mut locs: [Option<Vec2>; 3] = [Some(current_tile); 3];

    //     // let NE = FRAC_PI_4;
    //     // let NW = 3. * FRAC_PI_4;
    //     // let SW = 5. * FRAC_PI_4;
    //     // let SE = 7. * FRAC_PI_4;

    //     let ant_loc: IVec2 = ant_loc.as_ivec2();
    //     // for checking iu locs are in bounds
    //     fn within_bounds(target: isize, min: u32, max: u32) -> bool {
    //         let min = min as isize;
    //         let max = max as isize;
    //         return target < max && target >= min;
    //     }
    //     let mut angle = angle;

    //     if angle < 0.0 {
    //         angle = TAU - angle;
    //     }

    //     let mut locs = Vec::with_capacity(16);
    //     let (ax, ay) = (ant_loc.x, ant_loc.y);
    //     let mut min_x = ax;
    //     let mut max_x = ax;
    //     let mut min_y = ay;
    //     let mut max_y = ay;

    //     let range = 4;
    //     let hrange = 2;
    //     let dir: u8 = (angle / FRAC_PI_8).floor() as u8;
    //     if dir <= 2 || dir <= 14 {
    //         // E
    //         min_x += hrange;
    //         max_x += range;
    //         min_y -= range;
    //         max_y += range;
    //     } else if dir <= 6 {
    //         // N
    //         max_x += range;
    //         min_x -= range;
    //         min_y += hrange;
    //         max_y += range;
    //     } else if dir <= 10 {
    //         // W
    //         min_x -= hrange;
    //         max_x -= range;
    //         max_y += range;
    //         min_y -= range;
    //     } else if dir <= 14 {
    //         // S
    //         max_x += range;
    //         min_x -= range;
    //         min_y -= hrange;
    //         max_y -= range;
    //     }

    //     for x in min_x..=max_x {
    //         for y in min_y..=max_y {
    //             if within_bounds(x, 0, self.grid_dims.x)
    //                 && within_bounds(y, 0, self.grid_dims.y)
    //                 && (x != ant_loc.x && y != ant_loc.y)
    //             {
    //                 let loc = Vec2 { x, y };
    //                 locs.push(loc)
    //             } else {
    //                 continue;
    //             };
    //         }
    //     }

    //     let mut ids: Vec<(Option<Entity>, Vec2)> = Vec::with_capacity(3);
    //     for tile_loc in locs.iter() {
    //         let world_loc = ant_loc + (PHEROMONE_GRANULARITY * (current_tile - *tile_loc));
    //         ids.push((
    //             self.id_of_pheromone_at(*tile_loc, self.win.x, self.win.y),
    //             world_loc,
    //         ));
    //     }
    //     return ids;
    // }
}

impl Index<UVec2> for PheromoneManager {
    type Output = Option<Entity>;
    fn index(&self, index: UVec2) -> &Self::Output {
        let idx = self.index_grid(index);
        // println!(
        //     "idx: {:?} | x,y: {:?} / dims: {:?}",
        //     idx, index, self.grid_dims
        // );

        &self.child_ids[idx as usize] as _
    }
}

impl IndexMut<UVec2> for PheromoneManager {
    fn index_mut(&mut self, index: UVec2) -> &mut Self::Output {
        let idx = self.index_grid(index);

        &mut self.child_ids[idx as usize] as _
    }
}

pub fn create_pheromone_manager(
    mut commands: Commands,
    windows: Res<Windows>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.primary();
    let (height, width) = (window.height(), window.width());
    let manager = PheromoneManager::new(width, height);
    let mut entity_commands = commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(-(width / 2.0), -(height / 2.0), BOARD_HEIGHT as f32),
            ..default()
        },
        Name::new("PheromoneManager"),
    ));
    // let default_handle = materials.add(ColorMaterial::default());
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
    mut pheromones: Query<(Entity, &mut Pheromone, &mut Visibility)>,
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
            // log::info!("despawning pheromone {:?} at {:?}", id, pheromone_loc);
            commands.entity(id).despawn();
            pheromone_manager[pheromone_loc] = None;
        }
        // log::info!("pheromone visible: {}", visibility.is_visible);
    }
}

pub fn create_required_pheromones(
    mut commands: Commands,
    ants: Query<(&Ant, &Transform)>,
    mut pheromone_manager: Query<(Entity, &mut PheromoneManager)>,
    colors: Res<Colors>,
    pher_params: Res<PheromoneParams>,
    hex_mesh: Res<HexagonMesh>,
) {
    let (manager_id, mut pheromone_manager) = pheromone_manager
        .get_single_mut()
        .expect("there should be pheromones");
    // let bounds = pheromone_manager.win;
    for (ant, transform) in &ants {
        let ant_loc = transform.translation.truncate();

        let pheromone_loc = pheromone_manager.cell_containing(ant_loc);
        let opt_pheromone_tile = pheromone_manager[pheromone_loc];

        let trail_color = ant.parent_color;

        match opt_pheromone_tile {
            Some(pheromone_id) => {
                pheromone_manager
                    .color_queue
                    .push((trail_color, pheromone_id));
            }
            None => {
                let mut pheromone = Pheromone::new(colors.colors.len(), pheromone_loc);

                pheromone.add_trail(trail_color, pher_params.trail_step);

                let scaled_loc = (pheromone_loc * PHEROMONE_GRANULARITY).as_vec2();
                commands.entity(manager_id).with_children(|builder| {
                    let pheromone_id = builder
                        .spawn((
                            MaterialMesh2dBundle {
                                // mesh: meshes.add(shape::Circle::default().into()).into(),
                                mesh: hex_mesh.clone_weak().into(),
                                //FIXME: no color here
                                material: colors.color_handles[trail_color].clone_weak(),
                                transform: Transform::from_xyz(
                                    scaled_loc.x,
                                    scaled_loc.y,
                                    BOARD_HEIGHT as f32,
                                )
                                .with_scale(Vec3::splat(PHEROMONE_SCALE)),
                                ..default()
                            },
                            pheromone,
                        ))
                        .id();
                    pheromone_manager[pheromone_loc] = Some(pheromone_id);
                    // log::info!("created pheromone {:?} at {:?}", pheromone_id, pheromone_loc);
                    pheromone_manager
                        .color_queue
                        .push((trail_color, pheromone_id));
                });
            }
        }
    }
}

pub fn leave_pheromone_trails(
    mut pheromone_manager: Query<&mut PheromoneManager>,
    mut pheromones: Query<(Entity, &mut Pheromone, &mut Handle<ColorMaterial>)>,
    colors: Res<Colors>,
    pher_params: Res<PheromoneParams>,
) {
    let mut pheromone_manager = pheromone_manager
        .get_single_mut()
        .expect("there should be pheromones");
    // let bounds = pheromone_manager.win;
    for (trail_color, pheromone_id) in pheromone_manager.color_queue.drain(0..) {
        let (_, mut pheromone, mut color_handle) = pheromones
            .get_mut(pheromone_id)
            .expect("pheromone manager grid should only contain exisiting entities");

        // to find our way home
        pheromone.add_trail(trail_color, pher_params.trail_step);

        // color trail
        let color_id = pheromone.most_prominent();
        let cur_color_handle: &Handle<ColorMaterial> = &colors.color_handles[color_id];
        if cur_color_handle.id() != color_handle.id() {
            *color_handle = cur_color_handle.clone_weak();
        }
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn get_grid_loc_0_0_should_be_win_dims_div_8() {
//         let (x, y) = (8., 8.);
//         let loc = PheromoneManager::get_grid_loc(Vec2::splat(0.), x, y);
//         assert_eq!(
//             loc,
//             Vec2 {
//                 x: x / PHEROMONE_GRANULARITY,
//                 y: y / PHEROMONE_GRANULARITY
//             }
//         );
//         let (x, y) = (8., 16.);
//         let loc = PheromoneManager::get_grid_loc(Vec2::splat(0.), x, y);
//         assert_eq!(
//             loc,
//             Vec2 {
//                 x: x / PHEROMONE_GRANULARITY,
//                 y: y / PHEROMONE_GRANULARITY
//             }
//         );
//         let (x, y) = (16., 16.);
//         let loc = PheromoneManager::get_grid_loc(Vec2::splat(0.), x, y);
//         assert_eq!(
//             loc,
//             Vec2 {
//                 x: x / PHEROMONE_GRANULARITY,
//                 y: y / PHEROMONE_GRANULARITY
//             }
//         );
//         let (x, y) = (2. * PHEROMONE_GRANULARITY, 2. * PHEROMONE_GRANULARITY);
//         let loc = PheromoneManager::get_grid_loc(Vec2::splat(-PHEROMONE_GRANULARITY), x, y);
//         assert_eq!(loc, Vec2 { x: 1., y: 1. });
//     }
// }
