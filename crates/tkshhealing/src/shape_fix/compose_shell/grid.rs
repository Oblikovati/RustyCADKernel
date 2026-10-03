use crate::shape_analysis::adjust_to_period;
use crate::shape_extend::JointValues;
use crate::shape_fix::ShapeFixWireSegment;
use rustycad_tkernel::standard::Failure;

pub(super) fn patch_index(
    parameter: f64,
    joints: &JointValues,
    is_closed: bool,
) -> Result<i32, Failure> {
    let values = joints.array1();
    let upper = values.upper();
    let (first, last) = joint_endpoints(&values, upper)?;
    let shift = parameter_shift(parameter, first, last, is_closed);
    let index = patch_containing(parameter + shift, &values, upper)?;
    Ok(index - rounded_patch_shift(shift, last - first) * (upper - 1))
}

fn joint_endpoints(
    joints: &rustycad_tkernel::ncollection::Array1<'_, f64>,
    upper: i32,
) -> Result<(f64, f64), Failure> {
    let first = joint_value(joints, 1)?;
    let last = joint_value(joints, upper)?;
    Ok((first, last))
}

fn joint_value(
    joints: &rustycad_tkernel::ncollection::Array1<'_, f64>,
    index: i32,
) -> Result<f64, Failure> {
    joints
        .value(index)
        .copied()
        .map_err(|error| error.into_failure("ShapeFix_ComposeShell::GetPatchIndex"))
}

fn parameter_shift(parameter: f64, first: f64, last: f64, is_closed: bool) -> f64 {
    if is_closed {
        adjust_to_period(parameter, first, last)
    } else {
        0.0
    }
}

fn rounded_patch_shift(shift: f64, period: f64) -> i32 {
    let periods = shift / period;
    (if periods < 0.0 {
        periods - 0.5
    } else {
        periods + 0.5
    }) as i32
}

fn patch_containing(
    parameter: f64,
    joints: &rustycad_tkernel::ncollection::Array1<'_, f64>,
    upper: i32,
) -> Result<i32, Failure> {
    for index in 2..upper {
        if parameter < joint_value(joints, index)? {
            return Ok(index - 1);
        }
    }
    Ok(upper - 1)
}

pub(super) fn define_patch(
    wire: &ShapeFixWireSegment,
    code: i32,
    cut_by_u: bool,
    cut_index: i32,
    edge_index: Option<i32>,
) -> Result<(), Failure> {
    let edge = edge_index.unwrap_or(wire.nb_edges());
    if cut_by_u {
        define_u_patch(wire, code, cut_index, edge)?;
    } else {
        define_v_patch(wire, code, cut_index, edge)?;
    }
    Ok(())
}

fn define_u_patch(
    wire: &ShapeFixWireSegment,
    code: i32,
    cut: i32,
    edge: i32,
) -> Result<(), Failure> {
    if code & 1 == 0 {
        wire.define_u_min(edge, cut)?;
    }
    if code & 2 == 0 {
        wire.define_u_max(edge, cut)?;
    }
    Ok(())
}

fn define_v_patch(
    wire: &ShapeFixWireSegment,
    code: i32,
    cut: i32,
    edge: i32,
) -> Result<(), Failure> {
    if code & 2 == 0 {
        wire.define_v_min(edge, cut)?;
    }
    if code & 1 == 0 {
        wire.define_v_max(edge, cut)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::ncollection::HArray1;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkmath::gp::Point3;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

    #[test]
    fn patch_indices_choose_the_interval_above_an_exact_joint() {
        let joints = joints(&[0.0, 1.0, 3.0]);
        assert_eq!(patch_index(0.5, &joints, false).unwrap(), 1);
        assert_eq!(patch_index(1.0, &joints, false).unwrap(), 2);
        assert_eq!(patch_index(2.0, &joints, false).unwrap(), 2);
    }

    #[test]
    fn side_bits_constrain_only_the_native_patch_limits() {
        let expected =
            include_str!("../../../../../tools/oracle/tkshhealing_compose_shell.expected");
        let edge = BRepBuilderAPIMakeEdge::from_points(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
        )
        .unwrap()
        .edge()
        .unwrap();
        let segment = ShapeFixWireSegment::new();
        segment.add_edge(0, &edge).unwrap();
        define_patch(&segment, 0, true, 2, None).unwrap();
        define_patch(&segment, 1, false, 3, Some(1)).unwrap();
        let patch = segment.patch_index(1).unwrap();
        assert_eq!((patch.u_min, patch.u_max), (2, 2));
        assert_eq!((patch.v_min, patch.v_max), (3, 32_000));
        assert_eq!(
            format!(
                "patch_limits {} {} {} {}",
                patch.u_min, patch.u_max, patch.v_min, patch.v_max
            ),
            expected.lines().nth(6).unwrap()
        );
    }

    fn joints(values: &[f64]) -> JointValues {
        let upper = values.len() as i32;
        let array = HArray1::with_value(1, upper, 0.0).unwrap();
        for (index, value) in values.iter().enumerate() {
            array
                .change_array1()
                .set_value(index as i32 + 1, *value)
                .unwrap();
        }
        Handle::new(array)
    }
}
