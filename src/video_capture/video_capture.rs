use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use opencv::{
    core::{self, flip},
    imgproc,
    prelude::*,
    videoio::{self, VideoCapture},
};

pub struct VideoSource {
    capture: VideoCapture,
    pub frame: Arc<Mutex<Mat>>,
    resolution: (i32, i32),
    source_type: String,
    constrast: f64,
    brightness: f64,
}

impl VideoSource {
    pub fn new(resolution: (i32, i32)) -> Result<Self> {
        Ok(Self {
            capture: VideoCapture::default()?,
            frame: Arc::new(Mutex::new(Mat::default())),
            resolution,
            source_type: String::from(""),
            constrast: 1.0,
            brightness: 0.0,
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

        self.source_type = "file".to_string();
        Ok(())
    }

    pub fn set_source_webcam(&mut self, webcam_index: i32) -> Result<()> {
        self.capture.open(webcam_index, videoio::CAP_ANY)?;
        if !self.capture.is_opened()? {
            bail!("Unable to open the webcam!");
        }

        self.source_type = "webcam".to_string();
        Ok(())
    }

    pub fn set_contrast(&mut self, contrast: f64) {
        self.constrast = contrast;
    }

    pub fn set_brightness(&mut self, brightness: f64) {
        self.brightness = brightness;
    }

    pub fn update_frame(&mut self) -> Result<bool> {
        let mut frame = Mat::default();
        self.capture.read(&mut frame)?;
        if frame.empty() {
            return Ok(false);
        }

        // Resize the frame to the desired resolution
        let mut resized_frame = Mat::default();
        imgproc::resize(
            &frame,
            &mut resized_frame,
            core::Size::new(self.resolution.0, self.resolution.1),
            0.0,
            0.0,
            imgproc::INTER_LINEAR,
        )?;

        if self.source_type == "webcam" {
            let mut bright_frame = Mat::default();
            resized_frame.convert_to(&mut bright_frame, -1, self.constrast, self.brightness)?;

            // Flip the frame vertically
            let mut flipped_frame = Mat::default();
            flip(&bright_frame, &mut flipped_frame, 1)?;

            // Update the shared frame with the brightened frame
            let mut shared_frame = self.frame.lock().unwrap();
            *shared_frame = flipped_frame;
        } else {
            // Update the shared frame with the resized frame
            let mut shared_frame = self.frame.lock().unwrap();
            *shared_frame = resized_frame;
        }

        Ok(true)
    }
}
