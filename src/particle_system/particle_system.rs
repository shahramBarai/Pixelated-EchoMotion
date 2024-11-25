use anyhow::Result; // Automatically handle the error types
use opencv::{
    core::{self, Point, Rect, Scalar},
    imgproc,
    prelude::*,
};

pub struct Particle {
    window_size: Point,
    origin: Point,
    size: i32,
    color: core::Scalar,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    pub on_position: bool,
}

impl Particle {
    pub fn new(window_size: Point, origin: Point, size: i32, color: core::Scalar) -> Self {
        Particle {
            window_size,
            origin,
            size,
            color,
            x: origin.x as f64,
            y: origin.y as f64,
            vx: 0.0,
            vy: 0.0,
            on_position: true,
        }
    }

    pub fn draw(&self, frame: &mut Mat) -> Result<()> {
        imgproc::rectangle(
            frame,
            Rect::new(self.x as i32, self.y as i32, self.size, self.size),
            self.color,
            -1,
            imgproc::LINE_8,
            0,
        )?;
        Ok(())
    }

    pub fn update(&mut self, mouse_coords: Point, mouse_radius: f64) {
        // Influence by mouse
        let dx = mouse_coords.x as f64 - self.x;
        let dy = mouse_coords.y as f64 - self.y;
        let distance = dx * dx + dy * dy;
        let force;
        if distance == 0.0 {
            force = 0.0;
        } else {
            force = -mouse_radius / distance;
        }

        if distance < mouse_radius {
            let angel = dy.atan2(dx) as f64;
            self.vx += force * angel.cos() as f64;
            self.vy += force * angel.sin() as f64;
        }

        self.vx *= 0.80;
        self.vy *= 0.80;

        self.x += (self.origin.x as f64 - self.x) * 0.05 + self.vx;
        self.y += (self.origin.y as f64 - self.y) * 0.05 + self.vy;

        if (self.origin.x as f64 - self.x).abs() < 1.0
            && (self.origin.y as f64 - self.y).abs() < 1.0
        {
            self.x = self.origin.x as f64;
            self.y = self.origin.y as f64;
            self.on_position = true;
        } else {
            self.on_position = false;
        }

        if self.x < 0.0 {
            self.x = 0.0;
        }
        if self.y < 0.0 {
            self.y = 0.0;
        }
        if self.x > self.window_size.x as f64 {
            self.x = self.window_size.x as f64;
        }
        if self.y > self.window_size.y as f64 {
            self.y = self.window_size.y as f64;
        }
    }
}

pub struct Effect {
    pub window_size: Point,
    pub pixel_size: i32,
    pub pixel_spacing: i32,
    pub brightness_threshold: f64,
    particles: Vec<Particle>,
    pub mouse_coords: Point,
    mouse_radius: f64,
    animation: bool,
}

impl Effect {
    pub fn new(window_width: i32, window_height: i32, pixel_size: i32, pixel_spacing: i32) -> Self {
        Effect {
            window_size: Point::new(window_width, window_height),
            pixel_size,
            pixel_spacing,
            brightness_threshold: 200.0,
            particles: Vec::new(),
            mouse_coords: Point::new(0, 0),
            mouse_radius: 40000.0,
            animation: false,
        }
    }

    pub fn init(&mut self, input_frame: &Mat) -> Result<()> {
        // Clear the particles array
        self.particles.clear();

        // Convert the input frame to grayscale
        let mut gray = Mat::default();
        imgproc::cvt_color(input_frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

        // Create a binary image by thresholding the grayscale image
        let mut mask = Mat::default();
        imgproc::threshold(
            &gray,
            &mut mask,
            self.brightness_threshold,
            255.0,
            imgproc::THRESH_BINARY,
        )?;

        let mut y = 0;
        while y < mask.rows() {
            let mut x = 0;
            while x < mask.cols() {
                if *mask.at_2d::<u8>(y, x)? == 0 {
                    let color = input_frame.at_2d::<core::Vec3b>(y, x)?;
                    self.particles.push(Particle::new(
                        self.window_size,
                        Point::new(x, y),
                        self.pixel_size,
                        Scalar::new(color[0] as f64, color[1] as f64, color[2] as f64, 0.0),
                    ));
                }
                x += self.pixel_size + self.pixel_spacing;
            }
            y += self.pixel_size + self.pixel_spacing;
        }
        println!("Particles: {}", self.particles.len());
        self.animation = true;
        Ok(())
    }

    pub fn draw(&self, frame: &mut Mat) -> Result<()> {
        for particle in &self.particles {
            particle.draw(frame)?;
        }
        Ok(())
    }

    pub fn update(&mut self) {
        let mut all_on_position = true;
        for particle in &mut self.particles {
            particle.update(self.mouse_coords, self.mouse_radius);
            if all_on_position && !particle.on_position {
                all_on_position = false;
            }
        }
        self.animation = !all_on_position;
    }

    pub fn get_animation_status(&self) -> bool {
        self.animation
    }
}
