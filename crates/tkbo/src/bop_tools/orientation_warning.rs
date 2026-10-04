use super::is_split_shape_to_reverse;
use crate::{bop_algo::AlertUnableToOrientTheShape, int_tools::Context};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape},
};
use rustycad_tkernel::{
    message::{AlertHandle, Gravity, Report},
    standard::Failure,
};
use std::sync::Arc;
/// Record orientation failure with both shapes in a warning compound; `is_split_to_reverse_with_warning(&split,&original,&context,Some(&report))?`.
pub fn is_split_to_reverse_with_warning(
    split: &Shape,
    original: &Shape,
    context: &Context,
    report: Option<&Report>,
) -> Result<bool, Failure> {
    let mut error = 0;
    let reverse = is_split_shape_to_reverse(split, original, context, &mut error)?;
    if error != 0 {
        if let Some(report) = report {
            append_orientation_warning(split, original, report)?;
        }
    }
    Ok(reverse)
}
fn append_orientation_warning(
    split: &Shape,
    original: &Shape,
    report: &Report,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, split)?;
    builder.add(&mut shape, original)?;
    let alert: AlertHandle = Arc::new(AlertUnableToOrientTheShape::new(&shape));
    report.add_alert(Gravity::Warning, &alert);
    Ok(())
}
