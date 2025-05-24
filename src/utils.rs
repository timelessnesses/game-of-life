/// Truncate float with [`precision`] as how many digits you needed in final result
pub fn truncate(b: f64, precision: usize) -> f64 {
    f64::trunc(b * ((10 * precision) as f64)) / ((10 * precision) as f64)
}

/// Basic word wrap based on [`sdl2::ttf::Font`] and [`max_width`]
pub fn word_wrap(text: &str, max_width: u32, font: &sdl2::ttf::Font<'_, '_>) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    // Split text by newlines first
    for raw_line in text.split('\n') {
        let words = raw_line.split_whitespace();
        let mut current_line = String::new();

        for word in words {
            let test_line = current_line.clone() + word + " ";
            let (test_width, _) = font.size_of(&test_line).unwrap();

            if test_width <= max_width {
                current_line = test_line;
            } else {
                lines.push(current_line.trim_end().to_string());
                current_line = word.to_owned() + " ";
            }
        }

        if !current_line.trim().is_empty() {
            lines.push(current_line.trim_end().to_string());
        }
    }

    lines
}

pub fn render_text_as_texture<'a, T>(
    segments: impl Iterator<Item = (impl AsRef<str> + 'a)>,
    font: &sdl2::ttf::Font<'a, 'a>,
    texture_creator: &'a sdl2::render::TextureCreator<T>,
    shading_color: sdl2::pixels::Color,
    shading_background: sdl2::pixels::Color,
) -> Vec<sdl2::render::Texture<'a>> {
    segments
        .into_iter()
        .map(|s| {
            texture_creator
                .create_texture_from_surface(
                    font.render(s.as_ref())
                        .shaded(shading_color, shading_background)
                        .unwrap(),
                )
                .unwrap()
        })
        .collect()
}

pub fn create_grid_texture<T: sdl2::render::RenderTarget>(
    canvas: &mut sdl2::render::Canvas<T>,
    grid_texture: &mut sdl2::render::Texture<'_>,
    width: u32,
    height: u32,
    cube_size: u32,
) {
    canvas
        .with_texture_canvas(grid_texture, |texture| {
            texture.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 0));
            texture.clear();
            texture.set_draw_color(sdl2::pixels::Color::BLACK);
            for y in (0..height).step_by(cube_size as usize) {
                texture
                    .draw_line(
                        sdl2::rect::Point::new(0, y as i32),
                        sdl2::rect::Point::new(width as i32, y as i32),
                    )
                    .unwrap();
            }
            for x in (0..width).step_by(cube_size as usize) {
                texture
                    .draw_line(
                        sdl2::rect::Point::new(x as i32, 0),
                        sdl2::rect::Point::new(x as i32, height as i32),
                    )
                    .unwrap();
            }
        })
        .unwrap();
}
