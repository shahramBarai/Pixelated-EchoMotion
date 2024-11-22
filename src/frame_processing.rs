use anyhow::Result;
use opencv::{core, imgproc, prelude::*};

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

pub fn highlight_objects_with_contours(
    input_frame: &Mat,
    output_frame: &mut Mat,
    filter_option: i32,
    lower_threshold: f64,
    upper_threshold: f64,
) -> Result<()> {
    let mut edges = Mat::default();

    if filter_option == 0 {
        threshold_based_edge_detection(input_frame, &mut edges, lower_threshold)?;
    } else if filter_option == 1 {
        canny_operator(input_frame, &mut edges, lower_threshold, upper_threshold)?;
    } else if filter_option == 2 {
        scharr_operator(input_frame, &mut edges, lower_threshold, upper_threshold)?;
    }

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

fn scharr_operator(
    frame: &Mat,
    edges: &mut Mat,
    lower_threshold: f64,
    upper_threshold: f64,
) -> Result<()> {
    let mut gray = Mat::default();
    imgproc::cvt_color(frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    let mut blurred = Mat::default();
    imgproc::gaussian_blur(
        &gray,
        &mut blurred,
        core::Size::new(5, 5),
        0.0,
        0.0,
        core::BORDER_DEFAULT,
    )?;

    let mut thresholded = Mat::default();
    imgproc::threshold(
        &blurred,
        &mut thresholded,
        lower_threshold,
        upper_threshold,
        imgproc::THRESH_BINARY,
    )?;

    let mut grad_x = Mat::default();
    let mut grad_y = Mat::default();
    let mut abs_grad_x = Mat::default();
    let mut abs_grad_y = Mat::default();

    imgproc::scharr(
        &thresholded,
        &mut grad_x,
        core::CV_16S,
        1,
        0,
        1.0,
        0.0,
        core::BORDER_DEFAULT,
    )?;
    imgproc::scharr(
        &thresholded,
        &mut grad_y,
        core::CV_16S,
        0,
        1,
        1.0,
        0.0,
        core::BORDER_DEFAULT,
    )?;

    core::convert_scale_abs(&grad_x, &mut abs_grad_x, 1.0, 0.0)?;
    core::convert_scale_abs(&grad_y, &mut abs_grad_y, 1.0, 0.0)?;

    core::add_weighted(&abs_grad_x, 0.5, &abs_grad_y, 0.5, 0.0, edges, core::CV_8U)?;

    Ok(())
}

fn canny_operator(
    frame: &Mat,
    edges: &mut Mat,
    lower_threshold: f64,
    upper_threshold: f64,
) -> Result<()> {
    let mut gray = Mat::default();
    imgproc::cvt_color(frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    let mut blurred = Mat::default();
    imgproc::gaussian_blur(
        &gray,
        &mut blurred,
        core::Size::new(5, 5),
        0.0,
        0.0,
        core::BORDER_DEFAULT,
    )?;

    imgproc::canny(&blurred, edges, lower_threshold, upper_threshold, 3, false)?;
    Ok(())
}

fn threshold_based_edge_detection(frame: &Mat, edges: &mut Mat, threshold: f64) -> Result<()> {
    let mut gray = Mat::default();
    imgproc::cvt_color(frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    let mut blurred = Mat::default();
    imgproc::gaussian_blur(
        &gray,
        &mut blurred,
        core::Size::new(5, 5),
        0.0,
        0.0,
        core::BORDER_DEFAULT,
    )?;

    imgproc::threshold(&blurred, edges, threshold, 255.0, imgproc::THRESH_BINARY)?;

    Ok(())
}
