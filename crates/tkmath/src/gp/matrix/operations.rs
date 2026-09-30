use super::{Matrix3, Xyz};
use std::ops;
impl ops::Add for Matrix3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(std::array::from_fn(|r| {
            std::array::from_fn(|c| self.coefficients[r][c] + rhs.coefficients[r][c])
        }))
    }
}
impl ops::Sub for Matrix3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(std::array::from_fn(|r| {
            std::array::from_fn(|c| self.coefficients[r][c] - rhs.coefficients[r][c])
        }))
    }
}
impl ops::Mul for Matrix3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::new(std::array::from_fn(|r| {
            std::array::from_fn(|c| {
                self.coefficients[r][0] * rhs.coefficients[0][c]
                    + self.coefficients[r][1] * rhs.coefficients[1][c]
                    + self.coefficients[r][2] * rhs.coefficients[2][c]
            })
        }))
    }
}
impl ops::Mul<f64> for Matrix3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.coefficients.map(|row| row.map(|v| v * rhs)))
    }
}
impl ops::Mul<Matrix3> for f64 {
    type Output = Matrix3;
    fn mul(self, rhs: Matrix3) -> Matrix3 {
        rhs * self
    }
}
impl ops::Mul<Xyz> for Matrix3 {
    type Output = Xyz;
    fn mul(self, rhs: Xyz) -> Xyz {
        self.apply(rhs)
    }
}
impl ops::Mul<Matrix3> for Xyz {
    type Output = Xyz;
    fn mul(self, rhs: Matrix3) -> Xyz {
        rhs.apply(self)
    }
}
impl ops::AddAssign for Matrix3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl ops::SubAssign for Matrix3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl ops::MulAssign for Matrix3 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
impl ops::MulAssign<f64> for Matrix3 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}
impl ops::MulAssign<Matrix3> for Xyz {
    fn mul_assign(&mut self, rhs: Matrix3) {
        *self = rhs.apply(*self);
    }
}
