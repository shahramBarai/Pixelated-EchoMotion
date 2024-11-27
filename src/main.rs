mod frame_processing;
mod gui_interaction;
mod particle_system;
mod video_capture;

use gui_interaction::Window;
use video_capture::VideoSource;

use std::sync::{Arc, Mutex};

use anyhow::Result; // Automatically handle the error types
use frame_processing::detect_interference; // Import the detect_interference_near_point function
use frame_processing::pixelate_frame; // Import the pixelate_frame function
use particle_system::{particle_system::EffectType, Effect};

use opencv::{
    core::{self, Point},
    highgui::wait_key,
    prelude::*,
};

// Define the constants
const PIXEL_SIZE: i32 = 5; // Define maximum possible pixel size
const PIXEL_SPACING: i32 = 0; // Define the spacing between pixels
const WINDOW_NAME: &str = "Window"; // Define the name of the window
const WINDOW_WIDTH: i32 = 960; // Define the width of the window
const WINDOW_HEIGHT: i32 = 540; // Define the height of the window
const VIDEO_RESOLUTION_WIDTH: i32 = 1920; // Define the width of the video resolution
const VIDEO_RESOLUTION_HEIGHT: i32 = 1080; // Define the height of the video resolution
const OBJECTS_INTERFERENCE_DISTANCE: i32 = 100; // Define the distance to detect interference

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
    let mut videio_source_1 = VideoSource::new()?;
    videio_source_1.set_source_file(&args[2])?;
    videio_source_1.set_resolution(VIDEO_RESOLUTION_WIDTH, VIDEO_RESOLUTION_HEIGHT)?;
    let mut videio_source_2 = VideoSource::new()?;
    videio_source_2.set_source_file(&args[3])?;
    videio_source_2.set_resolution(VIDEO_RESOLUTION_WIDTH, VIDEO_RESOLUTION_HEIGHT)?;

    // Initialize gui window and handle mouse events
    let window = Window::new(WINDOW_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)?;
    let mouse_coords = Arc::new(Mutex::new(Point::new(0, 0)));
    window.handle_mouse_events(mouse_coords.clone())?;

    // Create a frame to display the output
    videio_source_1.update_frame()?;
    videio_source_2.update_frame()?;
    let mut output_frame = videio_source_1.frame.clone();

    // Initialize the particle system effect
    let mut effect = Effect::new(
        videio_source_1.frame.size()?,
        PIXEL_SIZE,
        PIXEL_SPACING,
        OBJECTS_INTERFERENCE_DISTANCE,
    );
    effect.set_effect_type(EffectType::Push);

    loop {
        // Set the output frame to white before drawing the pixelated image
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

        // If the effect is active, update and draw the particles else pixelate the frame
        if effect.get_animation_status() {
            effect.update();
            effect.draw(&mut output_frame)?;
        } else {
            match videio_source_1.update_frame() {
                Ok(()) => {}
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
            match videio_source_2.update_frame() {
                Ok(()) => {}
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
            // pixelate_frame(
            //     &videio_source.frame,
            //     &mut output_frame,
            //     PIXEL_SIZE,
            //     PIXEL_SPACING,
            // )
            // .await?;

            detect_interference(
                &videio_source_1.frame,
                &videio_source_2.frame,
                &mut output_frame,
                OBJECTS_INTERFERENCE_DISTANCE,
            )?;
        }

        // Show the output frame in the window
        window.show(&output_frame)?;

        // Break the loop when the user presses the 'q'
        if wait_key(1)? == 113 {
            println!("Exit");
            break;
        }

        // Sleep for a short duration to avoid high CPU usage
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}
