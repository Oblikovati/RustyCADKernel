use super::Tools;
use crate::int_tools::{CommonPrt, Range};
use rustycad_tkernel::{precision, standard::Failure};
impl Tools {
    /// Tests strict proximity to either endpoint; `Tools::is_on_pave(0., &range, 1e-7)`.
    pub fn is_on_pave(parameter: f64, range: &Range, tolerance: f64) -> bool {
        (range.first() - parameter).abs() < tolerance
            || (range.last() - parameter).abs() < tolerance
    }
    /// Includes the interval and non-strict endpoint tolerance; `Tools::is_on_pave1(0., &range, 1e-7)`.
    pub fn is_on_pave1(parameter: f64, range: &Range, tolerance: f64) -> bool {
        if parameter >= range.first() && parameter <= range.last() {
            return true;
        }
        (parameter - range.first()).abs() <= tolerance
            || (parameter - range.last()).abs() <= tolerance
    }
    /// Tests only the candidate endpoints, preserving native asymmetric containment; `Tools::is_in_range(&reference, &range, 1e-7)`.
    pub fn is_in_range(reference: &Range, range: &Range, tolerance: f64) -> bool {
        let (first, last) = (reference.first() - tolerance, reference.last() + tolerance);
        (range.first() >= first && range.first() <= last)
            || (range.last() >= first && range.last() <= last)
    }
    /// Chooses the stored vertex parameter only when it lies inside range1; `Tools::vertex_parameter(&part, &mut t)`.
    pub fn vertex_parameter(part: &CommonPrt, parameter: &mut f64) {
        *parameter = parameter_in_range(part.range1(), part.vertex_parameter1());
    }
    /// Updates range1 before reading the mandatory first range2; `Tools::vertex_parameters(&part, &mut t1, &mut t2)?`.
    pub fn vertex_parameters(
        part: &CommonPrt,
        first: &mut f64,
        last: &mut f64,
    ) -> Result<(), Failure> {
        Self::vertex_parameter(part, first);
        let range = first_second_range(part)?;
        *last = parameter_in_range(range, part.vertex_parameter2());
        Ok(())
    }
    /// Computes the native angular intersection allowance; `Tools::compute_int_range(1e-7, 1e-7, 0.5)`.
    pub fn compute_int_range(first: f64, last: f64, angle: f64) -> f64 {
        let right = std::f64::consts::FRAC_PI_2;
        if (right - angle).abs() < precision::angular() {
            return last;
        }
        let acute = if angle > right {
            std::f64::consts::PI - angle
        } else {
            angle
        };
        let a = first * (right - acute).tan();
        let b = last / acute.sin();
        a + b
    }
}
fn parameter_in_range(range: &Range, parameter: f64) -> f64 {
    if parameter >= range.first() && parameter <= range.last() {
        return parameter;
    }
    0.5 * (range.first() + range.last())
}
pub(super) fn first_second_range(part: &CommonPrt) -> Result<&Range, Failure> {
    part.ranges2().first().ok_or_else(|| {
        Failure::with_type(
            "Standard_OutOfRange",
            "IntTools_CommonPrt::Ranges2 index 1 requires at least one range; length is 0",
            "",
        )
    })
}
