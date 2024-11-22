mod frame_processing;

use anyhow::Result; // Automatically handle the error types
use frame_processing::{highlight_objects_with_contours, pixelate_frame};
use opencv::{
    core::{self, Scalar},
    highgui::{self, wait_key, WINDOW_NORMAL},
    prelude::*,
    videoio,
};

// Define the constants
const PIXEL_SIZE: i32 = 5; // Define maximum possible pixel size
const PIXEL_SPACING: i32 = 0; // Define the spacing between pixels

const WINDOW_NAME: &str = "Window"; // Define the name of the window
const WINDOW_WIDTH: i32 = 960; // Define the width of the window
const WINDOW_HEIGHT: i32 = 540; // Define the height of the window

const VIDEO_DEVICE: i32 = 0; // Define the video device
const VIDEO_RESOLUTION_WIDTH: i32 = 1920; // Define the width of the video resolution
const VIDEO_RESOLUTION_HEIGHT: i32 = 1080; // Define the height of the video resolution

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} [webcam|file <video_path>]", args[0]);
        return Ok(());
    }

    // Open a GUI window and set the size of it
    highgui::named_window(WINDOW_NAME, WINDOW_NORMAL)?;
    highgui::resize_window(WINDOW_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)?;

    let mut video = videoio::VideoCapture::default()?;

    if args[1] == "webcam" {
        video.open(VIDEO_DEVICE, videoio::CAP_ANY)?;
        if !video.is_opened()? {
            panic!("Unable to open the webcam!");
        }
        if VIDEO_DEVICE != 0 {
            video.set(videoio::CAP_PROP_FRAME_WIDTH, VIDEO_RESOLUTION_WIDTH as f64)?;
            video.set(
                videoio::CAP_PROP_FRAME_HEIGHT,
                VIDEO_RESOLUTION_HEIGHT as f64,
            )?;
        }
    } else if args[1] == "file" && args.len() == 3 {
        let file_path = &args[2];
        // Check if the file exists
        if !std::path::Path::new(file_path).exists() {
            panic!("File does not exist: {}", file_path);
        }
        video.open_file(&file_path, videoio::CAP_ANY)?;
        if !video.is_opened()? {
            panic!("Unable to open video file: {}", file_path);
        }
    } else {
        panic!("Invalid arguments! Use 'webcam' or 'file <video_path>'.");
    };

    let mut frame = Mat::default();
    video.read(&mut frame)?;
    let mut processed_frame =
        Mat::new_size_with_default(frame.size()?, frame.typ(), Scalar::all(255.0))?;

    let mut pre_processed_frame =
        Mat::new_size_with_default(frame.size()?, frame.typ(), Scalar::all(255.0))?;

    // let mut lower_threshold: i32 = 120;
    // let mut upper_threshold: i32 = 300;

    // highgui::create_trackbar(
    //     "Lower Threshold",
    //     WINDOW_NAME,
    //     Some(&mut lower_threshold),
    //     255,
    //     None,
    // )?;
    // highgui::create_trackbar(
    //     "Upper Threshold",
    //     WINDOW_NAME,
    //     Some(&mut upper_threshold),
    //     1000,
    //     None,
    // )?;

    let mut object_pixels: Vec<(core::Point, core::Scalar)> = Vec::new();

    loop {
        video.read(&mut frame)?;
        if frame.empty() {
            println!("End of stream");
            break; // End of stream
        }

        // Set the output frame to white before drawing the pixelated image
        processed_frame.set_to(&core::Scalar::all(255.0), &core::no_array())?;
        // Set the output frame to green before drawing the pixelated image
        pre_processed_frame.set_to(&core::Scalar::new(0.0, 255.0, 0.0, 0.0), &core::no_array())?;

        // Pixelate the frame
        pixelate_frame(
            &frame,
            &mut processed_frame,
            PIXEL_SIZE,
            PIXEL_SPACING,
            false,
        )?;

        // Highlight objects with contours
        // highlight_objects_with_contours(&frame, &mut processed_frame)?;

        // Extract the object from the frame
        object_pixels = frame_processing::extract_object(&processed_frame, 200 as f64)?;

        // Draw the object pixels on the processed frame
        for (point, color) in object_pixels.iter() {
            *pre_processed_frame.at_2d_mut::<core::Vec3b>(point.y, point.x)? =
                core::Vec3b::from([color[0] as u8, color[1] as u8, color[2] as u8]);
        }

        // Display the frame_show in the window
        highgui::imshow(WINDOW_NAME, &pre_processed_frame)?;

        // Break the loop when the user presses the 'q'
        if wait_key(1)? == 113 {
            println!("Exit");
            break;
        }
        //std::thread::sleep(std::time::Duration::from_millis(500));
    }
    Ok(())
}
