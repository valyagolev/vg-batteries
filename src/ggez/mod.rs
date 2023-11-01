use std::sync::Arc;

use ggegui::{egui::Ui, Gui};
use ggez::{
    event::EventHandler,
    glam::Vec2,
    graphics::{Canvas, Color, DrawParam},
    input::mouse::set_cursor_hidden,
    Context, GameResult,
};

use self::{
    debug::get_debug_strings,
    graphics::{Graphics, GraphicsParams},
};

pub mod controllable;
pub mod debug;
pub mod graphics;

pub struct SimpleGame {
    pub gui: Gui,
    pub bg_color: Color,
    pub replace_cursor: bool,
    pub graphics_params: Arc<graphics::GraphicsParams>,
}

impl SimpleGame {
    pub fn calculate_graphic_params(ctx: &mut Context) -> GameResult<graphics::GraphicsParams> {
        let (w, h) = ctx.gfx.size();

        let scale = w.min(h) * 0.8 / 2.0;
        let offset = (w.min(h) - scale * 2.0) / 2.0;

        let pixel_width = 1.0 / scale;

        let grid_start = Vec2::new(w - offset - scale * 2.0, offset);
        let basis = grid_start + Vec2::new(scale, scale);

        let draw_param = DrawParam::default()
            .dest(basis)
            .scale(Vec2::new(scale, scale));

        Ok(GraphicsParams {
            basis,
            scale,
            pixel_width,
            draw_param,
        })
    }

    pub fn new(ctx: &mut Context, bg_color: Color, replace_cursor: bool) -> GameResult<Self> {
        if replace_cursor {
            set_cursor_hidden(ctx, true);
        }

        let graphics_params = Arc::new(SimpleGame::calculate_graphic_params(ctx)?);

        Ok(Self {
            gui: Gui::new(ctx),
            bg_color,
            replace_cursor,
            graphics_params,
        })
    }

    pub fn graphics<'a>(&self, ctx: &'a mut Context) -> Graphics<'a, false> {
        Graphics {
            canvas: (),
            ctx,
            params: self.graphics_params.clone(),
        }
    }

    pub fn graphics_canvas<'a>(
        &self,
        canvas: &'a mut Canvas,
        ctx: &'a mut Context,
    ) -> Graphics<'a, true> {
        Graphics {
            canvas,
            ctx,
            params: self.graphics_params.clone(),
        }
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        draw: impl FnOnce(&mut Graphics<true>) -> GameResult<()>,
    ) -> GameResult<()> {
        let mut canvas = Canvas::from_frame(ctx, self.bg_color);

        let mut graphics = self.graphics_canvas(&mut canvas, ctx);

        draw(&mut graphics)?;

        if self.replace_cursor {
            let mouse_pos = graphics.screen_to_real(graphics.ctx.mouse.position());

            graphics.draw_point_at(mouse_pos, Color::new(1.0, 1.0, 1.0, 0.1))?;
        }

        canvas.draw(&self.gui, DrawParam::default().dest(Vec2::ZERO));

        canvas.finish(ctx)?;

        Ok(())
    }

    pub fn egui(&self, ui: &mut Ui, ctx: &mut Context) -> GameResult<()> {
        let mouse_pos = ctx.mouse.position();
        let graphics = self.graphics(ctx);

        let real_pos = graphics.screen_to_real(Vec2::new(mouse_pos.x, mouse_pos.y));

        ui.label(format!("Mouse pos: {:.3} {:.3}", mouse_pos.x, mouse_pos.y));

        ui.label(format!("Real pos : {:.3} {:.3}", real_pos.x, real_pos.y));

        ui.label(format!("Fps: {:.0}", graphics.ctx.time.fps()));

        ui.separator();

        ui.label("Debug log:");

        for log in get_debug_strings() {
            ui.label(log);
        }

        Ok(())
    }
}
