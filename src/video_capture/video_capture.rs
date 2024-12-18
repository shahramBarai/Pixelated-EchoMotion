use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use opencv::{
    prelude::*,
    videoio::{self, VideoCapture},
};

pub struct VideoSource {
    capture: VideoCapture,
    pub frame: Arc<Mutex<Mat>>,
}

impl VideoSource {
    pub fn new() -> Result<Self> {
        Ok(Self {
            capture: VideoCapture::default()?,
            frame: Arc::new(Mutex::new(Mat::default())),
        })
    }

    pub fn set_source_file(&mut self, file_path: &String) -> Result<()> {
        if !std::path::Path::new(file_path).exists() {
            bail!("File does not exist: {}", file_path);
        }
        self.capture.open_file(file_path, videoio::CAP_ANY)?;
        if !self.capture.is_opened()? {
            bail!("Unable to open video file: {}", file_path);
        }
        Ok(())
    }

    pub fn set_source_webcam(&mut self, webcam_index: i32) -> Result<()> {
        self.capture.open(webcam_index, videoio::CAP_ANY)?;
        if !self.capture.is_opened()? {
            bail!("Unable to open the webcam!");
        }
        Ok(())
    }

    pub fn set_resolution(&mut self, width: i32, height: i32) -> Result<()> {
        self.capture
            .set(videoio::CAP_PROP_FRAME_WIDTH, width as f64)?;
        self.capture
            .set(videoio::CAP_PROP_FRAME_HEIGHT, height as f64)?;
        Ok(())
    }

    pub fn update_frame(&mut self) -> Result<bool> {
        let mut frame = Mat::default();
        self.capture.read(&mut frame)?;
        if frame.empty() {
            return Ok(false);
        }

        // Update the shared frame
        let mut shared_frame = self.frame.lock().unwrap();
        *shared_frame = frame;

        Ok(true)
    }
}
