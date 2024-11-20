use anyhow::Result; // Automatically handle the error types
use opencv::{core, highgui, imgproc, prelude::*, videoio}; // Note, the namespace of OpenCV is changed (to better or worse). It is no longer one enormous.

const MAX_SIZE: i32 = 10; // Define maximum possible pixel size
const MIN_SIZE: i32 = 0; // Define minimum possible pixel size
const SPACING: i32 = 0; // Define the spacing between pixels
const VIDEO_RESOLUTION_WIDTH: i32 = 1920; // Define the width of the video resolution
const VIDEO_RESOLUTION_HEIGHT: i32 = 1080; // Define the height of the video resolution
const VIDEO_DEVICE: i32 = 4; // Define the video device
const WINDOW_WIDTH: i32 = 960; // Define the width of the window
const WINDOW_HEIGHT: i32 = 540; // Define the height of the window

fn main() -> Result<()> {
    // Note, this is anyhow::Result
    // Open a GUI window and set the size of it
    highgui::named_window("window", highgui::WINDOW_NORMAL)?;
    highgui::resize_window("window", WINDOW_WIDTH, WINDOW_HEIGHT)?;

    // Open the web-camera (assuming you have one)
    let mut cam = videoio::VideoCapture::new(VIDEO_DEVICE, videoio::CAP_ANY)?;
    cam.set(videoio::CAP_PROP_FRAME_WIDTH, VIDEO_RESOLUTION_WIDTH as f64)?;
    cam.set(
        videoio::CAP_PROP_FRAME_HEIGHT,
        VIDEO_RESOLUTION_HEIGHT as f64,
    )?;

    // Check if the camera is opened
    if !cam.is_opened()? {
        panic!("Unable to open default camera!");
    }

    // Create two arrays to store the web-can data and the processed data
    // This array will store the web-cam data
    let mut frame_cam = Mat::default();
    cam.read(&mut frame_cam)?; // Read the first frame

    // Initialize frame_show with the same size and type as frame_cam
    let mut frame_show =
        Mat::new_size_with_default(frame_cam.size()?, frame_cam.typ(), core::Scalar::all(255.0))?;

    loop {
        cam.read(&mut frame_cam)?;

        // Reset frame_show to white
        frame_show.set_to(&core::Scalar::all(255.0), &core::no_array())?;

        let mut y = 0;
        while y <= frame_cam.rows() {
            let mut x = 0;
            while x <= frame_cam.cols() {
                if x + MAX_SIZE <= frame_cam.cols() && y + MAX_SIZE <= frame_cam.rows() {
                    let start_point = core::Point::new(x, y);
                    draw_filled_square_with_background_color(
                        &mut frame_show,
                        &mut frame_cam,
                        start_point,
                        false,
                    )?;
                }
                x += MAX_SIZE + SPACING;
            }
            y += MAX_SIZE + SPACING;
        }

        // Display the frame_show in the window
        highgui::imshow("window", &frame_show)?;

        // Break the loop when the user presses the 'q'
        let key = highgui::wait_key(1)?;
        if key == 113 {
            break;
        }
    }
    Ok(())
}

fn draw_filled_square_with_background_color(
    frame_show: &mut Mat,
    frame_cam: &mut Mat,
    start_point: core::Point,
    map_size: bool,
) -> Result<()> {
    // Calculate average color within the circle's region
    let mut rect = core::Rect::new(start_point.x, start_point.y, MAX_SIZE, MAX_SIZE);

    // Ensure the rectangle is within the frame
    let sub_mat = Mat::roi(frame_cam, rect)?;
    let avg_color = core::mean(&sub_mat, &core::no_array())?;

    if map_size {
        // Map brightness to size
        let brightness = (avg_color[0] + avg_color[1] + avg_color[2]) as i32 / 3;
        let mapped_size = map_brightness_to_size(brightness);
        let center = mapped_size / 2;
        rect.x = start_point.x - center;
        rect.y = start_point.y - center;
        rect.width = mapped_size;
        rect.height = mapped_size;
    }

    // Draw filled rectangle with the average color
    imgproc::rectangle(frame_show, rect, avg_color, -1, imgproc::LINE_AA, 0)?;
    Ok(())
}

fn map_brightness_to_size(brightness: i32) -> i32 {
    // Invert brightness and map to size
    let inverted_brightness = 255 - brightness;
    let mapped_size = MIN_SIZE + (inverted_brightness * (MAX_SIZE - MIN_SIZE)) / 255;
    mapped_size.clamp(MIN_SIZE, MAX_SIZE) // Ensure within min/max bounds
}
