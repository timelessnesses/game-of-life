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
