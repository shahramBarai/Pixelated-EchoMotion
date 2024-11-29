mod frame_processing;
mod gui_interaction;
mod particle_system;
mod video_capture;

use frame_processing::FrameProcessor;
use gui_interaction::Window;
use video_capture::VideoSource;

use std::sync::{Arc, Mutex};

use anyhow::{Ok, Result}; // Automatically handle the error types
use particle_system::{EffectType, ParticleSystem}; // Import the ParticleSystem and EffectType

use opencv::{
    core::{self, Point},
    highgui::wait_key,
    imgproc,
    prelude::*,
};

// Define the constants
const PIXEL_SIZE: i32 = 1; // Define maximum possible pixel size
const PIXEL_SPACING: i32 = 0; // Define the spacing between pixels
const WINDOW_NAME: &str = "Window"; // Define the name of the window
const WINDOW_WIDTH: i32 = 960; // Define the width of the window
const WINDOW_HEIGHT: i32 = 540; // Define the height of the window
const VIDEO_RESOLUTION_WIDTH: i32 = 1920; // Define the width of the video resolution
const VIDEO_RESOLUTION_HEIGHT: i32 = 1080; // Define the height of the video resolution
const OBJECTS_INTERFERENCE_DISTANCE: i32 = 100; // Define the distance to detect interference

fn detect_interference(point_1: Point, point_2: Point, output: &mut Mat) -> Result<()> {
    let dx = (point_1.x - point_2.x) as f64;
    let dy = (point_1.y - point_2.y) as f64;
    let distance = (dx * dx + dy * dy).sqrt() as i32;

    // Draw points and a line between the two closest points
    imgproc::circle(
        output,
        point_1,
        5,
        core::Scalar::new(0.0, 0.0, 255.0, 0.0),
        -1,
        imgproc::LINE_AA,
        0,
    )?;
    imgproc::circle(
        output,
        point_2,
        5,
        core::Scalar::new(0.0, 0.0, 255.0, 0.0),
        -1,
        imgproc::LINE_AA,
        0,
    )?;

    if distance < OBJECTS_INTERFERENCE_DISTANCE {
        imgproc::line(
            output,
            point_1,
            point_2,
            core::Scalar::new(0.0, 0.0, 255.0, 0.0),
            2,
            imgproc::LINE_AA,
            0,
        )?;
    } else {
        imgproc::line(
            output,
            point_1,
            point_2,
            core::Scalar::new(255.0, 0.0, 0.0, 0.0),
            2,
            imgproc::LINE_AA,
            0,
        )?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!(
            "Usage: {} [webcam <webcam_index>|file <video_path_1> <vido_path_2>]",
            args[0]
        );
        return Ok(());
    }

    // Initialize the video sources
    let mut videio_source_0 = VideoSource::new()?;
    videio_source_0.set_source_file(&args[2])?;
    videio_source_0.set_resolution(VIDEO_RESOLUTION_WIDTH, VIDEO_RESOLUTION_HEIGHT)?;
    let mut videio_source_1 = VideoSource::new()?;
    videio_source_1.set_source_file(&args[3])?;
    videio_source_1.set_resolution(VIDEO_RESOLUTION_WIDTH, VIDEO_RESOLUTION_HEIGHT)?;
    videio_source_0.update_frame()?;
    videio_source_1.update_frame()?;

    // Initialize the particle system effect
    let mut particle_system = ParticleSystem::new(
        videio_source_0.frame.size()?,
        PIXEL_SIZE,
        PIXEL_SPACING,
        OBJECTS_INTERFERENCE_DISTANCE,
    );
    particle_system.init(&videio_source_0.frame, 2)?;

    // Initialize the frame processor
    let mut frame_processor = FrameProcessor::new(PIXEL_SIZE, PIXEL_SPACING, 200.0);
    frame_processor.init(2);

    // Initialize gui window and handle mouse events
    let window = Window::new(WINDOW_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)?;
    let mouse_coords = Arc::new(Mutex::new(Point::new(0, 0)));
    window.handle_mouse_events(mouse_coords.clone())?;

    let mut object_1 = Vec::new();
    let mut object_2 = Vec::new();

    let mut point_1 = Point::new(0, 0);
    let mut point_2 = Point::new(0, 0);

    let mut output_frame = videio_source_0.frame.clone();

    loop {
        // Update the frames from the video sources
        if !videio_source_0.update_frame()? {
            break;
        }

        if !videio_source_1.update_frame()? {
            break;
        }

        // Clear the output frame
        particle_system.clean_output_frame()?;
        output_frame.set_to(&core::Scalar::all(255.0), &core::no_array())?;

        // // Get the latest mouse coordinates
        // {
        //     let coords = mouse_coords.lock().unwrap();
        //     effect.mouse_coords = *coords;
        //     if !effect.get_animation_status()
        //         && detect_interference_near_point(&videio_source.frame, *coords, 10)?
        //     {
        //         effect.init(&videio_source.frame)?;
        //         println!("Interference detected");
        //     }
        // }

        // Convert the frames to grayscale (black and white)
        frame_processor.convert_to_grayscale(&videio_source_0.frame, 0)?;
        frame_processor.convert_to_grayscale(&videio_source_1.frame, 1)?;

        //frame_processor.draw_mask(&mut particle_system.output_frame, 0)?;

        // // Extract the objects from the frames (black pixels)
        object_1 = frame_processor.extract_object(0)?;
        object_2 = frame_processor.extract_object(1)?;

        // // Add the objects to the particle system
        particle_system.add_oject(&videio_source_0.frame, &object_1, 0)?;
        particle_system.add_oject(&videio_source_1.frame, &object_2, 1)?;

        // Find the contours of the objects in the frames
        frame_processor.find_object_contour(0)?;
        frame_processor.find_object_contour(1)?;

        // // Draw the contours of the objects in the frames
        frame_processor.draw_contours(&mut particle_system.output_frame)?;

        (point_1, point_2) = frame_processor.find_closest_points(0, 1)?;

        // Draw the closest points in the frames
        detect_interference(point_1, point_2, &mut particle_system.output_frame)?;

        // Show the output frame in the window
        particle_system.draw()?;
        window.show(&particle_system.output_frame)?;

        // Break the loop when the user presses the 'q'
        if wait_key(1)? == 113 {
            println!("Exit");
            break;
        }

        // Sleep for a short duration to avoid high CPU usage
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}
