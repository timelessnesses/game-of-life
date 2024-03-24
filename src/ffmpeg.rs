/// FFMpeg rendering module
use std::{self, io::Write};

/// [`VideoRecorder`] struct for wrapping around FFMpeg for rendering video by passing frames in [`Vec<u8>`]
pub struct VideoRecorder {
    ffmpeg: std::process::Child,
}

impl VideoRecorder {
    /// Spawns new instance of FFMpeg with out file, size and FPS
    pub fn new(out: &str, width: u32, height: u32, fps: u32) -> Self {
        let ffmpeg_cmd = std::process::Command::new("ffmpeg")
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
                out,
            ])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("FFMpeg failed to start");
        Self { ffmpeg: ffmpeg_cmd }
    }

    /// Function for passing the frames to FFMpeg. This doesn't cost a lot performance.
    pub fn process_frame(&mut self, frame: Vec<u8>) {
        self.ffmpeg
            .stdin
            .as_mut()
            .unwrap()
            .write_all(frame.as_slice())
            .unwrap();
    }

    /// Finalizing rendering. Wait for FFMpeg to exit
    pub fn done(self) {
        let _ = self
            .ffmpeg
            .wait_with_output()
            .expect("Failed to wait for FFMpeg to exit");
        println!("Success");
    }
}
