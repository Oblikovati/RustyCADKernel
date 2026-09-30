//! SplitSurface state histories and geometric patch digests.
use super::{
    analysis_surface::analysis_surface_geometry,
    common::{Reader, attempt, end, flag, int, pnt, real, xyz},
    sh_common::surface,
};
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkshhealing::{
    shape_extend::{ShapeExtendCompositeSurface, ShapeExtendStatus},
    shape_upgrade::{ShapeUpgradeSplitSurface, SplitSurface, SurfaceSplitValues},
};
pub fn split_surface_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| split_surface_request(&mut reader));
    end();
}
fn split_surface_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let input = analysis_surface_geometry(reader)?;
    let splitter = ShapeUpgradeSplitSurface::new();
    let measure = initialize_surface(&splitter, &input, reader)?;
    print_splitter(&splitter, &input, &Handle::default(), measure)?;
    for _ in 0..reader.integer() {
        let previous = splitter.result_surfaces();
        surface_operation(&splitter, &input, reader);
        print_splitter(&splitter, &input, &previous, measure)?;
    }
    surface(&input);
    Ok(())
}
fn initialize_surface(
    splitter: &ShapeUpgradeSplitSurface,
    input: &Handle<dyn GeomSurface>,
    reader: &mut Reader<'_>,
) -> Result<bool, Failure> {
    let natural = reader.flag();
    let bounds = (reader.real(), reader.real(), reader.real(), reader.real());
    let area = reader.real();
    if natural {
        splitter.init_surface(input)?;
    } else {
        splitter.init_surface_bounds(input, bounds, area)?;
    }
    Ok(!natural && area != 0.)
}
fn surface_operation(
    splitter: &ShapeUpgradeSplitSurface,
    input: &Handle<dyn GeomSurface>,
    reader: &mut Reader<'_>,
) {
    match reader.integer() {
        0 => add_values(splitter, reader),
        op @ (1 | 2 | 5) => build_operation(splitter, reader, op),
        3 => change_values(splitter, reader),
        4 => {
            let status = reader.integer();
            splitter
                .split_surface_data()
                .with_mut(|f| f.status = status);
        }
        _ => {
            attempt(|| splitter.init_surface(input));
        }
    }
}
fn build_operation(splitter: &ShapeUpgradeSplitSurface, reader: &mut Reader<'_>, operation: i32) {
    let segment = reader.flag();
    attempt(|| match operation {
        1 => splitter.build(segment),
        2 => splitter.perform(segment),
        _ => splitter.compute(segment),
    });
}
fn add_values(splitter: &ShapeUpgradeSplitSurface, reader: &mut Reader<'_>) {
    let u = reader.flag();
    let values = Handle::new(HSequence::default());
    for _ in 0..reader.integer() {
        values.append(reader.real());
    }
    attempt(|| {
        if u {
            splitter.set_u_split_values(&values)
        } else {
            splitter.set_v_split_values(&values)
        }
    });
}
fn change_values(splitter: &ShapeUpgradeSplitSurface, reader: &mut Reader<'_>) {
    let u = reader.flag();
    let count = reader.integer();
    let values = if u {
        splitter.u_split_values()
    } else {
        splitter.v_split_values()
    };
    values.change_sequence(|s| s.clear(None));
    for _ in 0..count {
        values.append(reader.real());
    }
}
fn print_values(values: &SurfaceSplitValues) {
    let values = values.sequence(|s| s.iter().copied().collect::<Vec<_>>());
    int(values.len() as i32);
    values.into_iter().for_each(real);
}
fn print_splitter(
    splitter: &ShapeUpgradeSplitSurface,
    input: &Handle<dyn GeomSurface>,
    previous: &Handle<ShapeExtendCompositeSurface>,
    measure: bool,
) -> Result<(), Failure> {
    print_values(&splitter.u_split_values());
    print_values(&splitter.v_split_values());
    print_fields(splitter, measure);
    let result = splitter.result_surfaces();
    flag(result.object_address() == previous.object_address());
    print_result(&result, input)?;
    print_result(previous, input)
}
fn print_fields(splitter: &ShapeUpgradeSplitSurface, measure: bool) {
    for status in [
        ShapeExtendStatus::Ok,
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Fail,
    ] {
        flag(splitter.status_flag(status));
    }
    splitter.split_surface_data().with(|f| {
        int(f.rows);
        int(f.columns);
        if measure {
            real(f.area);
            real(f.u_size);
            real(f.v_size);
        }
    });
}
fn print_result(
    result: &Handle<ShapeExtendCompositeSurface>,
    input: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    flag(result.is_null());
    let Some(result) = result.get() else {
        return Ok(());
    };
    flag(result.patches().is_null());
    if result.patches().is_null() {
        return Ok(());
    }
    int(result.nb_u_patches()?);
    int(result.nb_v_patches()?);
    for values in [result.u_joint_values(), result.v_joint_values()] {
        for value in values.array1().iter() {
            real(*value);
        }
    }
    print_patches(result, input)
}
fn print_patches(
    result: &ShapeExtendCompositeSurface,
    input: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    for row in 1..=result.nb_u_patches()? {
        for column in 1..=result.nb_v_patches()? {
            let patch = result.patch(row, column)?;
            flag(patch.object_address() == input.object_address());
            surface(&patch);
            surface_midpoint(&patch);
        }
    }
    Ok(())
}

fn surface_midpoint(surface: &Handle<dyn GeomSurface>) {
    let (u1, u2, v1, v2) = surface.bounds();
    if [u1, u2, v1, v2]
        .iter()
        .any(|v| rustycad_tkernel::precision::is_infinite(*v))
    {
        return;
    }
    attempt(|| {
        let d1 = surface.d1((u1 + u2) * 0.5, (v1 + v2) * 0.5)?;
        pnt(d1.point);
        xyz(*d1.d1u.xyz());
        xyz(*d1.d1v.xyz());
        Ok(())
    });
}
