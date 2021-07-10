use crate::ship::ShipClass;
use crate::simulation::{Simulation, WORLD_SIZE};
use crate::webgl::WebGlRenderer;
use nalgebra::{point, vector, Point2, Rotation2, Translation2, Vector2};

pub struct Renderer {
    webgl: WebGlRenderer,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            webgl: WebGlRenderer::new().expect("Failed to create WebGlRenderer"),
        }
    }

    pub fn render(&mut self, camera_target: Point2<f32>, zoom: f32, sim: &Simulation) {
        self.webgl.clear();
        self.webgl.update_viewport();
        self.webgl
            .set_perspective(zoom, point![camera_target.x, camera_target.y]);

        self.webgl.draw_grid(100.0, vector![0.0, 1.0, 0.0, 1.0]);

        {
            let v = -WORLD_SIZE as f32 / 2.0;
            let red = vector![1.0, 0.0, 0.0, 1.0];
            self.webgl.draw_line(-v, -v, v, -v, 1.0, red);
            self.webgl.draw_line(-v, v, v, v, 1.0, red);
            self.webgl.draw_line(-v, -v, -v, v, 1.0, red);
            self.webgl.draw_line(v, -v, v, v, 1.0, red);
        }

        for &index in sim.bullets.iter() {
            let bullet = sim.bullet(index);
            let body = bullet.body();
            let x = body.position().translation.x as f32;
            let y = body.position().translation.y as f32;
            let vx = body.linvel().x as f32;
            let vy = body.linvel().y as f32;
            let dt = 2.0 / 60.0;
            let orange = vector![1.00, 0.63, 0.00, 1.00];
            self.webgl
                .draw_line(x, y, x - vx * dt, y - vy * dt, 1.0, orange);
        }

        for &index in sim.ships.iter() {
            let ship = sim.ship(index);
            let x = ship.position().x as f32;
            let y = ship.position().y as f32;
            let h = ship.heading() as f32;
            let translation = Translation2::new(x, y);
            let rotation = Rotation2::new(h);

            match ship.data().class {
                ShipClass::Fighter => self.draw_model(&crate::model::ship(), translation, rotation),
                ShipClass::Asteroid => self.draw_model(
                    &crate::model::asteroid(ship.data().model_variant),
                    translation,
                    rotation,
                ),
            }
        }
    }

    fn draw_model(
        &mut self,
        vertices: &[Vector2<f32>],
        translation: Translation2<f32>,
        rotation: Rotation2<f32>,
    ) {
        let thickness = 2.0;
        let color = vector![0.99, 0.98, 0.00, 1.00];
        self.webgl
            .draw_line_loop(vertices, translation, rotation, thickness, color);
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
