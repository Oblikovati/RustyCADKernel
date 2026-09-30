use super::{Failure, Matrix3, Xyz, construction};
impl Matrix3 {
    /// Evaluates the original signed minor expansion; `matrix.determinant()`.
    pub fn determinant(&self) -> f64 {
        let [[a, b, c], [d, e, f], [g, h, i]] = self.coefficients;
        a * (e * i - h * f) - b * (d * i - g * f) + c * (d * h - g * e)
    }
    /// Uses gp resolution on the determinant; `matrix.is_singular()`.
    pub fn is_singular(&self) -> bool {
        self.determinant().abs() <= f64::MIN_POSITIVE
    }
    /// Uses OCCT's adjugate arithmetic; release OCCT permits singular inversion; `matrix.inverted()?`.
    pub fn inverted(self) -> Result<Self, Failure> {
        let adjugate = self.adjugate();
        let determinant = self.coefficients[0][0] * adjugate.coefficients[0][0]
            + self.coefficients[0][1] * adjugate.coefficients[1][0]
            + self.coefficients[0][2] * adjugate.coefficients[2][0];
        if cfg!(feature = "occt-debug-checks") && determinant.abs() <= f64::MIN_POSITIVE {
            return Err(construction(
                "gp_Mat::Invert() - matrix has zero determinant",
            ));
        }
        Ok(adjugate * (1.0 / determinant))
    }
    fn adjugate(self) -> Self {
        let [[a, b, c], [d, e, f], [g, h, i]] = self.coefficients;
        Self::new([
            [e * i - f * h, c * h - b * i, b * f - c * e],
            [f * g - d * i, a * i - c * g, c * d - a * f],
            [d * h - e * g, b * g - a * h, a * e - b * d],
        ])
    }
    /// Inverts in place, retaining coefficients on checked failure; `matrix.invert()?`.
    pub fn invert(&mut self) -> Result<(), Failure> {
        *self = self.inverted()?;
        Ok(())
    }
    /// Uses the native normalized-axis Rodrigues expression; `matrix.set_rotation(axis,angle)?`.
    pub fn set_rotation(&mut self, axis: Xyz, angle: f64) -> Result<(), Failure> {
        let vector = if cfg!(feature = "occt-debug-checks") {
            axis.normalized()?
        } else {
            axis / axis.modulus()
        };
        self.coefficients = rotation_coefficients(vector, angle);
        Ok(())
    }
    /// Divides by multiplying the reciprocal, with header-level zero checks; `matrix.divided(2.0)?`.
    pub fn divided(self, scalar: f64) -> Result<Self, Failure> {
        if scalar.abs() <= f64::MIN_POSITIVE {
            return Err(construction("gp_Mat : Divide by 0"));
        }
        Ok(self * (1.0 / scalar))
    }
    /// Divides in place, preserving the matrix on failure; `matrix.divide(2.0)?`.
    pub fn divide(&mut self, scalar: f64) -> Result<(), Failure> {
        *self = self.divided(scalar)?;
        Ok(())
    }
    /// Applies repeated squaring starting at identity, as in gp_Mat::Power; `matrix.powered(3)?`.
    pub fn powered(self, exponent: i32) -> Result<Self, Failure> {
        if exponent == 1 {
            return Ok(self);
        }
        if exponent == -1 {
            return self.inverted();
        }
        let base = if exponent < 0 { self.inverted()? } else { self };
        Ok(binary_power(base, exponent.unsigned_abs()))
    }
    /// Raises this matrix to an integer power; `matrix.power(3)?`.
    pub fn power(&mut self, exponent: i32) -> Result<(), Failure> {
        *self = self.powered(exponent)?;
        Ok(())
    }
    /// Prepends another matrix; `matrix.pre_multiply(other)`.
    pub fn pre_multiply(&mut self, other: Self) {
        *self = other * *self;
    }
}
fn binary_power(mut base: Matrix3, mut exponent: u32) -> Matrix3 {
    let mut result = Matrix3::default();
    result.set_identity();
    while exponent > 0 {
        if exponent & 1 != 0 {
            result *= base;
        }
        base *= base;
        exponent >>= 1;
    }
    result
}
fn rotation_coefficients(vector: Xyz, angle: f64) -> [[f64; 3]; 3] {
    let [a, b, c] = *vector.coordinates();
    let cosine = angle.cos();
    let sine = angle.sin();
    let complement = 1.0 - cosine;
    let (aa, bb, cc, ab, ac, bc) = (a * a, b * b, c * c, a * b, a * c, b * c);
    [
        [
            1.0 + complement * (-(bb + cc)),
            complement * ab - sine * c,
            complement * ac + sine * b,
        ],
        [
            complement * ab + sine * c,
            1.0 + complement * (-(aa + cc)),
            complement * bc - sine * a,
        ],
        [
            complement * ac - sine * b,
            complement * bc + sine * a,
            1.0 + complement * (-(aa + bb)),
        ],
    ]
}
