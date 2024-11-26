use anyhow::{bail, Result};
use opencv::{
    prelude::*,
    videoio::{self, VideoCapture},
};

pub struct VideoSource {
    capture: VideoCapture,
    pub frame: Mat,
}

impl VideoSource {
    pub fn new(args: &[String], resolution_width: i32, resolution_height: i32) -> Result<Self> {
        let mut capture = VideoCapture::default()?;
        let mut webcam_index = 0;

        if args[1] == "webcam" && args.len() == 3 {
            webcam_index = args[2].parse::<i32>()?;
            capture.open(webcam_index, videoio::CAP_ANY)?;
            if !capture.is_opened()? {
                bail!("Unable to open the webcam!");
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
            bail!("Invalid arguments! Use 'webcam <webcam_index>' or 'file <video_path>'.");
        }

        if args[1] == "file" || webcam_index != 0 {
            capture.set(videoio::CAP_PROP_FRAME_WIDTH, resolution_width as f64)?;
            capture.set(videoio::CAP_PROP_FRAME_HEIGHT, resolution_height as f64)?;
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
