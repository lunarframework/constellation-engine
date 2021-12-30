use nalgebra::{Matrix4, Perspective3, Point3, UnitQuaternion, Vector3};

pub struct Camera {
    position: Point3<f32>,
    rotation: UnitQuaternion<f32>,
    projection: Perspective3<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3::<f32>::from([0.0, 0.0, 0.0]),
            projection: Perspective3::new(1.0, 3.14 / 6.0, 0.1, 100.0),
            rotation: UnitQuaternion::<f32>::identity(),
        }
    }
}

impl Camera {
    pub fn new(aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            position: Point3::<f32>::from([0.0, 0.0, 0.0]),
            projection: Perspective3::new(aspect, fovy, znear, zfar),
            rotation: UnitQuaternion::<f32>::identity(),
        }
    }

    pub fn aspect(&self) -> f32 {
        self.projection.aspect()
    }

    pub fn fovy(&self) -> f32 {
        self.projection.aspect()
    }

    pub fn znear(&self) -> f32 {
        self.projection.znear()
    }

    pub fn zfar(&self) -> f32 {
        self.projection.zfar()
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.projection.set_aspect(aspect);
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        self.projection.set_fovy(fovy);
    }

    pub fn set_clipping_planes(&mut self, near: f32, far: f32) {
        self.projection.set_znear_and_zfar(near, far);
    }

    pub fn proj_mat(&self) -> Matrix4<f32> {
        self.projection.clone().to_homogeneous()
    }

    pub fn view_mat(&self) -> Matrix4<f32> {
        let forward = self
            .rotation
            .transform_point(&Point3::<f32>::from([0.0, 0.0, 1.0]));
        let up = self.rotation.transform_vector(&Vector3::<f32>::y());
        Matrix4::look_at_lh(&self.position, &forward, &up)
    }

    pub fn proj_view_mat(&self) -> Matrix4<f32> {
        self.proj_mat() * self.view_mat()
    }
}
