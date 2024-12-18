use rayon::prelude::*;
use std::sync::Arc;

use anyhow::Result; // Automatically handle the error types
use opencv::{
    core::{self, Point, Rect, Scalar, Size},
    imgproc,
    prelude::*,
};

use rand::Rng;

#[derive(Clone, Copy)]
// Enum to represent different effects
pub enum EffectType {
    Push,      // Existing push-around-mouse effect
    Break,     // Particles fall down
    Explosion, // Particles explode away from a point
}

struct Particle {
    window_size: Size,
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
    pub fn new(window_size: Size, origin: Point, size: i32, color: core::Scalar) -> Self {
        Particle {
            window_size,
            origin,
            size,
            color,
            x: origin.x as f64,
            y: origin.y as f64,
            vx: 0.0,
            vy: 0.0,
            on_position: false,
        }
    }

    pub fn update_with_effect(
        &mut self,
        effect_type: &EffectType,
        mouse_coords: Point,
        interference_distance: f64,
    ) {
        match effect_type {
            EffectType::Push => self.update_push(mouse_coords, interference_distance),
            EffectType::Break => self.update_break(),
            EffectType::Explosion => self.update_explosion(mouse_coords),
        }

        // Fade color
        self.fade_color(0.98);
    }

    fn fade_color(&mut self, factor: f64) {
        self.color = Scalar::new(
            self.color[0] * factor,
            self.color[1] * factor,
            self.color[2] * factor,
            self.color[3],
        );
    }

    // Update the particle with the push effect based on the given point
    fn update_push(&mut self, point: Point, interference_distance: f64) {
        // Influence by mouse
        let dx = point.x as f64 - self.x;
        let dy = point.y as f64 - self.y;
        let distance = dx * dx + dy * dy;
        let force = if distance == 0.0 {
            0.0
        } else {
            -interference_distance / distance
        };

        if distance < interference_distance {
            let angle = dy.atan2(dx); // Corrected variable name
            self.vx += force * angle.cos();
            self.vy += force * angle.sin();
        }

        // Apply friction
        let friction = 0.80;
        self.vx *= friction;
        self.vy *= friction;

        self.check_world_boundaries();
        self.move_towards_origin();
    }

    fn update_break(&mut self) {
        self.vy += 0.5; // Simulate gravity by incrementing vertical velocity

        // Introduce slight horizontal randomness
        let mut rng = rand::thread_rng();
        let horizontal_force: f64 = rng.gen_range(-0.5..0.5);
        self.vx += horizontal_force;

        // Apply damping to both velocities
        self.vx *= 0.98;
        self.vy *= 0.98;
        self.x += self.vx;
        self.y += self.vy;

        if self.y >= (self.window_size.height as f64 - 20.0) {
            self.y = self.window_size.height as f64 - 20.0; // Stop particles at the bottom
            self.vy = 0.0;
        }
    }

    fn update_explosion(&mut self, explosion_center: Point) {
        let dx = self.x - explosion_center.x as f64;
        let dy = self.y - explosion_center.y as f64;
        let distance = (dx * dx + dy * dy).sqrt().max(1.0); // Avoid division by zero

        // Base force and randomness
        let base_force = 500.0 / distance;
        let mut rng = rand::thread_rng();
        let random_factor: f64 = rng.gen_range(0.8..1.2); // Random force scaling
        let random_angle: f64 = rng.gen_range(-0.1..0.1); // Random angle variation

        let adjusted_force = base_force * random_factor;

        // Apply randomized direction
        let angle = dy.atan2(dx) + random_angle;
        self.vx += adjusted_force * angle.cos();
        self.vy += adjusted_force * angle.sin();

        // Cap velocity to prevent excessive speeds
        let max_velocity = 20.0; // Maximum velocity
        let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        if speed > max_velocity {
            let scale = max_velocity / speed;
            self.vx *= scale;
            self.vy *= scale;
        }

        // Apply damping
        self.vx *= 0.90; // Reduced damping for faster movement
        self.vy *= 0.90; // Reduced damping

        // Update positions
        self.x += self.vx;
        self.y += self.vy;

        self.check_world_boundaries();
    }

    fn move_towards_origin(&mut self) {
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
    }

    fn check_world_boundaries(&mut self) {
        if self.x < 0.0 {
            self.x = 0.0;
        }
        if self.y < 0.0 {
            self.y = 0.0;
        }
        if self.x > self.window_size.width as f64 {
            self.x = self.window_size.width as f64;
        }
        if self.y > self.window_size.height as f64 {
            self.y = self.window_size.height as f64;
        }
    }
}

pub struct ParticleSystem {
    window_size: Size,
    particle_system: Vec<Vec<Particle>>,
    pixel_size: i32,
    pixel_spacing: i32,
    animation_statuses: Vec<bool>,
    interference_distance: f64,
    effect_types: Vec<EffectType>,
    pub output_frame: Mat,
}

impl ParticleSystem {
    pub fn new(
        window_size: Size,
        pixel_size: i32,
        pixel_spacing: i32,
        interference_distance: i32,
    ) -> Self {
        ParticleSystem {
            window_size,
            particle_system: Vec::new(),
            pixel_size,
            pixel_spacing,
            animation_statuses: Vec::new(),
            interference_distance: interference_distance as f64,
            effect_types: Vec::new(),
            output_frame: Mat::default(),
        }
    }

    pub fn init(&mut self, frame: &Mat, amount: i32) -> Result<()> {
        self.particle_system.clear();
        self.animation_statuses.clear();
        self.effect_types.clear();

        for _ in 0..amount {
            self.particle_system.push(Vec::new());
            self.animation_statuses.push(false);
            self.effect_types.push(EffectType::Push);
        }

        self.output_frame = frame.clone();

        Ok(())
    }

    // Get the color of a pixel in the frame at the given point synchronously
    fn get_pixel_color_sync(frame: Arc<Mat>, point: &Point, _pixel_size: i32) -> Result<Scalar> {
        let color = frame.at_2d::<core::Vec3b>(point.y, point.x)?;
        Ok(Scalar::new(
            color[0] as f64,
            color[1] as f64,
            color[2] as f64,
            0.0,
        ))
    }

    pub async fn add_object(
        &mut self,
        frame: Arc<Mat>,
        object: &Vec<Point>,
        index: usize,
    ) -> Result<()> {
        if object.is_empty() {
            self.particle_system[index].clear();
            self.animation_statuses[index] = false;
            return Ok(());
        }

        let window_size = self.window_size;
        let pixel_size = self.pixel_size;
        let num_cpus = num_cpus::get();
        let chunk_size = (object.len() + num_cpus - 1) / num_cpus;
        let chunk_size = if chunk_size == 0 { 1 } else { chunk_size }; // Ensure chunk_size >= 1

        // Use a reference-counted pointer to the frame
        let frame_arc = Arc::clone(&frame);

        let mut tasks = Vec::new();

        for chunk in object.chunks(chunk_size) {
            let frame_clone = Arc::clone(&frame_arc);
            let chunk_data = chunk.to_vec();
            let pixel_size = pixel_size;
            let window_size = window_size;
            tasks.push(tokio::task::spawn_blocking(move || {
                let mut particles = Vec::with_capacity(chunk_data.len());
                for point in chunk_data {
                    let color = ParticleSystem::get_pixel_color_sync(
                        frame_clone.clone(),
                        &point,
                        pixel_size,
                    )?;
                    particles.push(Particle::new(window_size, point, pixel_size, color));
                }
                Ok::<Vec<Particle>, anyhow::Error>(particles)
            }));
        }

        // Wait for all tasks to complete and gather results
        let mut all_particles = Vec::with_capacity(object.len());
        for t in tasks {
            let mut partial = t.await??;
            all_particles.append(&mut partial);
        }

        self.particle_system[index] = all_particles;
        self.animation_statuses[index] = false;
        Ok(())
    }

    // Update the particle system with the given point
    pub async fn update(&mut self, point: Point) -> Result<()> {
        let effect_types = self.effect_types.clone();
        let interference_distance = self.interference_distance;
        let particle_count = self.particle_system.len();

        // Iterate over each particle group in parallel
        self.particle_system
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, particles)| {
                let effect_type = effect_types[i];
                for particle in particles.iter_mut() {
                    particle.update_with_effect(&effect_type, point, interference_distance);
                }
            });

        // Update animation statuses
        self.animation_statuses = self
            .particle_system
            .iter()
            .map(|particles| !particles.iter().all(|p| p.on_position))
            .collect();

        Ok(())
    }

    pub fn clean_output_frame(&mut self) -> Result<()> {
        self.output_frame
            .set_to(&core::Scalar::all(255.0), &core::no_array())?;
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        // Create a list of pixel to draw
        let mut pixels = Vec::new();
        let mut colors = Vec::new();

        for particles in &self.particle_system {
            for particle in particles {
                pixels.push(Rect::new(
                    particle.x as i32,
                    particle.y as i32,
                    particle.size,
                    particle.size,
                ));
                colors.push(particle.color);
            }
        }

        // Draw all pixel in a single loop
        for (pixel, color) in pixels.iter().zip(colors.iter()) {
            imgproc::rectangle(
                &mut self.output_frame,
                *pixel,
                *color,
                -1,
                imgproc::LINE_8,
                0,
            )?;
        }

        Ok(())
    }

    pub fn get_animation_status(&self, index: usize) -> Result<bool> {
        Ok(self.animation_statuses[index])
    }

    pub fn set_animation_status(&mut self, index: usize, status: bool) {
        self.animation_statuses[index] = status;
    }

    pub fn set_effect_type(&mut self, index: usize, effect_type: EffectType) {
        self.effect_types[index] = effect_type;
    }
}
