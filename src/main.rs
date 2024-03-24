#![windows_subsystem = "windows"]

/// timelessnesses' implementation of Conway's Game Of Life in SDL2.
use std::collections::HashMap;

mod ffmpeg;

/// [`LifeState`] is an enum indicating if [`Life`] is alive or dead

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
enum LifeState {
    /// Alive
    Alive,
    /// Died
    Dead,
}

impl LifeState {
    /// Random life generator for the [`LifeState`]
    fn random_life_state() -> Self {
        return *random_choice::random_choice().random_choice_f32(
            &vec![LifeState::Alive, LifeState::Dead],
            &vec![5 as f32, 15 as f32],
            1,
        )[0];
    }
}

/// Struct representing each cube on screen (we call them [`Life`])
#[derive(Clone, Copy)]
struct Life {
    /// X positon of the cube
    x: i32,
    /// Y position of the cube
    y: i32,
    /// Life state of the cube
    state: LifeState,
}

/// A mapping for neighbor positions. Related to [`Game::get_neighbors`] (I'm sorry I am too stupid to create a function for this.)
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

/// Main condition and logics happens here
struct Game {
    cubes: HashMap<(i32, i32), Life>,
}

impl Game {
    /// Apply each [`Life`] with new state base on conditions
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

    /// Get neighbors around the [`Life`]
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

/// Game width (Used on [`ffmpeg::VideoRecorder`])
const WIDTH: u32 = 800;
/// Game height (Used on [`ffmpeg::VideoRecorder`])
const HEIGHT: u32 = 600;
/// Cube size (it will try to fit as much as possible without overfilling)
const CUBE_DIMENSION: u32 = 10;

// Showing width for showing stuff like FPS text
const SHOWING_WIDTH: u32 = WIDTH + 150;
/// Showing height for showing stuff like overfills (round corners sucks)
const SHOWING_HEIGHT: u32 = HEIGHT + 10;

/// Font
const ROBOTO: &[u8; 167000] = include_bytes!("assets/Roboto-Light.ttf");

fn main() {
    // Initialize SDL2
    let ctx = sdl2::init().unwrap();
    let video = ctx.video().unwrap();

    let window = video
        .window("Game Of Life", SHOWING_WIDTH, SHOWING_HEIGHT)
        .position_centered()
        .resizable()
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

    // Initialize the cubes
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

    // [`Game`] instance
    let mut game = Game { cubes };
    let tc = canvas.texture_creator();
    canvas.set_draw_color(sdl2::pixels::Color::BLACK);

    // Pre-rendering dead surface color so I can save time (Optimization gaming)
    let mut dead_surface = sdl2::surface::Surface::new(
        CUBE_DIMENSION,
        CUBE_DIMENSION,
        sdl2::pixels::PixelFormatEnum::RGB24,
    )
    .unwrap();
    // Same reason for [`dead_surface`]
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

    let font_ctx = sdl2::ttf::init().unwrap();

    let fps_font = font_ctx
        .load_font_from_rwops(sdl2::rwops::RWops::from_bytes(ROBOTO).unwrap(), 15)
        .unwrap();

    // fps stuff
    let mut ft = std::time::Instant::now(); // frame time
    let mut fc = 0; // frame count
    let mut fps = 0.0; // frame per sec
    let mut mf = 0.0; // maximum fps
    let mut lf = 0.0; // minimum fps (shows on screen)
    let mut lpf = 0.0; // act as a cache
    let mut lft = std::time::Instant::now(); // minimum frame refresh time thingy

    // Video initialization (`GOL_RECORD`)
    let mut vr: Option<ffmpeg::VideoRecorder> = None;

    if let Ok(_) = std::env::var("GOL_RECORD") {
        vr = Some(ffmpeg::VideoRecorder::new(
            "out.mp4",
            WIDTH,
            HEIGHT,
            video.desktop_display_mode(0).unwrap().refresh_rate as u32,
        ));
    }

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
        canvas.present();
        canvas.clear();
        // draw [`Life`]
        for life in game.cubes.values() {
            let color = match life.state {
                LifeState::Alive => &alive_texture,
                LifeState::Dead => &dead_texture,
            };
            let rect =
                sdl2::rect::Rect::new(life.x as i32, life.y as i32, CUBE_DIMENSION, CUBE_DIMENSION);
            canvas.copy(color, None, rect).unwrap();
        }
        // draw grid
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
        // FPS stuff (ignore them)
        fc += 1;
        let elapsed_time = ft.elapsed();
        if elapsed_time.as_secs() >= 1 {
            fps = fc as f64 / elapsed_time.as_secs_f64();
            fc = 0;
            ft = std::time::Instant::now();
            if fps > mf {
                mf = fps
            } else if fps < lpf {
                lpf = fps
            }
        }
        let elapsed_time = lft.elapsed();
        if elapsed_time.as_secs() >= 3 {
            lf = lpf;
            lpf = fps;
            lft = std::time::Instant::now();
        }
        let elasped = update_time.elapsed();
        if elasped.as_millis() >= 250 {
            update_time = std::time::Instant::now();
            game.apply_rules_to_each_lifes();
        }
        let fps_text = fps_font
            .render(&format!("FPS: {}", truncate(fps, 2)))
            .shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK)
            .unwrap();
        let mf_text = fps_font
            .render(&format!("Maximum FPS: {}", truncate(mf, 2)))
            .shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK)
            .unwrap();
        let lfp_text = fps_font
            .render(&format!("Minimum FPS: {}", truncate(lf, 2)))
            .shaded(sdl2::pixels::Color::WHITE, sdl2::pixels::Color::BLACK)
            .unwrap();
        canvas
            .copy(
                &tc.create_texture_from_surface(&fps_text).unwrap(),
                None,
                sdl2::rect::Rect::new(
                    (SHOWING_WIDTH - fps_text.width()) as i32,
                    0,
                    fps_text.width(),
                    fps_text.height(),
                ),
            )
            .unwrap();
        canvas
            .copy(
                &tc.create_texture_from_surface(&mf_text).unwrap(),
                None,
                sdl2::rect::Rect::new(
                    (SHOWING_WIDTH - mf_text.width()) as i32,
                    40,
                    mf_text.width(),
                    mf_text.height(),
                ),
            )
            .unwrap();
        canvas
            .copy(
                &tc.create_texture_from_surface(&lfp_text).unwrap(),
                None,
                sdl2::rect::Rect::new(
                    (SHOWING_WIDTH - lfp_text.width()) as i32,
                    80,
                    lfp_text.width(),
                    lfp_text.height(),
                ),
            )
            .unwrap();
        match vr.as_mut() {
            Some(v) => {
                v.process_frame(
                    canvas
                        .read_pixels(
                            sdl2::rect::Rect::new(0, 0, WIDTH, HEIGHT),
                            sdl2::pixels::PixelFormatEnum::RGB24,
                        )
                        .unwrap(),
                );
            }
            None => {}
        }
    }
    // Done feeding frames. Now showing result
    match vr {
        Some(v) => {
            v.done();
        }
        None => {}
    }
}

/// Truncate float with [`precision`] as how many digits you needed in final result
fn truncate(b: f64, precision: usize) -> f64 {
    f64::trunc(b * ((10 * precision) as f64)) / ((10 * precision) as f64)
}
