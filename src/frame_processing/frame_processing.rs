use std::sync::{Arc, Mutex};

use anyhow::Result;
use opencv::{
    core::{self, Point, Rect, Size, VecN},
    imgproc,
    prelude::*,
};

use tokio::task;

pub async fn pixelate_frame(
    input: &Mat,
    output: &mut Mat,
    pixel_size: i32,
    spacing: i32,
) -> Result<()> {
    let rows = input.rows();
    let cols = input.cols();
    let rows_per_pixel = (rows + pixel_size - 1) / pixel_size;
    let num_cpus = num_cpus::get() as i32;
    let chunk_size = (rows_per_pixel + num_cpus - 1) / num_cpus;

    // Vector to hold all async tasks
    let mut tasks = Vec::new();

    let mut chunk_y = 0;
    while chunk_y < chunk_size {
        let input_clone = input.clone(); // Clone input matrix for each task

        // Spawn async task for this chunk
        tasks.push(task::spawn(async move {
            let mut rectangles: Vec<(Rect, VecN<f64, 4>)> = Vec::new();
            let mut y = 0;
            while y < rows && y + pixel_size <= rows {
                let mut x = 0;
                while x < cols && x + pixel_size <= cols {
                    // Calculate average color within the circle's region
                    let rect = Rect::new(x, y, pixel_size, pixel_size);

                    // Get the sub-matrix of the input frame
                    let sub_mat = Mat::roi(&input_clone, rect)?;
                    let avg_color = core::mean(&sub_mat, &core::no_array())?;

                    rectangles.push((rect, avg_color));

                    x += pixel_size + spacing;
                }
                y += pixel_size + spacing;
            }
            Ok::<Vec<(Rect, VecN<f64, 4>)>, anyhow::Error>(rectangles)
        }));
        chunk_y += 1;
    }

    // Wait for all async tasks to complete and process the results
    for task in tasks {
        let rectangles = task.await??;
        for (rect, avg_color) in rectangles {
            // Draw the pixelated rectangle on the output frame
            imgproc::rectangle(output, rect, avg_color, -1, imgproc::LINE_8, 0)?;
        }
    }

    Ok(())
}

pub fn highlight_objects_with_contours(input_frame: &Mat, output_frame: &mut Mat) -> Result<()> {
    let mut edges = Mat::default();

    // Apply the canny algorithm to detect edges (steps: grayscale, blur, canny)
    let mut gray = Mat::default();
    imgproc::cvt_color(input_frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;
    let mut blurred = Mat::default();
    imgproc::gaussian_blur(
        &gray,
        &mut blurred,
        core::Size::new(5, 5),
        0.0,
        0.0,
        core::BORDER_DEFAULT,
    )?;
    imgproc::canny(&blurred, &mut edges, 120.0, 255.0, 3, false)?;

    // Find contours
    let mut contours = core::Vector::<core::Vector<core::Point>>::new();
    imgproc::find_contours(
        &edges,
        &mut contours,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        core::Point::new(0, 0),
    )?;

    for i in 0..contours.len() {
        let color = core::Scalar::new(0.0, 255.0, 0.0, 0.0); // Green color
        imgproc::draw_contours(
            output_frame,
            &contours,
            i as i32,
            color,
            2,
            imgproc::LINE_AA,
            &core::no_array(),
            0,
            core::Point::new(0, 0),
        )?;
    }

    // Select the largest contour
    if let Some(largest_contour) = contours
        .iter()
        .max_by_key(|contour| imgproc::contour_area(&contour, false).unwrap_or(0.0) as i32)
    {
        let mut approx = core::Vector::<core::Point>::new();
        let epsilon = 0.001 * imgproc::arc_length(&largest_contour, true)?; // Adjust epsilon for contour precision
        imgproc::approx_poly_dp(&largest_contour, &mut approx, epsilon, true)?;

        // Wrap the single contour in a Vector
        let approx_contours = core::Vector::<core::Vector<core::Point>>::from(vec![approx]);

        // Draw the single outline on the output frame
        imgproc::draw_contours(
            output_frame,
            &approx_contours,
            -1,
            core::Scalar::new(0.0, 0.0, 255.0, 0.0), // Red color
            2,
            imgproc::LINE_AA,
            &core::no_array(),
            0,
            core::Point::new(0, 0),
        )?;
    }
    Ok(())
}

pub fn detect_interference_near_point(
    input_frame: &Mat,
    object: Point,
    distance: i32,
) -> Result<bool> {
    // Convert the input frame to grayscale
    let mut gray = Mat::default();
    imgproc::cvt_color(input_frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    // Create a binary image by thresholding the grayscale image
    let mut mask = Mat::default();
    imgproc::threshold(&gray, &mut mask, 200.0, 255.0, imgproc::THRESH_BINARY)?;

    // Check if the object is within the interference region
    let mut y = object.y - distance;
    if y < 0 {
        y = 0;
    }
    while y < object.y + distance && y < mask.rows() {
        let mut x = object.x - distance;
        if x < 0 {
            x = 0;
        }
        while x < object.x + distance && x < mask.cols() {
            if *mask.at_2d::<u8>(y, x)? == 0 {
                return Ok(true);
            }
            x += 1;
        }
        y += 1;
    }
    Ok(false)
}
