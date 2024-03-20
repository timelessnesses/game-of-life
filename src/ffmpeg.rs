
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
            return None
        }
        let x = std::fs::read(self.files[self.current_index - 1].to_owned()).unwrap();
        std::fs::remove_file(self.files[self.current_index - 1].to_owned()
    ).unwrap();
        return Some(x)
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
            x.to_str().unwrap().split('\\')
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
    out: String,
    saver: Saver,
    width: u32,
    height: u32,
    fps: u32,
}

#[derive(Debug, Clone)]
pub struct DiskSaver {
    folder_name: String,
    count: i64,
}

#[derive(Debug, Clone)]
pub enum Saver {
    Disk(DiskSaver),
    Memory(MemorySaver)
}

impl DiskSaver {
    fn save_frame(&mut self, frame: Vec<u8>) {
        std::fs::write(format!("{}/{}.frame", self.folder_name, self.count), frame).unwrap();
        self.count += 1;
    }

    pub fn new() -> Self
    where
        Self: Sized,
    {
        let _ = std::fs::remove_dir_all("frames");
        std::fs::create_dir("frames").unwrap();
        Self {
            folder_name: "frames".to_owned(),
            count: 0,
        }
    }

    fn get_frames(&self) -> IteratorThroughDiskReadThing {
        let x = IteratorThroughDiskReadThing::new("frames".to_string());
        return x
    }

    fn cleanup(&mut self) {
        std::fs::remove_dir_all("frames").unwrap();
    }

}

#[derive(Debug, Clone)]
pub struct MemorySaver {
    frames: Vec<Vec<u8>>,
}

impl MemorySaver {
    pub fn new() -> Self
    where
        Self: Sized,
    {
        Self { frames: Vec::new() }
    }

    fn save_frame(&mut self, frame: Vec<u8>) {
        self.frames.push(frame);
    }

    fn get_frames(&self) -> std::vec::IntoIter<Vec<u8>> {
        return self.frames.clone().into_iter();
    }

    #[allow(dropping_references)]
    fn cleanup(&mut self) {
        drop(self);
    }
}

impl Saver {
    fn save_frame(&mut self, frame: Vec<u8>) {
        match self {
            Self::Disk(d) => d.save_frame(frame),
            Self::Memory(m) => m.save_frame(frame)
        }
    }

    fn get_frames(&self) -> Box<dyn Iterator<Item = Vec<u8>>> {
        match self {
            Self::Disk(d) => Box::new(d.get_frames()),
            Self::Memory(m) => Box::new(m.get_frames())
        }
    }

    fn cleanup(&mut self) {
        match self {
            Self::Disk(d) => d.cleanup(),
            Saver::Memory(m) => m.cleanup()
        }
    }
}

impl VideoRecorder {
    pub fn new(s: Saver, out: &str, width: u32, height: u32, fps: u32) -> Self {
        println!("Recording every frames storing with {:#?}", s);
        Self {
            out: out.to_owned(),
            saver: s,
            width,
            height,
            fps,
        }
    }

    pub fn save_frame(&mut self, frame: Vec<u8>) {
        self.saver.save_frame(frame);
    }

    pub fn process_frames(&mut self) {
        let f = self.saver.get_frames();
        let mut ffmpeg_cmd = std::process::Command::new("ffmpeg")
            .args([
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "-s",
                &format!("{}x{}", self.width, self.height),
                "-r",
                &format!("{}", self.fps),
                "-i",
                "pipe:0",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-preset",
                "veryslow",
                "-y",
                self.out.as_str(),
            ])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start FFMpeg");

        let stdin = ffmpeg_cmd
            .stdin
            .as_mut()
            .expect("Failed to open FFMpeg stdin");
        for frame in f {
            stdin.write_all(frame.as_slice()).expect("Failed to write frame");
        }
        #[allow(dropping_references)]
        drop(stdin);
        let _ = ffmpeg_cmd
            .wait_with_output()
            .expect("Failed to wait FFMpeg to exit");
        println!("Done");
        self.saver.cleanup();
    }
}
