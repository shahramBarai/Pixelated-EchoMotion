use anyhow::Result; // Automatically handle the error types
use opencv::{core, highgui, imgproc, prelude::*, videoio}; // Note, the namespace of OpenCV is changed (to better or worse). It is no longer one enormous.

const MAX_RADIUS: i32 = 3; // Define maximum possible radius
const MIN_RADIUS: i32 = 0; // Define minimum possible radius
const CIRCLE_RADIUS: i32 = MAX_RADIUS; // Define the radius here for easy adjustment
const CIRCLES_SPACING: i32 = 0; // Define the spacing between circles

fn main() -> Result<()> {
    // Note, this is anyhow::Result
    // Open a GUI window
    highgui::named_window("window", highgui::WINDOW_FULLSCREEN)?;
    // Open the web-camera (assuming you have one)
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    let mut frame_cam = Mat::default(); // This array will store the web-cam data
                                        // Read the camera
                                        // and display in the window

    cam.read(&mut frame_cam)?;
    let mut frame_show = Mat::default();

    // Get frame height and width as integers
    let frame_height = frame_cam.rows();
    let frame_width = frame_cam.cols();

    println!("Frame height: {}", frame_height);
    println!("Frame width: {}", frame_width);

    loop {
        cam.read(&mut frame_cam)?;

        // Set the show frame to white
        frame_show = Mat::new_size_with_default(
            frame_cam.size()?,
            frame_cam.typ(),
            core::Scalar::all(255.0),
        )?;

        let mut y = 0;
        while y < frame_height {
            let mut x = 0;
            while x < frame_width {
                let center = core::Point::new(x + CIRCLE_RADIUS, y + CIRCLE_RADIUS);
                draw_filled_circle_with_background_color(
                    &mut frame_show,
                    &mut frame_cam,
                    center,
                    CIRCLE_RADIUS,
                )?;
                x += (CIRCLE_RADIUS * 2) + CIRCLES_SPACING;
            }
            y += (CIRCLE_RADIUS * 2) + CIRCLES_SPACING;
        }

        highgui::imshow("window", &frame_show)?;
        let key = highgui::wait_key(1)?;
        if key == 113 {
            // quit with q
            break;
        }
    }
    Ok(())
}

fn draw_filled_circle_with_background_color(
    frame_show: &mut Mat,
    frame_cam: &mut Mat,
    center: core::Point,
    radius: i32,
) -> Result<()> {
    // Calculate average color within the circle's region
    let rect = core::Rect::new(center.x - radius, center.y - radius, radius * 2, radius * 2);

    // Ensure the rectangle is within frame bounds
    let safe_rect = rect & core::Rect::new(0, 0, frame_cam.cols(), frame_cam.rows());
    let sub_mat = Mat::roi(frame_cam, safe_rect)?;

    let avg_color = core::mean(&sub_mat, &core::no_array())?;
    let brightness = (avg_color[0] + avg_color[1] + avg_color[2]) as i32 / 3;

    // Map brightness to radius
    let mapped_radius = map_brightness_to_radius(brightness);

    // Draw filled circle with the average color
    imgproc::circle(
        frame_show,
        center,
        mapped_radius,
        core::Scalar::new(0.0, 0.0, 0.0, 0.0),
        -1, // Fill the circle
        imgproc::LINE_AA,
        0,
    )?;

    Ok(())
}

fn draw_filled_circle_with_color(
    frame: &mut Mat,
    center: core::Point,
    radius: i32,
    color: core::Scalar,
) -> Result<()> {
    // Draw filled circle with the specified color
    imgproc::circle(frame, center, radius, color, -1, imgproc::LINE_AA, 0)?;
    Ok(())
}

fn map_brightness_to_radius(brightness: i32) -> i32 {
    // Invert brightness and map to radius
    let inverted_brightness = 255 - brightness;
    let mapped_radius = MIN_RADIUS + (inverted_brightness * (MAX_RADIUS - MIN_RADIUS)) / 255;
    mapped_radius.clamp(MIN_RADIUS, MAX_RADIUS) // Ensure within min/max bounds
}
