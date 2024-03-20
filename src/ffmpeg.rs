
use std::{io::Write, path::PathBuf};

#[derive(Debug)]
pub enum SavingType {
    Disk,
    Memory,
}

struct IteratorThroughDiskReadThing {
    folder: String,
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
        return Some(std::fs::read(self.files[self.current_index - 1].to_owned()).unwrap())
    }
}

impl IteratorThroughDiskReadThing {
    fn new(folder_name: String) -> Self {
        let mut things = Vec::new();
        for file in std::fs::read_dir(&folder_name).unwrap() {
            things.push(file.unwrap().path());
        }
        Self {
            folder: folder_name,
            files: things,
            current_index: 0,
        }
    }
}

pub struct VideoRecorder {
    out: String,
    saver: Box<dyn Saver>,
    width: u32,
    height: u32,
    fps: u32,
}

#[derive(Debug)]
struct DiskSaver {
    folder_name: String,
    count: i64,
}

pub trait Saver {
    type FramesRT: Iterator<Item = Vec<u8>> + Sized;
    fn new() -> Self where Self: Sized;
    fn save_frame(&mut self, frame: Vec<u8>);
    fn get_frames(&self) -> Self::FramesRT;
    fn cleanup(&mut self);
}

impl Saver for DiskSaver {
    type FramesRT = IteratorThroughDiskReadThing;
    fn save_frame(&mut self, frame: Vec<u8>) {
        std::fs::write(format!("{}/{}.frame", self.folder_name, self.count), frame).unwrap();
        self.count += 1;
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        std::fs::remove_dir_all("frames");
        std::fs::create_dir("frames").unwrap();
        Self {
            folder_name: "frames".to_owned(),
            count: 0,
        }
    }

    fn get_frames(&self) -> Self::FramesRT {
        let x = IteratorThroughDiskReadThing::new("frames".to_string());
        return x
    }

    fn cleanup(&mut self) {
        std::fs::remove_dir_all("frames");
    }

}

#[derive(Debug)]
struct MemorySaver {
    frames: Vec<Vec<u8>>,
}

impl Saver for MemorySaver {
    type FramesRT = std::vec::IntoIter<Vec<u8>>;
    fn new() -> Self
    where
        Self: Sized,
    {
        Self { frames: Vec::new() }
    }

    fn save_frame(&mut self, frame: Vec<u8>) {
        self.frames.push(frame);
    }

    fn get_frames(&self) -> Self::FramesRT {
        return self.frames.clone().into_iter();
    }

    fn cleanup(&mut self) {
        drop(self)
    }
}

impl VideoRecorder {
    pub fn new(s: SavingType, out: &str, width: u32, height: u32, fps: u32) -> Self {
        println!("Recording every frames storing with {:#?}", s);
        Self {
            out: out.to_owned(),
            saver: match s {
                SavingType::Disk => Box::new(DiskSaver::new()),
                SavingType::Memory => Box::new(MemorySaver::new()),
            },
            width,
            height,
            fps,
        }
    }

    pub fn save_frame(&mut self, frame: Vec<u8>) {
        self.saver.save_frame(frame)
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
        drop(stdin);
        let _ = ffmpeg_cmd
            .wait_with_output()
            .expect("Failed to wait FFMpeg to exit");
        println!("Done");
        self.saver.cleanup();
    }
}
