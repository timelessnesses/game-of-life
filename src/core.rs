use std::collections::HashMap;
use std::sync::Mutex;

use dashmap::DashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// [`LifeState`] is an enum indicating if [`Life`] is alive or dead
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum LifeState {
    /// Alive
    Alive,
    /// Died
    Dead,
}

impl LifeState {
    /// Random life generator for the [`LifeState`]
    pub fn random_life_state() -> Self {
        return *random_choice::random_choice().random_choice_f32(
            &[LifeState::Alive, LifeState::Dead],
            &[1_f32, 1_f32],
            1,
        )[0];
    }
}

/// Struct representing each cube on screen (we call them [`Life`])
#[derive(Clone, Copy)]
pub struct Life {
    /// X positon of the cube
    pub x: i32,
    /// Y position of the cube
    pub y: i32,
    /// Life state of the cube
    pub state: LifeState,
}

/// Main condition and logics happens here
pub struct Game {
    pub cubes: DashMap<(i32, i32), Life>,
    pub cube_size: u32,
}

impl Game {
    /// Apply each [`Life`] with new state base on conditions
    pub fn apply_rules_to_each_lifes(&mut self) {
        let apply_new_states = DashMap::new();
        self.cubes.par_iter().for_each(|a| {
            let (pos, life) = a.pair();
            let neighbors = self.get_neighbors(life);
            let alive_neighbors = neighbors
                .iter()
                .filter(|n| n.state == LifeState::Alive)
                .count();
            let new_state = match life.state {
                LifeState::Alive => match alive_neighbors {
                    2 | 3 => LifeState::Alive,
                    _ => LifeState::Dead,
                },
                LifeState::Dead => match alive_neighbors {
                    3 => LifeState::Alive,
                    _ => LifeState::Dead,
                },
            };
            apply_new_states.insert(*pos, new_state);
        });

        apply_new_states.par_iter().for_each(|a|{
            let pos = a.key();
            let new_state = a.value();
            if let Some(mut life) = self.cubes.get_mut(&pos) {
                life.state = *new_state;
            }
        });
    }

    /// Get neighbors around the [`Life`]
    pub fn get_neighbors(&self, life: &Life) -> Vec<Life> {
        let neighbors = Mutex::new(Vec::new());
        let n: [(i32, i32); 8] = [
            (-(self.cube_size as i32), -(self.cube_size as i32)),
            (-(self.cube_size as i32), 0),
            (-(self.cube_size as i32), (self.cube_size as i32)),
            (0, -(self.cube_size as i32)),
            (0, (self.cube_size as i32)),
            ((self.cube_size as i32), -(self.cube_size as i32)),
            ((self.cube_size as i32), 0),
            ((self.cube_size as i32), (self.cube_size as i32)),
        ];
        n.par_iter().for_each(|(dx,dy)|{
            let nx = life.x + dx;
            let ny = life.y + dy;
            if let Some(n) = self.cubes.get(&(nx, ny)) {
                neighbors.lock().unwrap().push(*n);
            }
        });
        neighbors.into_inner().unwrap()
    }
}