mod frame_processing;
mod gui_interaction;
mod particle_system;
mod video_capture;

use frame_processing::FrameProcessor;
use gui_interaction::Window;
use rand::Rng;
use video_capture::VideoSource;

use std::{sync::Arc, time::Duration};

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

fn detect_interference(point_1: Point, point_2: Point, output: &mut Mat) -> Result<bool> {
    if point_1.x == 0 && point_1.y == 0 && point_2.x == 0 && point_2.y == 0 {
        return Ok(false);
    }

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

    Ok(distance < OBJECTS_INTERFERENCE_DISTANCE)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!(
            "Usage: {} [webcam <webcam_index>|file <video_path_1> <video_path_2>]",
            args[0]
        );
        return Ok(());
    }

    // Initialize the video sources
    let mut video_source_1 = VideoSource::new()?;
    video_source_1.set_source_file(&args[2])?;
    video_source_1.set_resolution(VIDEO_RESOLUTION_WIDTH, VIDEO_RESOLUTION_HEIGHT)?;
    video_source_1.update_frame()?;

    let mut video_source_2 = VideoSource::new()?;
    video_source_2.set_source_file(&args[3])?;
    video_source_2.set_resolution(VIDEO_RESOLUTION_WIDTH, VIDEO_RESOLUTION_HEIGHT)?;
    video_source_2.update_frame()?;

    // Initialize the particle system effect
    let mut particle_system = ParticleSystem::new(
        video_source_1.frame.lock().unwrap().size()?,
        PIXEL_SIZE,
        PIXEL_SPACING,
        OBJECTS_INTERFERENCE_DISTANCE * 1000,
    );
    particle_system.init(&video_source_1.frame.lock().unwrap(), 2)?;

    // Initialize the frame processor
    let mut frame_processor = FrameProcessor::new(PIXEL_SIZE, PIXEL_SPACING, 200.0);
    frame_processor.init(2);

    // Initialize GUI window and mouse events
    let window = Window::new(WINDOW_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)?;

    let mut object_1: Vec<Point> = Vec::new();
    let mut object_2: Vec<Point> = Vec::new();
    let mut point_1 = Point::new(0, 0);
    let mut point_2 = Point::new(0, 0);

    loop {
        // Measure loop start time
        let loop_start = std::time::Instant::now();

        // Update the frames from the video sources
        if !video_source_1.update_frame()? {
            break;
        }
        if !video_source_2.update_frame()? {
            break;
        }

        // Clear output frame
        particle_system.clean_output_frame()?;

        // Access frames without cloning
        let frame1 = {
            let locked_frame = video_source_1.frame.lock().unwrap();
            Arc::clone(&Arc::new(locked_frame.clone()))
        };
        let frame2 = {
            let locked_frame = video_source_2.frame.lock().unwrap();
            Arc::clone(&Arc::new(locked_frame.clone()))
        };

        // Convert the frames to grayscale (black and white)
        frame_processor.convert_to_grayscale(&frame1, 0)?;
        frame_processor.convert_to_grayscale(&frame2, 1)?;

        // Find the contours of the objects in the frames
        frame_processor.find_object_contour(0)?;
        frame_processor.find_object_contour(1)?;

        // Measure frame processing time
        let frame_processing_time = std::time::Instant::now() - loop_start;

        // Draw the contours of the objects in the frames
        // frame_processor.draw_contours(&mut particle_system.output_frame)?;

        (point_1, point_2) = frame_processor.find_closest_points(0, 1).await?;

        // Measure the closest points calculation time
        let closest_points_time = std::time::Instant::now() - loop_start - frame_processing_time;
        let mut extract_object_time = std::time::Duration::new(0, 0);
        let mut add_object_time = std::time::Duration::new(0, 0);

        // Update the particle system
        if particle_system.get_animation_status(0)? {
            particle_system.update(point_2).await?;
            object_2 = frame_processor.extract_object(1).await?;

            // Add the object to the particle system
            extract_object_time = std::time::Instant::now()
                - loop_start
                - frame_processing_time
                - closest_points_time;

            particle_system
                .add_object(Arc::clone(&frame2), &object_2, 1)
                .await?;

            // Measure the add object time
            add_object_time = std::time::Instant::now()
                - loop_start
                - frame_processing_time
                - closest_points_time
                - extract_object_time;
        } else {
            // Extract the objects from the frames (black pixels)
            object_1 = frame_processor.extract_object(0).await?;
            object_2 = frame_processor.extract_object(1).await?;

            // Measure the extract object time
            extract_object_time = std::time::Instant::now()
                - loop_start
                - frame_processing_time
                - closest_points_time;

            // Add the objects to the particle system
            particle_system
                .add_object(Arc::clone(&frame1), &object_1, 0)
                .await?;
            particle_system
                .add_object(Arc::clone(&frame2), &object_2, 1)
                .await?;

            // Measure the add object time
            add_object_time = std::time::Instant::now()
                - loop_start
                - frame_processing_time
                - closest_points_time
                - extract_object_time;

            if detect_interference(point_1, point_2, &mut particle_system.output_frame)? {
                println!("Interference detected");
                particle_system.set_animation_status(0, true);
                // Randomly choose the effect type for the particle system
                let rundom_number = Rng::gen_range(&mut rand::thread_rng(), 0..3);
                match rundom_number {
                    0 => particle_system.set_effect_type(0, EffectType::Explosion),
                    1 => particle_system.set_effect_type(0, EffectType::Push),
                    2 => particle_system.set_effect_type(0, EffectType::Break),
                    _ => particle_system.set_effect_type(0, EffectType::Push),
                }
            }
        }

        // Measure the particle system update time
        let particle_system_update_time =
            std::time::Instant::now() - loop_start - closest_points_time;

        // Show the output frame in the window
        particle_system.draw()?;
        window.show(&particle_system.output_frame)?;

        // Exit on 'q' key
        if wait_key(1)? == 113 {
            println!("Exit");
            break;
        }

        // Measure the total loop time
        let loop_time = std::time::Instant::now() - loop_start;
        println!(
            "Extract object: {:?}, Add object: {:?}, Particle system update: {:?}, Loop: {:?}",
            extract_object_time, add_object_time, particle_system_update_time, loop_time
        );

        // Sleep asynchronously to avoid high CPU usage
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    Ok(())
}
