use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

pub const EPSILON: f32 = 0.0001;

#[derive(Debug, Default, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const BLACK: Self = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const WHITE: Self = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
}

impl std::ops::Mul<f32> for Color {
    type Output = Color;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}
impl std::ops::Add<Color> for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub metalness: f32,
}

pub trait Renderable {
    fn intersect(&self, ray: Ray) -> Option<(f32, Vec3, Material)>;
    fn to_homogeneous(&mut self, view_mat: Mat4);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Camera {
    pub pos: Vec3,
    pub dir: Vec3,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Ray {
    pub pos: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn normalize(&mut self) {
        self.dir = self.dir.normalize();
    }
    pub fn reposition(mut self, t: f32) -> Self {
        self.normalize();
        self.pos = self.pos + self.dir * t;
        self
    }

    pub fn mirror(mut self, normal: Vec3) -> Self {
        self.normalize();
        let normal = normal.normalize();
        //self.dir = -self.dir;
        self.dir = self.dir - 2.0 * (self.dir.dot(normal)) * normal;
        self
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Tri {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    pub material: Material,
}

impl Renderable for Tri {
    fn intersect(&self, mut ray: Ray) -> Option<(f32, Vec3, Material)> {
        ray.dir = ray.dir.normalize();
        let edge1 = self.b - self.a;
        let edge2 = self.c - self.a;

        let h = ray.dir.cross(edge2);
        let a = edge1.dot(h);

        if a > -EPSILON && a < EPSILON {
            return None;
        }

        let f = 1.0 / a;
        let s = ray.pos - self.a;
        let u = f * s.dot(h);

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * ray.dir.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);

        if t > EPSILON {
            return Some((t, edge1.cross(edge2), self.material));
        }

        None
    }

    fn to_homogeneous(&mut self, view_mat: Mat4) {
        self.a = (view_mat * Vec4::from((self.a, 1.0))).xyz();
        self.b = (view_mat * Vec4::from((self.b, 1.0))).xyz();
        self.c = (view_mat * Vec4::from((self.c, 1.0))).xyz();
    }
}

pub struct Sphere {
    pub pos: Vec3,
    pub rad: f32,
    pub material: Material,
}

impl Renderable for Sphere {
    fn intersect(&self, mut ray: Ray) -> Option<(f32, Vec3, Material)> {
        ray.dir = ray.dir.normalize();
        let l_vec = self.pos - ray.pos;
        let l_l = l_vec.length();
        let tc = l_vec.dot(ray.dir);

        if tc < 0.0 {
            return None;
        }

        let d2 = ((tc * tc) - (l_l * l_l)).abs();

        let rad2 = self.rad * self.rad;
        if d2 > rad2 {
            return None;
        }

        let t1c = (rad2 - d2).sqrt();

        let t = tc - t1c;

        let p = ray.pos + ray.dir * t;

        Some((t, p - self.pos, self.material))
    }

    fn to_homogeneous(&mut self, view_mat: Mat4) {
        self.pos = (view_mat * Vec4::from((self.pos, 1.0))).xyz();
    }
}

pub struct Plane {
    pub pos: Vec3,
    pub norm: Vec3,
    pub material: Material,
}

impl Renderable for Plane {
    fn intersect(&self, ray: Ray) -> Option<(f32, Vec3, Material)> {
        let denom = self.norm.dot(ray.dir);
        if denom.abs() > EPSILON {
            let t = (self.pos - ray.pos).dot(self.norm) / denom;
            if t >= 0.0 {
                return Some((t - EPSILON, self.norm, self.material));
            }
        }
        None
    }
    fn to_homogeneous(&mut self, view_mat: Mat4) {
        self.pos = (view_mat * Vec4::from((self.pos, 1.0))).xyz();
    }
}

pub fn random_vec(min: f32, max: f32) -> Vec3 {
    let diff = max - min;
    Vec3 {
        x: (rand::random::<f32>() * diff) + min,
        y: (rand::random::<f32>() * diff) + min,
        z: (rand::random::<f32>() * diff) + min,
    }
}

pub fn random_vec_in_hemisphere(_normal: Vec3) -> Vec3 {
    loop {
        let v = random_vec(-1.0, 1.0);
        if v.length_squared() >= 1.0 {
            continue;
        }
        return v.normalize();
    }
}

#[cfg(test)]
mod test {
    use glam::Vec3;

    use super::Ray;

    #[test]
    fn ray_mirroring() {
        let mut ray = Ray {
            pos: Vec3::new(-3.0, 3.0, 0.0),
            dir: Vec3::new(1.0, -1.0, 0.0),
        };
        ray.normalize();
        let normal = Vec3::new(0.0, 1.0, 0.0);

        assert_eq!(
            ray.mirror(normal),
            Ray {
                pos: Vec3::new(-3.0, 3.0, 0.0),
                dir: Vec3::new(1.0, -1.0, 0.0),
            }
        );
    }
}
