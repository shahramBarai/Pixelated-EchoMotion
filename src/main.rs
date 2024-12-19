mod frame_processing;
mod gui_interaction;
mod particle_system;
mod video_capture;

use frame_processing::FrameProcessor;
use gui_interaction::Window;
use particle_system::{EffectType, ParticleSystem};
use rand::Rng;
use video_capture::VideoSource;

use anyhow::{Ok, Result}; // Automatically handle the error types
use opencv::{
    core::{self, Point},
    highgui::wait_key,
    imgproc,
    prelude::*,
};

use std::{fs, path::PathBuf, sync::Arc, time::Duration};

// Define the constants
const PIXEL_SIZE: i32 = 10; // Define maximum possible pixel size
const PIXEL_SPACING: i32 = 0; // Define the spacing between pixels
const WINDOW_NAME: &str = "Window"; // Define the name of the window
const WINDOW_WIDTH: i32 = 960; // Define the width of the window
const WINDOW_HEIGHT: i32 = 540; // Define the height of the window
const VIDEO_RESOLUTION_WIDTH: i32 = 1920; // Define the width of the video resolution
const VIDEO_RESOLUTION_HEIGHT: i32 = 1080; // Define the height of the video resolution
const OBJECTS_INTERFERENCE_DISTANCE: i32 = 10; // Define the distance to detect interference
const WEBCAM_CONTRAST: f64 = 1.0; // Define the video contrast
const WEBCAM_BRIGHTNESS: f64 = 90.0; // Define the video brightness

fn detect_interference(
    point_1: Point,
    point_2: Point,
    output: &mut Mat,
    draw: bool,
) -> Result<bool> {
    if point_1.x == 0 && point_1.y == 0 && point_2.x == 0 && point_2.y == 0 {
        return Ok(false);
    }

    let dx = (point_1.x - point_2.x) as f64;
    let dy = (point_1.y - point_2.y) as f64;
    let distance = (dx * dx + dy * dy).sqrt() as i32;

    if draw {
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
    }

    Ok(distance < OBJECTS_INTERFERENCE_DISTANCE)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        println!(
            "Usage: {} [webcam <webcam_index> | file <video_path_1>] <folder_for_video_sources> [print_info | print_time_logs]",
            args[0]
        );
        return Ok(());
    }

    // Initialize the first video source
    let mut video_source_1 = VideoSource::new((VIDEO_RESOLUTION_WIDTH, VIDEO_RESOLUTION_HEIGHT))?;
    if args[1] == "webcam" {
        video_source_1.set_source_webcam(args[2].parse::<i32>()?)?;
        video_source_1.set_contrast(WEBCAM_CONTRAST);
        video_source_1.set_brightness(WEBCAM_BRIGHTNESS);
    } else {
        video_source_1.set_source_file(&args[2])?;
    }
    video_source_1.update_frame()?;

    // Read all video files from the folder specified in args[3]
    let video_folder = std::path::Path::new(&args[3]);
    let mut video_files = fs::read_dir(video_folder)?
        .filter_map(|entry| entry.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "mp4")) // Filter for .mp4 files
        .collect::<Vec<PathBuf>>();

    // Sort files to have a consistent order
    video_files.sort();

    if video_files.is_empty() {
        eprintln!("No video files found in the specified folder: {}", args[3]);
        return Ok(());
    }

    // Initialize second video source with the first video in the folder
    let mut video_source_2 = VideoSource::new((VIDEO_RESOLUTION_WIDTH, VIDEO_RESOLUTION_HEIGHT))?;
    let mut current_video_index = 0;
    video_source_2.set_source_file(
        &video_files[current_video_index]
            .to_str()
            .unwrap()
            .to_string(),
    )?;
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

    let mut start_next_video = false;

    loop {
        // Measure loop start time
        let loop_start = std::time::Instant::now();

        // Update the first video source frame
        if !video_source_1.update_frame()? {
            video_source_1.set_source_file(&args[2])?;
            video_source_1.update_frame()?;
        }

        // Update the second video source frame
        if !particle_system.get_animation_status(1)? {
            if !video_source_2.update_frame()? || start_next_video {
                start_next_video = false; // Reset the flag

                // If the current video ended, move to the next one
                current_video_index = (current_video_index + 1) % video_files.len();
                video_source_2.set_source_file(
                    &video_files[current_video_index]
                        .to_str()
                        .unwrap()
                        .to_string(),
                )?;
                if !video_source_2.update_frame()? {
                    // If the new video also fails (shouldn't happen), just continue
                    continue;
                }
            }
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
        if particle_system.get_animation_status(1)? {
            particle_system.update(point_1).await?;
            object_1 = frame_processor.extract_object(0).await?;

            // Add the object to the particle system
            extract_object_time = std::time::Instant::now()
                - loop_start
                - frame_processing_time
                - closest_points_time;

            particle_system
                .add_object(Arc::clone(&frame1), &object_1, 0)
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

            if detect_interference(point_1, point_2, &mut particle_system.output_frame, false)? {
                particle_system.set_animation_status(1, true);
                let rundom_number = Rng::gen_range(&mut rand::thread_rng(), 0..3);
                let effect = match rundom_number {
                    0 => EffectType::Explosion,
                    2 => EffectType::Break,
                    _ => EffectType::Explosion,
                };
                particle_system.set_effect_type(1, effect);

                // Start the next video after the interference effect
                start_next_video = true;

                // Print the interference message
                if args.len() > 4 && args[4] == "print_info" {
                    println!("Interference detected! Effect: {:?}", effect);
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

        // Print the time logs
        if args.len() > 4 && args[4] == "print_time_logs" {
            println!(
                "{:<25} {:<25} {:<25} {:<25} {:<25} {:<25}",
                "Frame processing time:",
                "Closest points time:",
                "Extract object time:",
                "Add object time:",
                "P-system update time:",
                "Loop time:"
            );
            println!(
                "{:<25?} {:<25?} {:<25?} {:<25?} {:<25?} {:<25?}",
                frame_processing_time,
                closest_points_time,
                extract_object_time,
                add_object_time,
                particle_system_update_time,
                loop_time
            );
        }

        // Sleep asynchronously to avoid high CPU usage
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    Ok(())
}
