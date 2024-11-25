mod frame_processing;
mod particle_system;

use std::sync::{Arc, Mutex};

use anyhow::Result; // Automatically handle the error types
use frame_processing::{
    detect_interference_near_point, highlight_objects_with_contours, pixelate_frame,
};
use particle_system::Effect;

use opencv::{
    core::{self, Point, Scalar},
    highgui::{self, wait_key, WINDOW_NORMAL},
    prelude::*,
    videoio,
};

// Define the constants
const PIXEL_SIZE: i32 = 2; // Define maximum possible pixel size
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

    let mouse_coords = Arc::new(Mutex::new(Point::new(0, 0)));

    // Clone the Arc for the callback
    let mouse_coords_callback = Arc::clone(&mouse_coords);

    // Define the callback function
    let callback = Box::new(move |event: i32, x: i32, y: i32, _: i32| {
        if event == highgui::EVENT_MOUSEMOVE || event == highgui::EVENT_LBUTTONDOWN {
            let mut coords = mouse_coords_callback.lock().unwrap();
            coords.x = x;
            coords.y = y;
        }
    });

    // Set the mouse callback for the window
    highgui::set_mouse_callback(WINDOW_NAME, Some(callback))?;

    let mut effect = Effect::new(WINDOW_WIDTH, WINDOW_HEIGHT, PIXEL_SIZE, PIXEL_SPACING);

    loop {
        // Set the output frame to white before drawing the pixelated image
        processed_frame.set_to(&core::Scalar::all(255.0), &core::no_array())?;

        // Get the latest mouse coordinates
        {
            let coords = mouse_coords.lock().unwrap();
            effect.mouse_coords = *coords;
            if !effect.get_animation_status()
                && detect_interference_near_point(&frame, *coords, 10)?
            {
                effect.init(&frame)?;
                println!("Interference detected");
            }
        }

        if effect.get_animation_status() {
            effect.update();
            effect.draw(&mut processed_frame)?;
        } else {
            video.read(&mut frame)?;
            if frame.empty() {
                println!("End of stream");
                break; // End of stream
            }
        }

        // // Pixelate the frame
        // pixelate_frame(
        //     &frame,
        //     &mut processed_frame,
        //     PIXEL_SIZE,
        //     PIXEL_SPACING,
        //     false,
        // )?;

        // // Highlight objects with contours
        // highlight_objects_with_contours(&frame, &mut processed_frame)?;

        // Display the frame_show in the window
        if effect.get_animation_status() {
            highgui::imshow(WINDOW_NAME, &processed_frame)?;
        } else {
            highgui::imshow(WINDOW_NAME, &frame)?;
        }

        // Break the loop when the user presses the 'q'
        if wait_key(1)? == 113 {
            println!("Exit");
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}
