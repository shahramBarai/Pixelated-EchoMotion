use anyhow::{Ok, Result};
use opencv::{
    core::{self, no_array, Point, Rect, VecN, Vector},
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
            Ok(rectangles)
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

pub struct FrameProcessor {
    masks: Vec<Mat>,
    contours: Vec<Vector<Point>>,
    grayscale_threshold: f64,
    pixel_size: i32,
    spacing: i32,
}

impl FrameProcessor {
    pub fn new(pixel_size: i32, spacing: i32, grayscale_threshold: f64) -> Self {
        Self {
            masks: Vec::new(),
            contours: Vec::<Vector<Point>>::new(),
            grayscale_threshold,
            pixel_size,
            spacing,
        }
    }

    pub fn init(&mut self, frames_amount: i32) {
        self.masks.clear();
        self.contours.clear();

        for _ in 0..frames_amount {
            self.masks.push(Mat::default());
            self.contours.push(Vector::<Point>::new());
        }
    }

    pub fn convert_to_grayscale(&mut self, frame: &Mat, index: usize) -> Result<()> {
        let mut gray = Mat::default();
        imgproc::cvt_color(frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;
        imgproc::threshold(
            &gray,
            &mut self.masks[index],
            self.grayscale_threshold,
            255.0,
            imgproc::THRESH_BINARY,
        )?;
        Ok(())
    }

    pub fn draw_mask(&self, output_frame: &mut Mat, index: usize) -> Result<()> {
        self.masks[index].copy_to(output_frame)?;
        Ok(())
    }

    pub fn find_object_contour(&mut self, index: usize) -> Result<()> {
        let mut inverted_mask = Mat::default();
        core::bitwise_not(&self.masks[index], &mut inverted_mask, &no_array())?;

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
            self.contours[index] = contours
                .iter()
                .max_by_key(|contour| imgproc::contour_area(&contour, false).unwrap_or(0.0) as i32)
                .unwrap();

            return Ok(());
        }
        self.contours[index].clear();
        Ok(())
    }

    pub fn draw_contours(&self, output_frame: &mut Mat) -> Result<()> {
        for contour in &self.contours {
            if !contour.is_empty() {
                imgproc::draw_contours(
                    output_frame,
                    &Vector::<Vector<Point>>::from(vec![contour.clone()]),
                    -1,
                    core::Scalar::new(0.0, 255.0, 0.0, 0.0),
                    2,
                    imgproc::LINE_AA,
                    &no_array(),
                    0,
                    Point::new(0, 0),
                )?;
            }
        }
        Ok(())
    }

    // Extract object points from the mask (using findNonZero)
    pub async fn extract_object(&self, index: usize) -> Result<Vec<Point>> {
        let mask = self.masks[index].clone(); // Clone the mask (Mat is ref-counted)

        let handle = tokio::task::spawn_blocking(move || -> Result<Vec<Point>> {
            let mut inverted_mask = Mat::default();
            core::bitwise_not(&mask, &mut inverted_mask, &core::no_array())?;

            let mut points = Vector::<Point>::new();
            core::find_non_zero(&inverted_mask, &mut points)?;

            // Convert Vector<Point> to Vec<Point>
            let object: Vec<Point> = points.iter().map(|p| p.clone()).collect();

            Ok(object)
        });

        let object = handle.await??;
        Ok(object)
    }

    // Find the two closest points between two contours
    // Divide the work into chunks based on the number of points in the first contour
    pub async fn find_closest_points(
        &self,
        index_1: usize,
        index_2: usize,
    ) -> Result<(Point, Point)> {
        let contour_1 = &self.contours[index_1];
        let contour_2 = &self.contours[index_2];

        if contour_1.is_empty() || contour_2.is_empty() {
            return Ok((Point::new(0, 0), Point::new(0, 0)));
        }

        // Clone data since we'll be moving them into tasks
        let contour_1_data = contour_1.clone();
        let contour_2_data = contour_2.clone();

        let num_cpus = num_cpus::get();
        // Determine chunk size for splitting contour_1
        let chunk_size = (contour_1_data.len() + num_cpus - 1) / num_cpus;

        let mut tasks = Vec::new();

        for chunk in contour_1_data.as_slice().chunks(chunk_size) {
            let chunk = chunk.to_vec();
            let contour_2_data = contour_2_data.clone();

            // Spawn a blocking task for each chunk
            let handle = tokio::task::spawn_blocking(move || {
                let mut local_min_distance = f64::MAX;
                let mut local_closest_point_1 = Point::new(0, 0);
                let mut local_closest_point_2 = Point::new(0, 0);

                for &point_1 in &chunk {
                    for point_2 in &contour_2_data {
                        let dx = (point_1.x - point_2.x) as f64;
                        let dy = (point_1.y - point_2.y) as f64;
                        let distance = (dx * dx + dy * dy).sqrt();

                        if distance < local_min_distance {
                            local_min_distance = distance;
                            local_closest_point_1 = point_1;
                            local_closest_point_2 = point_2;
                        }
                    }
                }
                Ok::<(f64, Point, Point)>((
                    local_min_distance,
                    local_closest_point_1,
                    local_closest_point_2,
                ))
            });

            tasks.push(handle);
        }

        // Combine results from all tasks
        let mut global_min_distance = f64::MAX;
        let mut global_closest_point_1 = Point::new(0, 0);
        let mut global_closest_point_2 = Point::new(0, 0);

        for t in tasks {
            let (dist, p1, p2) = t.await??;
            if dist < global_min_distance {
                global_min_distance = dist;
                global_closest_point_1 = p1;
                global_closest_point_2 = p2;
            }
        }

        Ok((global_closest_point_1, global_closest_point_2))
    }
}
