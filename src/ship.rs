use crate::index_set::{HasIndex, Index};
use crate::simulation::Simulation;
use rapier2d_f64::prelude::*;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct ShipHandle(pub Index);

impl HasIndex for ShipHandle {
    fn index(self) -> Index {
        self.0
    }
}

pub struct ShipAccessor<'a> {
    pub(crate) simulation: &'a Simulation,
    pub(crate) handle: ShipHandle,
}

impl<'a> ShipAccessor<'a> {
    pub fn body(&self) -> &'a RigidBody {
        self.simulation
            .bodies
            .get(RigidBodyHandle(self.handle.index()))
            .unwrap()
    }

    pub fn position(&self) -> Translation<Real> {
        self.body().position().translation
    }

    pub fn velocity(&self) -> Vector<Real> {
        *self.body().linvel()
    }

    pub fn heading(&self) -> Real {
        self.body().rotation().angle()
    }
}

pub struct ShipAccessorMut<'a> {
    pub(crate) simulation: &'a mut Simulation,
    pub(crate) handle: ShipHandle,
}

impl<'a: 'b, 'b> ShipAccessorMut<'a> {
    pub fn body(&'b mut self) -> &'b mut RigidBody {
        self.simulation
            .bodies
            .get_mut(RigidBodyHandle(self.handle.index()))
            .unwrap()
    }

    pub fn thrust_main(&mut self, force: f64) {
        let body = self.body();
        let rotation_matrix = body.position().rotation.to_rotation_matrix();
        body.apply_force(rotation_matrix * vector![force, 0.0], true);
    }

    pub fn thrust_lateral(&mut self, force: f64) {
        let body = self.body();
        let rotation_matrix = body.position().rotation.to_rotation_matrix();
        body.apply_force(rotation_matrix * vector![0.0, force], true);
    }

    pub fn thrust_angular(&mut self, torque: f64) {
        self.body().apply_torque(torque, true);
    }

    pub fn fire_weapon(&mut self) {
        let body = self.body();
        let x = body.position().translation.x;
        let y = body.position().translation.y;
        let speed = 1000.0;
        let v2 = body.position().rotation.into_inner() * speed;
        let v = body.linvel() + vector![v2.re, v2.im];
        self.simulation.add_bullet(x, y, v.x, v.y);
    }
}
