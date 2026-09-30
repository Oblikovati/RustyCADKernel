//! The six gp_Vec::SetLinearForm overloads, each forwarding to the gp_XYZ form of the same
//! arity so the summation order (and therefore rounding) is gp_XYZ's.
use super::super::{Vector3, Xyz};
impl Vector3 {
    /// Sets `a*first + b*second + c*third + offset`;
    /// `vector.set_linear_form_three_offset([1.0,2.0,3.0],[a,b,c],d)`.
    pub fn set_linear_form_three_offset(
        &mut self,
        weights: [f64; 3],
        terms: [Self; 3],
        offset: Self,
    ) {
        self.coordinates
            .set_linear_form_three_offset(weights, terms.map(xyz), offset.coordinates);
    }
    /// Sets `a*first + b*second + c*third`;
    /// `vector.set_linear_form_three([1.0,2.0,3.0],[a,b,c])`.
    pub fn set_linear_form_three(&mut self, weights: [f64; 3], terms: [Self; 3]) {
        self.coordinates
            .set_linear_form_three(weights, terms.map(xyz));
    }
    /// Sets `a*first + b*second + offset`; `vector.set_linear_form_offset(2.0,a,3.0,b,c)`.
    pub fn set_linear_form_offset(
        &mut self,
        a: f64,
        first: Self,
        b: f64,
        second: Self,
        offset: Self,
    ) {
        let (first, second, offset) = (first.coordinates, second.coordinates, offset.coordinates);
        self.coordinates
            .set_linear_form_offset(a, first, b, second, offset);
    }
    /// Sets two weighted vector terms; `vector.set_linear_form(2.0,first,3.0,second)`.
    pub fn set_linear_form(&mut self, a: f64, first: Self, b: f64, second: Self) {
        self.coordinates
            .set_linear_form(a, first.coordinates, b, second.coordinates);
    }
    /// Sets `a*first + second`; `vector.set_scaled_sum(2.0,a,b)`.
    pub fn set_scaled_sum(&mut self, a: f64, first: Self, second: Self) {
        self.coordinates
            .set_scaled_sum(a, first.coordinates, second.coordinates);
    }
    /// Sets `first + second`; `vector.set_sum(a,b)`.
    pub fn set_sum(&mut self, first: Self, second: Self) {
        self.coordinates
            .set_sum(first.coordinates, second.coordinates);
    }
}
fn xyz(vector: Vector3) -> Xyz {
    vector.coordinates
}
