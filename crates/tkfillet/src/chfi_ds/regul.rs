// Rust translation of ChFiDS_Regul.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A regularity to code on an edge: the curve and the two supports it joins.

/// The indices of a curve and of its two supports, where a support index is negative
/// when the support is a surface instead of a face (`ChFiDS_Regul`).
///
/// Natively a support set with `IsFace == false` is stored negated and `IsSurface1`
/// reports `is1 < 0`, so the name reads "is a surface (not a face)".
///
/// ```
/// use rustycad_tkfillet::chfi_ds::Regul;
/// let mut regul = Regul::new();
/// regul.set_s1(-3, false);
/// assert!(regul.is_surface1());
/// assert_eq!(regul.s1(), 3);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Regul {
    curve: i32,
    support1: i32,
    support2: i32,
}

impl Regul {
    /// All indices zero; `Regul::new().curve() == 0`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetCurve(IC)`: stores `|IC|`; `regul.set_curve(-2)`.
    pub fn set_curve(&mut self, curve: i32) {
        self.curve = curve.wrapping_abs();
    }

    /// `SetS1(IS1, IsFace)`: `|IS1|` for a face, `-|IS1|` for a surface; `regul.set_s1(1, true)`.
    pub fn set_s1(&mut self, support: i32, is_face: bool) {
        self.support1 = signed_support(support, is_face);
    }

    /// `SetS2(IS2, IsFace)`: `|IS2|` for a face, `-|IS2|` for a surface; `regul.set_s2(1, true)`.
    pub fn set_s2(&mut self, support: i32, is_face: bool) {
        self.support2 = signed_support(support, is_face);
    }

    /// `IsSurface1()`: whether the first support was set as a surface; `regul.is_surface1()`.
    pub fn is_surface1(&self) -> bool {
        self.support1 < 0
    }

    /// `IsSurface2()`: whether the second support was set as a surface; `regul.is_surface2()`.
    pub fn is_surface2(&self) -> bool {
        self.support2 < 0
    }

    /// `Curve()`: the curve index; `regul.curve()`.
    pub fn curve(&self) -> i32 {
        self.curve
    }

    /// `S1()`: the absolute first support index; `regul.s1()`.
    pub fn s1(&self) -> i32 {
        self.support1.wrapping_abs()
    }

    /// `S2()`: the absolute second support index; `regul.s2()`.
    pub fn s2(&self) -> i32 {
        self.support2.wrapping_abs()
    }
}

/// The stored support: positive for a face, negative for a surface. `std::abs(INT_MIN)` is
/// undefined natively; the port wraps it to `i32::MIN`, as the two's complement hardware does.
fn signed_support(support: i32, is_face: bool) -> i32 {
    let magnitude = support.wrapping_abs();
    if is_face {
        magnitude
    } else {
        magnitude.wrapping_neg()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_index_is_stored_as_an_absolute_value() {
        let mut regul = Regul::new();
        regul.set_curve(-7);
        assert_eq!(regul.curve(), 7);
    }

    #[test]
    fn supports_keep_the_face_flag_in_the_sign() {
        let mut regul = Regul::new();
        regul.set_s1(-4, true);
        regul.set_s2(5, false);
        assert!(!regul.is_surface1());
        assert!(regul.is_surface2());
        assert_eq!((regul.s1(), regul.s2()), (4, 5));
    }

    #[test]
    fn zero_support_is_never_a_surface() {
        let mut regul = Regul::new();
        regul.set_s1(0, false);
        assert!(!regul.is_surface1());
    }
}
