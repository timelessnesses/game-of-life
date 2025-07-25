// #![windows_subsystem = "windows"]
use crate::core::{Game, Life, LifeState};
use crate::utils::{truncate, word_wrap};
use clap::Parser;
/// timelessnesses' implementation of Conway's Game Of Life in SDL2.
use std::collections::HashMap;
use utils::{create_grid_texture, render_text_as_texture};

mod core;
mod ffmpeg;
mod utils;

#[derive(clap::Parser)]
#[command(author = "timelessnesses", about = "Nothing")]
struct Cli {
    /// List GPU renderers (for the SELECTED_GPU_RENDERER arg)
    #[arg(long)]
    list_gpu_renderers: bool,
    /// Select your own renderer if you want to
    #[arg(short, long)]
    selected_gpu_renderer: Option<u32>,

    /// Force VSync
    #[arg(short, long, default_value_t = false)]
    vsync: bool,

    /// Record the game to a video file
    #[arg(short, long, default_value_t = false)]
    record: bool,

    /// Length of the video file
    #[arg(short, long)]
    length: Option<String>,

    /// Width of the window (default: 1280)
    #[arg(short, long, default_value_t = 1280)]
    width: u32,

    /// Height of the window (default: 720)
    #[arg(long, default_value_t = 720)]
    height: u32,

    /// Cube size (default: 10)
    #[arg(short, long)]
    cube_size: Option<u32>,

    /// Run simulation in every pre-defined time (next_simulation argument) or run simulation in every frames (defaults to run simulation in every pre-defined time)
    #[arg(short, long, default_value_t = false)]
    output_still_frame: bool,

    /// How long until next simulation (in milliseconds)
    #[arg(short, long, default_value_t = 250)]
    next_simulation: u64,
}

/// Font
const ROBOTO: &[u8; 167000] = include_bytes!("assets/Roboto-Light.ttf");

fn main() {
    let cli = Cli::parse();
    if cli.list_gpu_renderers {
        println!("Available GPU renderers:");
        for (i, r) in sdl2::render::drivers().enumerate() {
            let mut flags = vec![];
            if r.flags & 0x00000001 != 0 {
                flags.push("Software Fallback");
            }
            if r.flags & 0x00000002 != 0 {
                flags.push("Hardware Accelerated");
            }
            if r.flags & 0x00000004 != 0 {
                flags.push("Present Vsync");
            }
            if r.flags & 0x00000008 != 0 {
                flags.push("Target Texture");
            }
            println!("{}: Renderer: {}", i + 1, r.name);
            println!("  Texture Formats Supported: {:?}", r.texture_formats);
            println!("  Max Texture Width: {}", r.max_texture_width);
            println!("  Max Texture Height: {}", r.max_texture_height);
            println!("  Rendering Capability: {}", flags.join(", "));
            println!();
        }
        return;
    }
    // Game width (Used on [`ffmpeg::VideoRecorder`])
    let width = cli.width;
    // Game height (Used on [`ffmpeg::VideoRecorder`])
    let height = cli.height;
    // Cube size (it will try to fit as much as possible without overfilling)

    // Showing width for showing stuff like FPS text
    let showing_w = width + 150;
    // Showing height for showing stuff like overfills (round corners sucks)
    let showing_h = height;

    let cube_size: u32 = cli.cube_size.unwrap_or(10);

    let vsync = cli.vsync;
    let record = cli.record;
    let length = cli.length.map(|l| humantime::parse_duration(&l).expect("Wrong duration format. Please take a look at https://docs.rs/humantime/latest/humantime/fn.parse_duration.html"));
    let output_still_frame = cli.output_still_frame;
    let next_simulation = cli.next_simulation;

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
    if vsync || record {
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

    let mut update_time = std::time::Instant::now();

    let font_ctx = sdl2::ttf::init().unwrap();

    let fps_font = font_ctx
        .load_font_from_rwops(sdl2::rwops::RWops::from_bytes(ROBOTO).unwrap(), 15)
        .unwrap();

    let rendered_rand_sim_text = render_text_as_texture(
        word_wrap(
            "Press R to get a random grid of lifes",
            showing_w - width,
            &fps_font,
        )
        .into_iter(),
        &fps_font,
        &tc,
        sdl2::pixels::Color::WHITE,
        sdl2::pixels::Color::BLACK,
    );
    let rendered_clear_sim_text = render_text_as_texture(
        word_wrap(
            "Press C to clear the grid of lifes",
            showing_w - width,
            &fps_font,
        )
        .into_iter(),
        &fps_font,
        &tc,
        sdl2::pixels::Color::WHITE,
        sdl2::pixels::Color::BLACK,
    );
    let mut rendered_play_sim_text = render_text_as_texture(
        word_wrap(
            "Press Space to start the simulation (Will also start recording if it's on)",
            showing_w - width,
            &fps_font,
        )
        .into_iter(),
        &fps_font,
        &tc,
        sdl2::pixels::Color::WHITE,
        sdl2::pixels::Color::BLACK,
    );
    let rendered_draw_sim_text = render_text_as_texture(
        word_wrap(
            "You can hold your left mouse button to draw a shape",
            showing_w - width,
            &fps_font,
        )
        .into_iter(),
        &fps_font,
        &tc,
        sdl2::pixels::Color::WHITE,
        sdl2::pixels::Color::BLACK,
    );
    let rendered_status_text = render_text_as_texture(
        word_wrap(
            &format!(
                "Recording: {}\nLength: {}\nNext Simulation: {}ms",
                if record { "ON" } else { "OFF" },
                if let Some(l) = length {
                    format!(
                        "{}:{}:{}",
                        l.as_secs() / 60 / 60,
                        l.as_secs() / 60,
                        l.as_secs() % 60
                    )
                } else {
                    "N/A".to_string()
                },
                next_simulation
            ),
            showing_w - width,
            &fps_font,
        )
        .into_iter(),
        &fps_font,
        &tc,
        sdl2::pixels::Color::WHITE,
        sdl2::pixels::Color::BLACK,
    );
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
        println!("Recording will start once started simulation...");
        ctrlc::set_handler(move || {
            cloned_vr.lock().unwrap().kill();
        })
        .expect("Failed to listen for CTRL-C (Force exiting with FFMpeg)");
    } else {
        println!("Playing normally...");
    }

    let mut grid_texture = tc
        .create_texture(None, sdl2::render::TextureAccess::Target, width, height)
        .unwrap();
    grid_texture.set_blend_mode(sdl2::render::BlendMode::Blend);
    create_grid_texture(&mut canvas, &mut grid_texture, width, height, cube_size);

    let mut cell_texture = tc
        .create_texture_streaming(
            None,
            (width / cube_size) as u32,
            (height / cube_size) as u32,
        )
        .unwrap();

    let mut run_sim = false;
    let mut last_cord = (0, 0);

    'main_loop: loop {
        for e in event.poll_iter() {
            match e {
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(_, _),
                    ..
                } => {
                    // no idea why you have to redraw the grid every resizes...
                    create_grid_texture(&mut canvas, &mut grid_texture, width, height, cube_size);
                }
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'main_loop,
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Space),
                    ..
                } => {
                    if !record {
                        run_sim = !run_sim;
                    } else {
                        run_sim = true;
                    }
                    rendered_play_sim_text = render_text_as_texture(word_wrap(
                            if run_sim {"Running simulation. Press Space to pause it. (You can't pause while recording, however.)"} else {"Press Space to start the simulation (Will also start recording if it's on)"},
                            showing_w - width,
                            &fps_font,
                        )
                        .into_iter(),
                        &fps_font,
                        &tc,
                        sdl2::pixels::Color::WHITE,
                        sdl2::pixels::Color::BLACK,
                    );
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
        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas.clear();
        canvas.set_draw_color(sdl2::pixels::Color::WHITE);

        // draw [`Life`]
        /* canvas.fill_rects(game.cubes.values().filter(|i| {
            i.state == LifeState::Alive
        }).map(|i| {
            sdl2::rect::Rect::new(i.x, i.y, cube_size, cube_size)
        }).collect::<Vec<_>>().as_slice()).unwrap(); */

        cell_texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..(height / cube_size) as usize {
                    for x in 0..(width / cube_size) as usize {
                        let idx = y * pitch + x * 4;
                        let alive = game.cubes
                            [&(x as i32 * cube_size as i32, y as i32 * cube_size as i32)]
                            .state
                            == LifeState::Alive;
                        let color = if alive {
                            sdl2::pixels::Color::WHITE
                        } else {
                            sdl2::pixels::Color::GRAY
                        };
                        buffer[idx + 0] = color.r;
                        buffer[idx + 1] = color.g;
                        buffer[idx + 2] = color.b;
                        buffer[idx + 3] = color.a;
                    }
                }
            })
            .unwrap();
        canvas
            .copy(
                &cell_texture,
                None,
                sdl2::rect::Rect::new(0, 0, width, height),
            )
            .unwrap();

        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        // draw grid
        canvas
            .copy(
                &grid_texture,
                None,
                sdl2::rect::Rect::new(0, 0, width, height),
            )
            .unwrap();

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
        if (elasped.as_millis() >= next_simulation as u128 && !record) && run_sim {
            update_time = std::time::Instant::now();
            game.apply_rules_to_each_lifes();
        } else if run_sim {
            if output_still_frame && record {
                if elasped.as_millis() >= next_simulation as u128 {
                    update_time = std::time::Instant::now();
                    game.apply_rules_to_each_lifes();
                }
            } else if record {
                game.apply_rules_to_each_lifes();
            }
            if let Some(v) = vr.as_mut() {
                let mut v = v.lock().unwrap();
                v.process_frame(
                    &canvas
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
        let mut ys = 120u32;
        let groups = [
            &rendered_clear_sim_text,
            &rendered_rand_sim_text,
            &rendered_status_text,
            &rendered_draw_sim_text,
            &rendered_play_sim_text,
        ];
        groups.iter().for_each(|g| {
            g.iter().for_each(|s| {
                canvas
                    .copy(
                        s,
                        None,
                        sdl2::rect::Rect::new(
                            (showing_w - s.query().width) as i32,
                            ys as i32,
                            s.query().width,
                            s.query().height,
                        ),
                    )
                    .unwrap();
                ys += s.query().height + 10;
            });
            ys += 20;
        });

        canvas.present();
    }
    // Done feeding frames. Now showing result
    if let Some(v) = vr {
        let mut a = v.lock().unwrap();
        a.done();
    }
}
