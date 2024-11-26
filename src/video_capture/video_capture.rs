use anyhow::{bail, Result};
use opencv::{
    prelude::*,
    videoio::{self, VideoCapture},
};

const VIDEO_DEVICE: i32 = 0; // Define the video device
const VIDEO_RESOLUTION_WIDTH: i32 = 1920; // Define the width of the video resolution
const VIDEO_RESOLUTION_HEIGHT: i32 = 1080; // Define the height of the video resolution

pub struct VideoSource {
    capture: VideoCapture,
    pub frame: Mat,
}

impl VideoSource {
    pub fn new(args: &[String]) -> Result<Self> {
        let mut capture = VideoCapture::default()?;

        if args[1] == "webcam" {
            capture.open(VIDEO_DEVICE, videoio::CAP_ANY)?;
            if !capture.is_opened()? {
                bail!("Unable to open the webcam!");
            }
            if VIDEO_DEVICE != 0 {
                capture.set(videoio::CAP_PROP_FRAME_WIDTH, VIDEO_RESOLUTION_WIDTH as f64)?;
                capture.set(
                    videoio::CAP_PROP_FRAME_HEIGHT,
                    VIDEO_RESOLUTION_HEIGHT as f64,
                )?;
            }
        } else if args[1] == "file" && args.len() == 3 {
            let file_path = &args[2];
            if !std::path::Path::new(file_path).exists() {
                bail!("File does not exist: {}", file_path);
            }
            capture.open_file(file_path, videoio::CAP_ANY)?;
            if !capture.is_opened()? {
                bail!("Unable to open video file: {}", file_path);
            }
        } else {
            bail!("Invalid arguments! Use 'webcam' or 'file <video_path>'.");
        }

        Ok(Self {
            capture,
            frame: Mat::default(),
        })
    }

    pub fn update_frame(&mut self) -> Result<()> {
        self.capture.read(&mut self.frame)?;
        if self.frame.empty() {
            bail!("End of video stream!");
        }
        Ok(())
    }
}
