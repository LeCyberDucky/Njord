use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Vec3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3D {
    pub fn new<A: Into<f64>, B: Into<f64>, C: Into<f64>>(x: A, y: B, z: C) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    #[must_use]
    pub fn cross_product(&self, rhs: &Vec3D) -> Self {
        let x = self.y * rhs.z - self.z * rhs.y;
        let y = self.z * rhs.x - self.x * rhs.z;
        let z = self.x * rhs.y - self.y * rhs.x;

        Self::new(x, y, z)
    }

    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn near_zero(&self) -> bool {
        (self.x.abs() < Vec3D::THRESHOLD)
            && (self.y.abs() < Vec3D::THRESHOLD)
            && (self.z.abs() < Vec3D::THRESHOLD)
    }

    pub fn normalize(&mut self) {
        *self /= self.magnitude()
    }

    #[must_use]
    pub fn normalized(&self) -> Self {
        *self / self.magnitude()
    }

    pub const THRESHOLD: f64 = 1e-8;
}

impl<T: Into<f64>> Add<T> for Vec3D {
    type Output = Vec3D;

    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl Add<Vec3D> for Vec3D {
    type Output = Vec3D;

    fn add(self, rhs: Vec3D) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<Vec3D> for &Vec3D {
    type Output = Vec3D;

    fn add(self, rhs: Vec3D) -> Self::Output {
        // Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
        *self + rhs
    }
}

impl Add<Vec3D> for i32 {
    type Output = Vec3D;

    fn add(self, rhs: Vec3D) -> Self::Output {
        rhs + self
    }
}

impl Add<Vec3D> for f64 {
    type Output = Vec3D;

    fn add(self, rhs: Vec3D) -> Self::Output {
        rhs + self
    }
}

impl<T: Into<f64>> AddAssign<T> for Vec3D {
    fn add_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}

impl AddAssign<Vec3D> for Vec3D {
    fn add_assign(&mut self, rhs: Vec3D) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<T: Into<f64>> Sub<T> for Vec3D {
    type Output = Vec3D;

    fn sub(self, rhs: T) -> Self::Output {
        self.add(-rhs.into())
    }
}

impl Sub<Vec3D> for Vec3D {
    type Output = Vec3D;

    fn sub(self, rhs: Vec3D) -> Self::Output {
        self.add(-rhs)
    }
}

impl Sub<Vec3D> for &Vec3D {
    type Output = Vec3D;

    fn sub(self, rhs: Vec3D) -> Self::Output {
        self.add(-rhs)
    }
}

// Does subtracting a vector from a scalar actually make sense?
// Eh, this non-symmetric operator stuff has mushed my brain 🧠
impl Sub<Vec3D> for i32 {
    type Output = Vec3D;

    fn sub(self, rhs: Vec3D) -> Self::Output {
        -(rhs - self)
    }
}

impl Sub<Vec3D> for f64 {
    type Output = Vec3D;

    fn sub(self, rhs: Vec3D) -> Self::Output {
        -(rhs - self)
    }
}

impl<T: Into<f64>> SubAssign<T> for Vec3D {
    fn sub_assign(&mut self, rhs: T) {
        self.add_assign(-rhs.into())
    }
}

impl SubAssign<Vec3D> for Vec3D {
    fn sub_assign(&mut self, rhs: Vec3D) {
        self.add_assign(-rhs)
    }
}

impl<T: Into<f64>> Mul<T> for Vec3D {
    type Output = Vec3D;

    fn mul(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<T: Into<f64>> MulAssign<T> for Vec3D {
    fn mul_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Mul<&Vec3D> for Vec3D {
    type Output = f64;

    fn mul(self, rhs: &Vec3D) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

#[allow(clippy::op_ref)]
impl Mul<Vec3D> for Vec3D {
    type Output = f64;

    fn mul(self, rhs: Vec3D) -> Self::Output {
        self * &rhs
    }
}

#[allow(clippy::op_ref)]
impl Mul<Vec3D> for &Vec3D {
    type Output = f64;

    fn mul(self, rhs: Vec3D) -> Self::Output {
        *self * &rhs
    }
}

impl Mul<&Vec3D> for &Vec3D {
    type Output = f64;

    fn mul(self, rhs: &Vec3D) -> Self::Output {
        *self * rhs
    }
}

impl Mul<Vec3D> for i32 {
    type Output = Vec3D;

    fn mul(self, rhs: Vec3D) -> Self::Output {
        rhs * self
    }
}

impl Mul<Vec3D> for f64 {
    type Output = Vec3D;

    fn mul(self, rhs: Vec3D) -> Self::Output {
        rhs * self
    }
}

impl Mul<&Vec3D> for f64 {
    type Output = Vec3D;

    fn mul(self, rhs: &Vec3D) -> Self::Output {
        *rhs * self
    }
}

impl<T: Into<f64>> Div<T> for Vec3D {
    type Output = Vec3D;

    fn div(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl<T: Into<f64>> DivAssign<T> for Vec3D {
    fn div_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Neg for &Vec3D {
    type Output = Vec3D;

    fn neg(self) -> Self::Output {
        Self::Output::new(-self.x, -self.y, -self.z)
    }
}

impl Neg for Vec3D {
    type Output = Vec3D;

    fn neg(self) -> Self::Output {
        -&self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let a = Vec3D::new(1, 2, 3);
        let b = Vec3D::new(4, 5, 6);
        let c = Vec3D::new(5, 7, 9);
        let d = Vec3D::new(2, 3, 4);
        assert_eq!(a + b, c);
        assert_eq!(a + 1, d);
        assert_eq!(1 + a, d);
        assert_eq!(1.0 + a, d);
    }

    #[test]
    fn test_sub() {
        let a = Vec3D::new(1, 2, 3);
        let b = Vec3D::new(4, 5, 6);
        let c = Vec3D::new(-3, -3, -3);
        let d = Vec3D::new(0, 1, 2);
        assert_eq!(a - b, c);
        assert_eq!(a - 1, d);
        assert_eq!(-(1 - a), d);
        assert_eq!(-(1.0 - a), d);
    }

    #[test]
    fn test_add_assign() {
        let mut a = Vec3D::new(1, 2, 3);
        let b = Vec3D::new(4, 5, 6);
        let c = Vec3D::new(5, 7, 9);
        let d = Vec3D::new(6, 8, 10);
        a += b;
        assert_eq!(a, c);
        a += 1;
        assert_eq!(a, d);
    }

    #[test]
    fn test_sub_assign() {
        let mut a = Vec3D::new(1, 2, 3);
        let b = Vec3D::new(4, 5, 6);
        let c = Vec3D::new(-3, -3, -3);
        let d = Vec3D::new(-4, -4, -4);
        a -= b;
        assert_eq!(a, c);
        a -= 1;
        assert_eq!(a, d);
    }

    #[test]
    fn test_mul() {
        let a = Vec3D::new(1, 2, 3);
        let b = Vec3D::new(-16, 5, 2);
        assert_eq!(a * b, 0.0);
        assert_eq!(a * (-2), -a * 2);
        assert_eq!(-2 * a, -a * 2);
    }

    #[test]
    fn test_mul_assign() {
        let a = Vec3D::new(1, 2, 3);
        let mut b = a;
        b *= -2;
        assert_eq!(b, -a * 2);
    }

    #[test]
    fn test_div() {
        let a = Vec3D::new(1, 2, 3);
        let b = Vec3D::new(0.5, 1, 1.5);
        assert_eq!(a / 2, b);
    }

    #[test]
    fn test_div_assign() {
        let mut a = Vec3D::new(1, 2, 3);
        let b = Vec3D::new(0.5, 1, 1.5);
        a /= 2;
        assert_eq!(a, b);
    }

    #[test]
    fn test_neg() {
        let a = -Vec3D::new(1, 2, 3);
        let b = Vec3D::new(-1, -2, -3);
        assert_eq!(a, b);
    }

    #[test]
    fn test_magnitude() {
        assert_eq!(Vec3D::new(1, 2, 2).magnitude(), 3.0);
        assert_eq!(Vec3D::new(-2, -3, -6).magnitude(), 7.0);
        assert_eq!(Vec3D::new(1, -4, 8).magnitude(), 9.0);
    }

    #[test]
    fn test_cross_product() {
        let a = Vec3D::new(1, 2, 3);
        let b = Vec3D::new(4, 5, 6);
        let c = Vec3D::new(-12, 3, 2);

        assert_eq!(a.cross_product(&b), Vec3D::new(-3, 6, -3));
        assert_eq!(b.cross_product(&a), Vec3D::new(3, -6, 3));
        assert_eq!(a * c, 0.0); // a and c need to be perpendicular for the next tests
        assert_eq!(a.cross_product(&c) * a, 0.0);
        assert_eq!(a.cross_product(&c) * c, 0.0);
        assert_eq!(c.cross_product(&a) * a, 0.0);
        assert_eq!(c.cross_product(&a) * c, 0.0);
    }

    #[test]
    fn test_normalize() {
        let mut a = Vec3D::new(1, -1, 1);
        a.normalize();
        assert_eq!(a.magnitude(), 1.0);
        assert_eq!(
            a,
            Vec3D::new(
                1.0 / 3.0_f64.sqrt(),
                -1.0 / 3.0_f64.sqrt(),
                1.0 / 3.0_f64.sqrt()
            )
        );
    }
}
