use nalgebra::{Affine3, Matrix4, Perspective3, Point3, UnitQuaternion, Vector3};

pub struct Transform {
    /// Position of the entity
    pub position: Point3<f32>,
    /// Rotation of the entity
    pub rotation: UnitQuaternion<f32>,
    /// Scale of the entity
    pub scale: Vector3<f32>,
}

impl Transform {
    /// Computes the view matrix required to displace objects relative to
    /// the entity's position
    pub fn view_mat(&self) -> Matrix4<f32> {
        Affine3::from_matrix_unchecked(self.transform_mat())
            .inverse()
            .to_homogeneous()
    }

    pub fn transform_mat(&self) -> Matrix4<f32> {
        let mut matrix = Matrix4::<f32>::identity();
        matrix *= Matrix4::new_translation(&self.position.coords);
        matrix *= self.rotation.to_homogeneous();
        matrix *= Matrix4::new_nonuniform_scaling(&self.scale);
        matrix
    }
}

pub struct Camera {
    projection: Perspective3<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: Perspective3::new(1.0, 3.14 / 6.0, 0.1, 100.0),
        }
    }
}

impl Camera {
    pub fn new(aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            projection: Perspective3::new(aspect, fovy, znear, zfar),
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
}
