#![windows_subsystem = "windows"]
use random_choice;
use sdl2;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
enum LifeState {
    Alive,
    Dead,
}

impl LifeState {
    fn random_life_state() -> Self {
        return *random_choice::random_choice().random_choice_f32(
            &vec![LifeState::Alive, LifeState::Dead],
            &vec![5 as f32, 15 as f32],
            1,
        )[0];
    }
}

#[derive(Clone, Copy)]
struct Life {
    x: i32,
    y: i32,
    state: LifeState,
}

const NEIGHBOR_POSITIONS: [(i32, i32); 8] = [
    (-(CUBE_DIMENSION as i32), -(CUBE_DIMENSION as i32)),
    (-(CUBE_DIMENSION as i32), 0),
    (-(CUBE_DIMENSION as i32), (CUBE_DIMENSION as i32)),
    (0, -(CUBE_DIMENSION as i32)),
    (0, (CUBE_DIMENSION as i32)),
    ((CUBE_DIMENSION as i32), -(CUBE_DIMENSION as i32)),
    ((CUBE_DIMENSION as i32), 0),
    ((CUBE_DIMENSION as i32), (CUBE_DIMENSION as i32)),
];

struct Game {
    cubes: HashMap<(i32, i32), Life>,
}

impl Game {
    fn apply_rules_to_each_lifes(&mut self) {
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

    fn get_neighbors(&self, life: &Life) -> Vec<Life> {
        let mut neighbors = Vec::new();
        for (dx, dy) in NEIGHBOR_POSITIONS.iter() {
            let nx = life.x + dx;
            let ny = life.y + dy;
            if let Some(n) = self.cubes.get(&(nx, ny)) {
                neighbors.push(*n);
            }
        }
        return neighbors;
    }
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const CUBE_DIMENSION: u32 = 10;

const SHOWING_WIDTH: u32 = WIDTH + 100;
const SHOWING_HEIGHT: u32 = HEIGHT + 100;

fn main() {
    let ctx = sdl2::init().unwrap();
    let video = ctx.video().unwrap();

    let window = video
        .window("Game Of Life", SHOWING_WIDTH, SHOWING_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();

    let mut event = ctx.event_pump().unwrap();

    let mut cubes: HashMap<(i32, i32), Life> = HashMap::new();

    for y in 0..(HEIGHT / CUBE_DIMENSION) as i32 {
        for x in 0..(WIDTH / CUBE_DIMENSION) as i32 {
            cubes.insert(
                (x * CUBE_DIMENSION as i32, y * CUBE_DIMENSION as i32),
                Life {
                    x: x * CUBE_DIMENSION as i32,
                    y: y * CUBE_DIMENSION as i32,
                    state: LifeState::random_life_state(),
                },
            );
        }
    }

    let mut game = Game { cubes };
    let tc = canvas.texture_creator();
    canvas.set_draw_color(sdl2::pixels::Color::BLACK);

    let mut dead_surface = sdl2::surface::Surface::new(
        CUBE_DIMENSION,
        CUBE_DIMENSION,
        sdl2::pixels::PixelFormatEnum::RGB24,
    )
    .unwrap();
    let mut alive_surface = sdl2::surface::Surface::new(
        CUBE_DIMENSION,
        CUBE_DIMENSION,
        sdl2::pixels::PixelFormatEnum::RGB24,
    )
    .unwrap();

    dead_surface
        .fill_rect(None, sdl2::pixels::Color::GREY)
        .unwrap();
    alive_surface
        .fill_rect(None, sdl2::pixels::Color::WHITE)
        .unwrap();

    let dead_texture = tc.create_texture_from_surface(dead_surface).unwrap();
    let alive_texture = tc.create_texture_from_surface(alive_surface).unwrap();

    let mut update_time = std::time::Instant::now();

    'main_loop: loop {
        for e in event.poll_iter() {
            match e {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'main_loop,
                _ => {}
            }
        }
        canvas.clear();
        // draw cubes
        for life in game.cubes.values() {
            let color = match life.state {
                LifeState::Alive => &alive_texture,
                LifeState::Dead => &dead_texture,
            };
            let rect =
                sdl2::rect::Rect::new(life.x as i32, life.y as i32, CUBE_DIMENSION, CUBE_DIMENSION);
            canvas.copy(color, None, rect).unwrap();
        }
        for y in (0..HEIGHT).step_by(CUBE_DIMENSION as usize) {
            canvas
                .draw_line(
                    sdl2::rect::Point::new(0, y as i32),
                    sdl2::rect::Point::new(WIDTH as i32, y as i32),
                )
                .unwrap();
        }
        for x in (0..WIDTH).step_by(CUBE_DIMENSION as usize) {
            canvas
                .draw_line(
                    sdl2::rect::Point::new(x as i32, 0),
                    sdl2::rect::Point::new(x as i32, HEIGHT as i32),
                )
                .unwrap();
        }
        canvas.present();
        let elasped = update_time.elapsed();
        if elasped.as_millis() >= 500 {
            update_time = std::time::Instant::now();
            game.apply_rules_to_each_lifes();
        }
    }
}
