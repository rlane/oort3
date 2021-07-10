use crate::ship::ShipClass;
use crate::simulation::{Simulation, WORLD_SIZE};
use crate::webgl::WebGlRenderer;
use macroquad::color;
use nalgebra::{point, Point2, Rotation2, Translation2, Vector2};

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

        self.webgl
            .set_perspective(zoom, point![camera_target.x, camera_target.y]);

        let grid_size = 100.0;
        let n = 1 + (WORLD_SIZE as f32 / grid_size) as i32;
        for i in -(n / 2)..(n / 2 + 1) {
            self.webgl.draw_line(
                (i as f32) * grid_size,
                (-WORLD_SIZE as f32) / 2.0,
                (i as f32) * grid_size,
                (WORLD_SIZE as f32) / 2.0,
                1.0,
                color::GREEN,
            );
            self.webgl.draw_line(
                (-WORLD_SIZE as f32) / 2.0,
                (i as f32) * grid_size,
                (WORLD_SIZE as f32) / 2.0,
                (i as f32) * grid_size,
                1.0,
                color::GREEN,
            );
        }

        {
            let v = -WORLD_SIZE as f32 / 2.0;
            self.webgl.draw_line(-v, -v, v, -v, 1.0, color::RED);
            self.webgl.draw_line(-v, v, v, v, 1.0, color::RED);
            self.webgl.draw_line(-v, -v, -v, v, 1.0, color::RED);
            self.webgl.draw_line(v, -v, v, v, 1.0, color::RED);
        }

        for &index in sim.bullets.iter() {
            let bullet = sim.bullet(index);
            let body = bullet.body();
            let x = body.position().translation.x as f32;
            let y = body.position().translation.y as f32;
            let vx = body.linvel().x as f32;
            let vy = body.linvel().y as f32;
            let dt = 2.0 / 60.0;
            self.webgl
                .draw_line(x, y, x - vx * dt, y - vy * dt, 1.0, color::ORANGE);
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
        let new_vertices = vertices
            .iter()
            .map(|&v| translation.transform_point(&Point2::from(rotation.transform_vector(&v))))
            .collect::<Vec<_>>();
        let thickness = 2.0;
        let color = color::YELLOW;
        for i in 1..vertices.len() {
            self.webgl.draw_line(
                new_vertices[i - 1].x,
                new_vertices[i - 1].y,
                new_vertices[i].x,
                new_vertices[i].y,
                thickness,
                color,
            );
        }
        let i = vertices.len() - 1;
        self.webgl.draw_line(
            new_vertices[i].x,
            new_vertices[i].y,
            new_vertices[0].x,
            new_vertices[0].y,
            thickness,
            color,
        );
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
