use anyhow::Result; // Automatically handle the error types
use opencv::{
    core::{self, Point, Rect, Scalar, Size},
    imgproc,
    prelude::*,
};

use rand::Rng;
use tokio::task::spawn_blocking;

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
    }

    fn update_push(&mut self, mouse_coords: Point, interference_distance: f64) {
        // Influence by mouse
        let dx = mouse_coords.x as f64 - self.x;
        let dy = mouse_coords.y as f64 - self.y;
        let distance = dx * dx + dy * dy;
        let force = if distance == 0.0 {
            0.0
        } else {
            -interference_distance / distance
        };

        if distance < interference_distance {
            let angel = dy.atan2(dx);
            self.vx += force * angel.cos();
            self.vy += force * angel.sin();
        }

        self.vx *= 0.80;
        self.vy *= 0.80;

        self.check_world_boundaries();
        self.move_towards_origin();
    }

    fn update_break(&mut self) {
        self.vy += 0.5; // Simulate gravity by incrementing vertical velocity
        self.vy *= 0.98; // Apply slight vertical damping
        self.y += self.vy;

        if self.y > self.window_size.height as f64 {
            self.y = self.window_size.height as f64; // Stop particles at the bottom
            self.vy = 0.0;
        }
    }

    fn update_explosion(&mut self, explosion_center: Point) {
        let dx = self.x - explosion_center.x as f64;
        let dy = self.y - explosion_center.y as f64;
        let distance = (dx * dx + dy * dy).sqrt().max(1.0); // Avoid division by zero
        let force = 10.0 / distance; // Arbitrary explosion force

        self.vx += force * dx / distance;
        self.vy += force * dy / distance;

        self.vx *= 0.98; // Apply damping
        self.vy *= 0.98; // Apply damping
        self.x += self.vx;
        self.y += self.vy;
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

    fn get_pixel_color(&self, frame: &Mat, point: &Point) -> Result<Scalar> {
        let color = frame.at_2d::<core::Vec3b>(point.y, point.x)?;
        Ok(Scalar::new(
            color[0] as f64,
            color[1] as f64,
            color[2] as f64,
            0.0,
        ))
    }

    pub fn add_object(&mut self, frame: &Mat, object: &Vec<Point>, index: usize) -> Result<()> {
        let mut particles = Vec::new();
        for point in object {
            particles.push(Particle::new(
                self.window_size,
                *point,
                self.pixel_size,
                self.get_pixel_color(frame, point)?,
            ));
        }
        self.particle_system[index] = particles;
        self.animation_statuses[index] = false;

        Ok(())
    }

    // Update the particle system with the given point
    pub async fn update(&mut self, point: Point) -> Result<()> {
        let effect_types = self.effect_types.clone();
        let interference_distance = self.interference_distance;
        let particle_count = self.particle_system.len();

        // Spawn a blocking task for each particle group
        let tasks: Vec<_> = (0..particle_count)
            .map(|i| {
                let effect_type = effect_types[i];
                // Extract the particle vector to avoid borrow conflicts
                let mut particles = std::mem::take(&mut self.particle_system[i]);
                spawn_blocking(move || {
                    let mut all_on_position = true;
                    for particle in &mut particles {
                        particle.update_with_effect(&effect_type, point, interference_distance);
                        if all_on_position && !particle.on_position {
                            all_on_position = false;
                        }
                    }
                    // Return the index, updated particles, and position status
                    Ok::<(usize, Vec<Particle>, bool), anyhow::Error>((
                        i,
                        particles,
                        all_on_position,
                    ))
                })
            })
            .collect();

        // Await all tasks and reinsert updated particle data
        for t in tasks {
            let (i, particles, all_on_position) = t.await??;
            self.particle_system[i] = particles;
            self.animation_statuses[i] = !all_on_position;
        }

        Ok(())
    }

    pub fn clean_output_frame(&mut self) -> Result<()> {
        self.output_frame
            .set_to(&core::Scalar::all(255.0), &core::no_array())?;
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        // Draw the particles
        for particles in &self.particle_system {
            for particle in particles {
                particle.draw(&mut self.output_frame)?;
            }
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
