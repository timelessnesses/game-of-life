use std::collections::HashMap;

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
    pub cubes: HashMap<(i32, i32), Life>,
    pub cube_size: u32,
}

impl Game {
    /// Apply each [`Life`] with new state base on conditions
    pub fn apply_rules_to_each_lifes(&mut self) {
        let mut apply_new_states = HashMap::new();
        for (pos, life) in &self.cubes {
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
        }

        for (pos, new_state) in apply_new_states {
            if let Some(life) = self.cubes.get_mut(&pos) {
                life.state = new_state;
            }
        }
    }

    /// Get neighbors around the [`Life`]
    pub fn get_neighbors(&self, life: &Life) -> Vec<Life> {
        let mut neighbors = Vec::new();
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
        for (dx, dy) in n.iter() {
            let nx = life.x + dx;
            let ny = life.y + dy;
            if let Some(n) = self.cubes.get(&(nx, ny)) {
                neighbors.push(*n);
            }
        }
        neighbors
    }
}