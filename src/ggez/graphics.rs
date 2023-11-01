use crate::typelevel::{Condition, TruthType, _Condition};
use std::sync::Arc;

use ggez::{
    glam::Vec2,
    graphics::{self, Color, DrawParam, Drawable, Mesh},
    GameResult,
};

pub struct GraphicsParams {
    pub basis: Vec2,
    pub scale: f32,
    pub pixel_width: f32,
    pub draw_param: DrawParam,
}

pub struct Graphics<'a, const CANVAS: bool>
where
    _Condition<CANVAS, &'a mut ggez::graphics::Canvas, ()>: TruthType,
{
    pub canvas: Condition<CANVAS, &'a mut ggez::graphics::Canvas, ()>,
    pub ctx: &'a mut ggez::Context,
    pub params: Arc<GraphicsParams>,
}

impl<'a, const CANVAS: bool> Graphics<'a, CANVAS>
where
    _Condition<CANVAS, &'a mut ggez::graphics::Canvas, ()>: TruthType,
{
    pub fn screen_to_real<P: Into<Vec2>>(&self, v: P) -> Vec2 {
        (v.into() - self.params.basis) / self.params.scale
    }

    pub fn real_to_screen(&self, v: Vec2) -> Vec2 {
        self.params.basis - v * self.params.scale
    }
}

impl<'a> Graphics<'a, true> {
    pub fn draw(&mut self, d: &impl Drawable) -> GameResult {
        self.canvas.draw(d, self.params.draw_param);

        Ok(())
    }

    pub fn draw_point_at(&mut self, pos: Vec2, color: Color) -> GameResult {
        self.draw(&Mesh::new_circle(
            self.ctx,
            graphics::DrawMode::fill(),
            pos,
            self.params.pixel_width * 5.0,
            self.params.pixel_width * 5.0,
            color,
        )?)?;

        Ok(())
    }

    pub fn draw_line_at(&mut self, p1: Vec2, p2: Vec2, color: Color) -> GameResult {
        self.draw(&Mesh::new_line(
            self.ctx,
            &[p1, p2],
            self.params.pixel_width * 1.0,
            color,
        )?)?;

        Ok(())
    }
}
