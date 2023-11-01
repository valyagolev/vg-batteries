use ggegui::egui;
use ggez::{glam::Vec2, GameResult};

use crate::ggez::graphics::Graphics;

pub const UI_TOLERANCE: f32 = 0.02;
pub type ControlPath = usize;

pub trait Drawable {
    fn draw(&mut self, graphics: &mut Graphics<true>) -> GameResult;
}

pub trait Controllable: Drawable {
    fn closest_path(&self, p: Vec2) -> Option<(ControlPath, f32)>;
    fn update_path(&mut self, path: ControlPath, p: Vec2) -> bool;

    fn controls(
        &self,
        graphics: &mut Graphics<true>,
        close_path: Option<(ControlPath, f32)>,
    ) -> GameResult;

    fn egui(&mut self, _ui: &mut egui::Ui, close_path: Option<(ControlPath, f32)>) -> GameResult {
        Ok(())
    }
}
