use anyhow::Result;
use opencv::{
    core::{self, Point, Scalar},
    imgproc,
    prelude::*,
};

pub fn pixelate_frame(
    input: &Mat,
    output: &mut Mat,
    pixel_size: i32,
    spacing: i32,
    adjust_size_based_on_brightness: bool,
) -> Result<()> {
    let mut y = 0;
    while y <= input.rows() {
        let mut x = 0;
        while x <= input.cols() {
            if x + pixel_size <= input.cols() && y + pixel_size <= input.rows() {
                // Calculate average color within the circle's region
                let mut rect = core::Rect::new(x, y, pixel_size, pixel_size);

                // Get the sub-matrix of the input frame
                let sub_mat = Mat::roi(input, rect)?;
                let avg_color = core::mean(&sub_mat, &core::no_array())?;

                if adjust_size_based_on_brightness {
                    // Map brightness to size
                    let brightness = (avg_color[0] + avg_color[1] + avg_color[2]) as i32 / 3;
                    let mapped_size = 1 + ((255 - brightness) * (pixel_size - 1)) / 255;
                    let center = mapped_size / 2;
                    rect.x = x - center;
                    rect.y = y - center;
                    rect.width = mapped_size;
                    rect.height = mapped_size;
                }

                // Draw filled rectangle with the average color
                imgproc::rectangle(output, rect, avg_color, -1, imgproc::LINE_AA, 0)?;
            }
            x += pixel_size + spacing;
        }
        y += pixel_size + spacing;
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
