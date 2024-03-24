use std::{io::Write, path::PathBuf};

struct IteratorThroughDiskReadThing {
    current_index: usize,
    files: Vec<PathBuf>,
}
impl Iterator for IteratorThroughDiskReadThing {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        self.current_index += 1;
        if self.current_index - 1 >= self.files.len() {
            return None;
        }
        let x = std::fs::read(self.files[self.current_index - 1].to_owned()).unwrap();
        std::fs::remove_file(self.files[self.current_index - 1].to_owned()).unwrap();
        return Some(x);
    }
}

impl IteratorThroughDiskReadThing {
    fn new(folder_name: String) -> Self {
        let mut things = Vec::new();
        let files = std::fs::read_dir(folder_name).unwrap();
        let mut ordering = Vec::new();
        for file in files {
            ordering.push(file.unwrap().path());
        }
        ordering.sort_by_key(|x| {
            x.to_str()
                .unwrap()
                .split('\\')
                .last()
                .and_then(|s| s.split(".").next())
                .and_then(|n| n.parse::<i32>().ok())
                .unwrap_or(0)
        });
        things.append(&mut ordering);
        println!("{:#?}", things);
        Self {
            files: things,
            current_index: 0,
        }
    }
}

pub struct VideoRecorder {
    ffmpeg: std::process::Child,
}

impl VideoRecorder {
    pub fn new(out: &str, width: u32, height: u32, fps: u32) -> Self {
        let mut ffmpeg_cmd = std::process::Command::new("ffmpeg")
            .args([
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "-s",
                &format!("{}x{}", width, height),
                "-r",
                &format!("{}", fps),
                "-i",
                "pipe:0",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-preset",
                "veryslow",
                "-y",
                out
            ])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("FFMpeg failed to start");
        Self {
            ffmpeg: ffmpeg_cmd
        }
    }

    pub fn process_frame(&mut self, frame: Vec<u8>) {
        self.ffmpeg.stdin.as_mut().unwrap().write_all(frame.as_slice()).unwrap();
    }

    pub fn done(self) {
        let _ = self.ffmpeg.wait_with_output().expect("Failed to wait for FFMpeg to exit");
        println!("Success");
    }
}
