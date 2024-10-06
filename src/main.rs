// #![windows_subsystem = "windows"]
use clap::Parser;
/// timelessnesses' implementation of Conway's Game Of Life in SDL2.
use ctrlc;
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
            &vec![1 as f32, 1 as f32],
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

/// Main condition and logics happens here
struct Game {
    cubes: HashMap<(i32, i32), Life>,
    cube_size: u32,
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
        return neighbors;
    }
}

#[derive(clap::Parser)]
#[command(author = "timelessnesses", about = "Nothing")]
struct Cli {
    /// Frame limiting
    #[arg(short, long)]
    fps: Option<u64>,
    /// List GPU renderers (for the SELECTED_GPU_RENDERER arg)
    #[arg(short, long)]
    list_gpu_renderers: bool,
    /// Select your own renderer if you want to
    #[arg(short, long)]
    selected_gpu_renderer: Option<u32>,

    /// Force VSync
    #[arg(short, long)]
    vsync: Option<bool>,

    /// Record the game to a video file
    #[arg(short, long)]
    record: Option<bool>,

    /// Length of the video file
    #[arg(short, long)]
    length: Option<String>,

    /// Width of the window (default: 1920)
    #[arg(short, long)]
    width: Option<u32>,

    /// Height of the window (default: 1080)
    #[arg(short, long)]
    height: Option<u32>,

    /// Cube size (default: 10)
    #[arg(short, long)]
    cube_size: Option<u32>,
}

/// Font
const ROBOTO: &[u8; 167000] = include_bytes!("assets/Roboto-Light.ttf");

fn main() {
    let cli = Cli::parse();
    if cli.list_gpu_renderers {
        println!("Available GPU renderers:");
        for (i, r) in sdl2::render::drivers().enumerate() {
            println!("{}: Renderer: {}", i + 1, r.name);
        }
        return;
    }
    // Game width (Used on [`ffmpeg::VideoRecorder`])
    let width = cli.width.unwrap_or(1280);
    // Game height (Used on [`ffmpeg::VideoRecorder`])
    let height = cli.height.unwrap_or(720);
    // Cube size (it will try to fit as much as possible without overfilling)

    // Showing width for showing stuff like FPS text
    let showing_w = width + 150;
    // Showing height for showing stuff like overfills (round corners sucks)
    let showing_h = height;

    let cube_size: u32 = cli.cube_size.unwrap_or(10);

    let vsync = cli.vsync.unwrap_or(false);
    let record = cli.record.unwrap_or(false);
    let length = cli.length.map(|l| humantime::parse_duration(&l).expect("Wrong duration format. Please take a look at https://docs.rs/humantime/latest/humantime/fn.parse_duration.html"));

    // Initialize SDL2
    let ctx = sdl2::init().unwrap();
    let video = ctx.video().unwrap();

    let window = video
        .window("Game Of Life", showing_w, showing_h)
        .position_centered()
        .resizable()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().accelerated();
    if vsync {
        canvas = canvas.present_vsync();
    }
    if let Some(renderer) = cli.selected_gpu_renderer {
        canvas = canvas.index(renderer - 1);
    }
    let mut canvas = canvas.build().unwrap();

    let mut event = ctx.event_pump().unwrap();

    let mut cubes: HashMap<(i32, i32), Life> = HashMap::new();

    // Initialize the cubes
    for y in 0..(height / cube_size) as i32 {
        for x in 0..(width / cube_size) as i32 {
            cubes.insert(
                (x * cube_size as i32, y * cube_size as i32),
                Life {
                    x: x * cube_size as i32,
                    y: y * cube_size as i32,
                    state: LifeState::Dead,
                },
            );
        }
    }

    // [`Game`] instance
    let mut game = Game { cubes, cube_size };
    let tc = canvas.texture_creator();
    canvas.set_draw_color(sdl2::pixels::Color::BLACK);

    // Pre-rendering dead surface color so I can save time (Optimization gaming)
    let mut dead_surface =
        sdl2::surface::Surface::new(cube_size, cube_size, sdl2::pixels::PixelFormatEnum::RGB24)
            .unwrap();
    // Same reason for [`dead_surface`]
    let mut alive_surface =
        sdl2::surface::Surface::new(cube_size, cube_size, sdl2::pixels::PixelFormatEnum::RGB24)
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
    let mut vr: Option<std::sync::Arc<std::sync::Mutex<ffmpeg::VideoRecorder>>> = None;

    if record {
        vr = Some(std::sync::Arc::new(std::sync::Mutex::new(
            ffmpeg::VideoRecorder::new(
                "out.mp4",
                width,
                height,
                video.desktop_display_mode(0).unwrap().refresh_rate as u32,
            ),
        )));
        let cloned_vr = std::sync::Arc::clone(&vr.clone().unwrap());
        println!("Recording...");
        ctrlc::set_handler(move || {
            cloned_vr.lock().unwrap().kill();
        })
        .expect("Failed to listen for CTRL-C (Force exiting with FFMpeg)");
    } else {
        println!("Playing normally...");
    }

    let mut run_sim = false;
    let mut last_cord = (0, 0);

    'main_loop: loop {
        for e in event.poll_iter() {
            match e {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'main_loop,
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Space),
                    ..
                } => {
                    run_sim = true;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::R),
                    ..
                } => {
                    if !run_sim {
                        game.cubes = {
                            let mut new_cubes = HashMap::new();
                            for (pos, life) in game.cubes.iter() {
                                new_cubes.insert(
                                    *pos,
                                    Life {
                                        x: life.x,
                                        y: life.y,
                                        state: LifeState::random_life_state(),
                                    },
                                );
                            }
                            new_cubes
                        }
                    }
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::C),
                    ..
                } => {
                    if !run_sim {
                        game.cubes.iter_mut().for_each(|(_, l)| {
                            l.state = LifeState::Dead;
                        });
                    }
                }
                sdl2::event::Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    // round them again
                    if !run_sim {
                        let x = x / cube_size as i32 * cube_size as i32;
                        let y = y / cube_size as i32 * cube_size as i32;
                        if let Some(life) = game.cubes.get_mut(&(x, y)) {
                            if mouse_btn == sdl2::mouse::MouseButton::Left {
                                life.state = if life.state == LifeState::Alive {
                                    LifeState::Dead
                                } else {
                                    LifeState::Alive
                                }
                            }
                        }
                    }
                }
                sdl2::event::Event::MouseMotion {
                    x, y, mousestate, ..
                } => {
                    // println!("Mouse at ({}, {})", x, y);
                    // round those cord to nearest cube
                    if !run_sim {
                        let x = x / cube_size as i32 * cube_size as i32;
                        let y = y / cube_size as i32 * cube_size as i32;
                        if (x, y) == last_cord {
                            continue;
                        }
                        if let Some(life) = game.cubes.get_mut(&(x, y)) {
                            if mousestate.left() {
                                life.state = if life.state == LifeState::Alive {
                                    LifeState::Dead
                                } else {
                                    LifeState::Alive
                                }
                            }
                        }
                        last_cord = (x, y);
                    }
                }
                _ => {}
            }
        }
        canvas.clear();
        // draw [`Life`]
        for life in game.cubes.values() {
            let color = match life.state {
                LifeState::Alive => &alive_texture,
                LifeState::Dead => &dead_texture,
            };
            let rect = sdl2::rect::Rect::new(life.x as i32, life.y as i32, cube_size, cube_size);
            canvas.copy(color, None, rect).unwrap();
        }
        // draw grid
        for y in (0..height).step_by(cube_size as usize) {
            canvas
                .draw_line(
                    sdl2::rect::Point::new(0, y as i32),
                    sdl2::rect::Point::new(width as i32, y as i32),
                )
                .unwrap();
        }
        for x in (0..width).step_by(cube_size as usize) {
            canvas
                .draw_line(
                    sdl2::rect::Point::new(x as i32, 0),
                    sdl2::rect::Point::new(x as i32, height as i32),
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
        if run_sim {
            if elasped.as_millis() >= 250 || vr.is_none() {
                update_time = std::time::Instant::now();
                game.apply_rules_to_each_lifes();
                match vr.as_mut() {
                    Some(v) => {
                        let mut v = v.lock().unwrap();
                        v.process_frame(
                            canvas
                                .read_pixels(
                                    sdl2::rect::Rect::new(0, 0, width, height),
                                    sdl2::pixels::PixelFormatEnum::RGB24,
                                )
                                .unwrap(),
                        );
                        if length.is_some() {
                            if let Some(status) = v.get_render_status() {
                                if status.time >= length.unwrap() {
                                    break 'main_loop;
                                }
                            }
                        }
                    }
                    None => {}
                }
            }
        } else if vr.is_some() && run_sim {
            game.apply_rules_to_each_lifes();
            match vr.as_mut() {
                Some(v) => {
                    let mut v = v.lock().unwrap();
                    v.process_frame(
                        canvas
                            .read_pixels(
                                sdl2::rect::Rect::new(0, 0, width, height),
                                sdl2::pixels::PixelFormatEnum::RGB24,
                            )
                            .unwrap(),
                    );
                    if length.is_some() {
                        if let Some(status) = v.get_render_status() {
                            if status.time >= length.unwrap() {
                                break 'main_loop;
                            }
                        }
                    }
                }
                None => {}
            }
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
                    (showing_w - fps_text.width()) as i32,
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
                    (showing_w - mf_text.width()) as i32,
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
                    (showing_w - lfp_text.width()) as i32,
                    80,
                    lfp_text.width(),
                    lfp_text.height(),
                ),
            )
            .unwrap();
        canvas.present();
    }
    // Done feeding frames. Now showing result
    match vr {
        Some(v) => {
            let mut a = v.lock().unwrap();
            a.done();
        }
        None => {}
    }
}

/// Truncate float with [`precision`] as how many digits you needed in final result
fn truncate(b: f64, precision: usize) -> f64 {
    f64::trunc(b * ((10 * precision) as f64)) / ((10 * precision) as f64)
}
