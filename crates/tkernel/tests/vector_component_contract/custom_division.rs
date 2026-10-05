use rustycad_tkernel::ncollection::VectorMagnitudeComponent;
use std::ops::Div;

/// No DivAssign: upstream Normalize assigns the result of / instead of invoking /=.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DivisionElement {
    pub value: f64,
    pub divisions: usize,
}
impl From<i8> for DivisionElement {
    fn from(value: i8) -> Self {
        Self {
            value: f64::from(value),
            divisions: 0,
        }
    }
}
impl Div for DivisionElement {
    type Output = Self;
    fn div(self, divisor: Self) -> Self {
        Self {
            value: self.value / divisor.value,
            divisions: self.divisions + 1,
        }
    }
}
impl VectorMagnitudeComponent for DivisionElement {
    fn vector_component_modulus<const N: usize>(components: [Self; N]) -> Self {
        let values: [f64; N] = components.map(|component| component.value);
        Self {
            value: f64::vector_component_modulus(values),
            divisions: 0,
        }
    }
}
