mod frame_processing;
mod gui_interaction;
mod particle_system;
mod video_capture;

use gui_interaction::Window;
use video_capture::VideoSource;

use std::sync::{Arc, Mutex};

use anyhow::Result; // Automatically handle the error types
use frame_processing::{detect_interference_near_point, pixelate_frame};
use particle_system::Effect;

use opencv::{
    core::{self, Point},
    highgui::wait_key,
    prelude::*,
};

// Define the constants
const PIXEL_SIZE: i32 = 2; // Define maximum possible pixel size
const PIXEL_SPACING: i32 = 0; // Define the spacing between pixels
const WINDOW_NAME: &str = "Window"; // Define the name of the window
const WINDOW_WIDTH: i32 = 960; // Define the width of the window
const WINDOW_HEIGHT: i32 = 540; // Define the height of the window
const OBJECTS_INTERFERENCE_DISTANCE: i32 = 3000; // Define the distance to detect interference

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} [webcam|file <video_path>]", args[0]);
        return Ok(());
    }

    // Initialize the video source
    let mut videio_source = VideoSource::new(&args)?;

    let mut effect = Effect::new(
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        PIXEL_SIZE,
        PIXEL_SPACING,
        OBJECTS_INTERFERENCE_DISTANCE,
    );

    // Initialize gui window and handle mouse events
    let window = Window::new(WINDOW_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)?;
    let mouse_coords = Arc::new(Mutex::new(Point::new(0, 0)));
    window.handle_mouse_events(mouse_coords.clone())?;

    // Create a frame to display the output
    videio_source.update_frame()?;
    let mut output_frame = videio_source.frame.clone();

    loop {
        // Set the output frame to white before drawing the pixelated image
        output_frame.set_to(&core::Scalar::all(255.0), &core::no_array())?;

        // Get the latest mouse coordinates
        {
            let coords = mouse_coords.lock().unwrap();
            effect.mouse_coords = *coords;
            if !effect.get_animation_status()
                && detect_interference_near_point(&videio_source.frame, *coords, 10)?
            {
                effect.init(&videio_source.frame)?;
                println!("Interference detected");
            }
        }

        // If the effect is active, update and draw the particles else pixelate the frame
        if effect.get_animation_status() {
            effect.update();
            effect.draw(&mut output_frame)?;
        } else {
            match videio_source.update_frame() {
                Ok(()) => {}
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
            pixelate_frame(
                &videio_source.frame,
                &mut output_frame,
                PIXEL_SIZE,
                PIXEL_SPACING,
                false,
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
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}
