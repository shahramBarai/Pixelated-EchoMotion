use std::sync::{Arc, Mutex};

use anyhow::Result;
use opencv::{
    core::{self, no_array, Point, Rect, Size, VecN, Vector},
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
    let num_cpus = num_cpus::get() as i32;
    let pixels_per_row = rows / pixel_size;
    let chunk_size = ((pixels_per_row + num_cpus - 1) / num_cpus) as i32 * pixel_size;

    // Vector to hold all async tasks
    let mut tasks = Vec::new();

    let mut i = 1;
    let mut chunk_start = 0;
    while chunk_start < rows {
        let input_clone = input.clone(); // Clone input matrix for each task
        let chunk_end = (chunk_start + chunk_size).min(rows);

        println!(
            "{}: Processing chunk: {} - {} ({})",
            i, chunk_start, chunk_end, chunk_size
        );
        i += 1;
        // Spawn async task for this chunk
        tasks.push(task::spawn(async move {
            let mut rectangles: Vec<(Rect, VecN<f64, 4>)> = Vec::new();
            let mut y = chunk_start;
            while y < chunk_end && y + pixel_size <= rows {
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
        chunk_start = chunk_end;
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

fn get_object_contour(input_frame: &Mat) -> Result<Vector<Vector<Point>>> {
    // Apply the canny algorithm to detect edges (steps: grayscale, blur, canny)
    let mut gray = Mat::default();
    imgproc::cvt_color(input_frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;
    let mut mask = Mat::default();
    imgproc::threshold(&gray, &mut mask, 200.0, 255.0, imgproc::THRESH_BINARY)?;
    let mut inverted_mask = Mat::default();
    core::bitwise_not(&mask, &mut inverted_mask, &no_array())?;

    // Find contours
    let mut contours = Vector::<Vector<Point>>::new();
    imgproc::find_contours(
        &inverted_mask,
        &mut contours,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        Point::new(0, 0),
    )?;

    if !contours.is_empty() {
        // Select the largest contour
        let largest_contour = contours
            .iter()
            .max_by_key(|contour| imgproc::contour_area(&contour, false).unwrap_or(0.0) as i32)
            .unwrap();

        return Ok(Vector::<Vector<Point>>::from(vec![largest_contour]));
    }

    return Ok(Vector::<Vector<Point>>::new());
}

fn get_two_closest_points(
    frame_1: &Mat,
    frame_2: &Mat,
    output: &mut Mat,
) -> Result<(Point, Point)> {
    // Get the object contour from the two frames
    let contour_1 = get_object_contour(frame_1)?;
    let contour_2 = get_object_contour(frame_2)?;

    if !contour_1.is_empty() {
        imgproc::draw_contours(
            output,
            &contour_1,
            -1,
            core::Scalar::new(0.0, 0.0, 0.0, 0.0),
            2,
            imgproc::LINE_AA,
            &no_array(),
            0,
            Point::new(0, 0),
        )?;
    }

    if !contour_2.is_empty() {
        imgproc::draw_contours(
            output,
            &contour_2,
            -1,
            core::Scalar::new(0.0, 0.0, 0.0, 0.0),
            2,
            imgproc::LINE_AA,
            &no_array(),
            0,
            Point::new(0, 0),
        )?;
    }

    // Initialize variables to find the closest points
    let mut min_distance = f64::MAX;
    let mut closest_point_1 = Point::new(0, 0);
    let mut closest_point_2 = Point::new(0, 0);

    // Compare all points from contours_1 with all points from contours_2
    for contour_1 in contour_1.iter() {
        for point_1 in contour_1.iter() {
            for contour_2 in contour_2.iter() {
                for point_2 in contour_2.iter() {
                    let dx = (point_1.x - point_2.x) as f64;
                    let dy = (point_1.y - point_2.y) as f64;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance < min_distance {
                        min_distance = distance;
                        closest_point_1 = point_1;
                        closest_point_2 = point_2;
                    }
                }
            }
        }
    }

    Ok((closest_point_1, closest_point_2))
}

pub fn detect_interference(
    frame_1: &Mat,
    frame_2: &Mat,
    output: &mut Mat,
    min_distance: i32,
) -> Result<bool> {
    let (point_1, point_2) = get_two_closest_points(frame_1, frame_2, output)?;

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

    println!("Distance: {}", distance);

    if distance < min_distance {
        imgproc::line(
            output,
            point_1,
            point_2,
            core::Scalar::new(0.0, 0.0, 255.0, 0.0),
            2,
            imgproc::LINE_AA,
            0,
        )?;
        return Ok(true);
    }
    imgproc::line(
        output,
        point_1,
        point_2,
        core::Scalar::new(255.0, 0.0, 0.0, 0.0),
        2,
        imgproc::LINE_AA,
        0,
    )?;

    Ok(false)
}
