//! The Rust twin of tools/oracle/tkbool_probe_brepfill_evolved.cxx.
use super::brep_fill_generator::failed;
use super::brep_fill_location_law::at;
use super::common::{Reader, attempt, end, flag, int};
use super::evolved_digest::evolved_digest as shape;
use super::shapes::tp_shapes;
use rustycad_tkbool::brep_fill::BRepFillEvolved;
use rustycad_tkbrep::{
    topexp,
    topods::{self, Shape, Wire},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{geom_abs::JoinType, gp::Axis3};
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

fn evolved_result(evolved: &BRepFillEvolved) {
    flag(evolved.is_done());
    attempt(|| shape(evolved.shape()));
    attempt(|| shape(evolved.top()));
    attempt(|| shape(evolved.bottom()));
}

fn evolved_history(
    evolved: &BRepFillEvolved,
    spine: &Shape,
    profile: &Shape,
) -> Result<(), Failure> {
    let (mut spines, mut profiles) = (IndexedMapOfShape::default(), IndexedMapOfShape::default());
    topexp::map_shapes(spine, &mut spines, true, true);
    topexp::map_shapes(profile, &mut profiles, true, true);
    for s in spines.iter() {
        for p in profiles.iter() {
            let generated = evolved.generated_shapes(s, p);
            int(generated.len() as i32);
            for result in generated.iter() {
                int(result.shape_type()? as i32);
                int(result.orientation() as i32);
            }
        }
    }
    Ok(())
}

fn perform_evolved(
    spine: &Wire,
    profile: &Wire,
    face: bool,
    solid: bool,
    constructor: bool,
    join: JoinType,
    axes: &Axis3,
) -> Result<BRepFillEvolved, Failure> {
    let mut evolved = BRepFillEvolved::new();
    if face {
        let spine = BRepLibMakeFace::from_wire(spine, true)?.face()?;
        if constructor {
            return BRepFillEvolved::from_face(&spine, profile, axes, join, solid);
        }
        evolved.perform_face(&spine, profile, axes, join, solid)?;
    } else {
        if constructor {
            return BRepFillEvolved::from_wire(spine, profile, axes, join, solid);
        }
        evolved.perform_wire(spine, profile, axes, join, solid)?;
    }
    Ok(evolved)
}

/// Compare defined default-state queries; `evolved_empty_case(&[])`.
pub fn evolved_empty_case(_: &[f64]) {
    let evolved = BRepFillEvolved::new();
    evolved_result(&evolved);
    int(evolved
        .generated_shapes(&Shape::default(), &Shape::default())
        .len() as i32);
    end();
}

fn evolved_record(r: &mut Reader<'_>) -> Result<(), Failure> {
    let table = tp_shapes(r)?;
    let (spine, profile) = (
        topods::wire(&at(&table, r.integer())?)?,
        topods::wire(&at(&table, r.integer())?)?,
    );
    let (face, solid, constructor) = (r.flag(), r.flag(), r.flag());
    let join = match r.integer() {
        0 => JoinType::Arc,
        1 => JoinType::Tangent,
        _ => JoinType::Intersection,
    };
    let axes = Axis3::new(r.pnt(), r.dir(), r.dir())?;
    let evolved = perform_evolved(&spine, &profile, face, solid, constructor, join, &axes)?;
    evolved_result(&evolved);
    int(evolved.join_type() as i32);
    evolved_history(&evolved, spine.as_ref(), profile.as_ref())
}

/// Compare one evolved sweep; `evolved_case(&request_numbers)`.
pub fn evolved_case(values: &[f64]) {
    if let Err(error) = evolved_record(&mut Reader::new(values)) {
        return failed(error);
    }
    end();
}
