use opencv::{
    core::Point,
    highgui::{self, WINDOW_NORMAL},
    prelude::*,
    Result,
};
use std::sync::{Arc, Mutex};

pub struct Window {
    name: String,
}

impl Window {
    pub fn new(name: &str, width: i32, height: i32) -> Result<Self> {
        highgui::named_window(&name, WINDOW_NORMAL)?;
        highgui::resize_window(&name, width, height)?;
        Ok(Self {
            name: name.to_string(),
        })
    }

    pub fn handle_mouse_events(&self, mouse_coords: Arc<Mutex<Point>>) -> Result<()> {
        let callback = Box::new(move |event: i32, x: i32, y: i32, _: i32| {
            if event == highgui::EVENT_MOUSEMOVE || event == highgui::EVENT_LBUTTONDOWN {
                let mut coords = mouse_coords.lock().unwrap();
                coords.x = x;
                coords.y = y;
            }
        });
        highgui::set_mouse_callback(&self.name, Some(callback))?;
        Ok(())
    }

    pub fn show(&self, frame: &Mat) -> Result<()> {
        highgui::imshow(&self.name, frame)?;
        Ok(())
    }
}
