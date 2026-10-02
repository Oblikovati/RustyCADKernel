//! WireSegment histories compare sharing, ordered topology and patch intervals.
use super::{
    brep_lib_common::shape,
    common::{Reader, attempt, end, flag, int},
    shapes::{TpShapes, tp_same_index, tp_shapes},
    wire_data::{sh_edge, sh_shape},
};
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendWireData,
    shape_fix::{ShapeFixWireSegment, WirePatchIndex},
};
/// Compare native segment histories; `wire_segment_case(&request)`.
pub fn wire_segment_case(values: &[f64]) {
    attempt(|| segment_request(&mut Reader::new(values)));
    end();
}
fn segment_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let shapes = tp_shapes(reader)?;
    let source = Handle::new(ShapeExtendWireData::new());
    source.set_manifold_mode(reader.flag());
    for _ in 0..reader.integer() {
        source.add_edge(&sh_edge(&shapes, reader.integer())?, 0)?;
    }
    let constructor = reader.integer();
    let orientation = Orientation::ALL[reader.integer() as usize];
    let segment = match constructor {
        0 => ShapeFixWireSegment::new(),
        1 => ShapeFixWireSegment::from_wire_data(&source, orientation)?,
        2 => ShapeFixWireSegment::from_wire_data(&Handle::default(), orientation)?,
        _ => unreachable!(),
    };
    let mut segments = [segment.clone(), segment];
    print_state(&segments[0], &shapes)?;
    segment_steps(&mut segments, &source, &shapes, reader)
}
fn segment_steps(
    segments: &mut [ShapeFixWireSegment; 2],
    source: &Handle<ShapeExtendWireData>,
    shapes: &TpShapes,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    for _ in 0..reader.integer() {
        let target = reader.integer() as usize;
        let operation = reader.integer();
        let operands: Vec<i32> = (0..reader.integer()).map(|_| reader.integer()).collect();
        attempt(|| segment_operation(segments, source, shapes, target, operation, &operands));
        for segment in segments.iter() {
            print_state(segment, shapes)?;
        }
        flag(segments[0].wire_data() == segments[1].wire_data());
        flag(segments[0].wire_data() == source);
    }
    Ok(())
}
fn segment_operation(
    segments: &mut [ShapeFixWireSegment; 2],
    source: &Handle<ShapeExtendWireData>,
    shapes: &TpShapes,
    target: usize,
    op: i32,
    a: &[i32],
) -> Result<(), Failure> {
    match op {
        0 => segments[target].clear(),
        1 => segments[target].load(source)?,
        12 => segments[target] = segments[1 - target].clone(),
        15 => {
            let other = segments[1 - target].wire_data().clone();
            segments[target].load(&other)?;
        }
        _ => segment_edit(&mut segments[target], shapes, op, a)?,
    }
    Ok(())
}
fn segment_edit(
    segment: &mut ShapeFixWireSegment,
    shapes: &TpShapes,
    op: i32,
    a: &[i32],
) -> Result<(), Failure> {
    match op {
        2 => segment.set_orientation(Orientation::ALL[a[0] as usize]),
        3 => segment.add_edge(a[0], &sh_edge(shapes, a[1])?)?,
        4 => segment.add_edge_with_patch(a[0], &sh_edge(shapes, a[1])?, patch(&a[2..]))?,
        5 => segment.set_edge(a[0], &sh_edge(shapes, a[1])?)?,
        6 => segment.set_patch_index(a[0], patch(&a[1..]))?,
        7 => segment.define_u_min(a[0], a[1])?,
        8 => segment.define_u_max(a[0], a[1])?,
        9 => segment.define_v_min(a[0], a[1])?,
        10 => segment.define_v_max(a[0], a[1])?,
        11 => segment.set_vertex(&topods::vertex(&sh_shape(shapes, a[0]))?),
        13 => segment
            .wire_data()
            .add_edge(&sh_edge(shapes, a[0])?, a[1])?,
        14 => segment.reload(),
        16 => segment.load(&Handle::default())?,
        _ => unreachable!(),
    }
    Ok(())
}
fn patch(values: &[i32]) -> WirePatchIndex {
    WirePatchIndex {
        u_min: values[0],
        u_max: values[1],
        v_min: values[2],
        v_max: values[3],
    }
}
fn print_patch(segment: &ShapeFixWireSegment, index: i32) -> Result<(), Failure> {
    let patch = segment.patch_index(index)?;
    for value in [patch.u_min, patch.u_max, patch.v_min, patch.v_max] {
        int(value);
    }
    flag(segment.check_patch_index(index)?);
    Ok(())
}
fn print_state(segment: &ShapeFixWireSegment, shapes: &TpShapes) -> Result<(), Failure> {
    int(segment.orientation() as i32);
    int(segment.nb_edges());
    flag(segment.wire_data().manifold_mode());
    for index in 1..=segment.nb_edges() {
        print_edge(shapes, &segment.edge(index)?);
    }
    attempt(|| shape(&segment.first_vertex()?.into_shape()));
    attempt(|| shape(&segment.last_vertex()?.into_shape()));
    attempt(|| segment.is_closed().map(flag));
    flag(segment.is_vertex());
    shape(&segment.vertex())?;
    for index in 0..=segment.nb_edges() + 1 {
        attempt(|| print_patch(segment, index));
    }
    attempt(|| segment.edge(-1).map(|edge| print_edge(shapes, &edge)));
    Ok(())
}
fn print_edge(shapes: &TpShapes, edge: &topods::Edge) {
    int(tp_same_index(shapes, edge));
    int(edge.orientation() as i32);
}
