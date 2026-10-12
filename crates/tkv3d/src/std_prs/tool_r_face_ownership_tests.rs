//! Tool ownership preserves real topology without retaining its application surface or invoking hidden Face.
use super::{
    StdPrsIsolineSurfaceSource, StdPrsToolRFace, tool_r_face_source_fake::RFaceLifetimeSourceFake,
};
use crate::select3d::compare_oracle_snapshot;
use rustycad_tkernel::standard::Handle;
const NATIVE_OWNERSHIP: &str =
    include_str!("../../tests/fixtures/tool-r-face/native-ownership.txt");
#[test]
fn native_nonvirtual_parent_face_and_released_source_ownership() {
    let source = Handle::new(RFaceLifetimeSourceFake::new());
    let weak = source.downgrade();
    assert!(!source.face().is_null());
    let tool =
        StdPrsToolRFace::with_surface(&source.cast::<dyn StdPrsIsolineSurfaceSource>()).unwrap();
    let surface_kind =
        rustycad_tkg3d::adaptor3d::Adaptor3dSurface::get_type(&*source).unwrap() as i32 as f64;
    let hidden_calls = *source.hidden_face_calls.read();
    drop(source);
    tool.init().unwrap();
    let mut row = vec![
        0.,
        f64::from(weak.upgrade().is_null()),
        f64::from(hidden_calls),
        surface_kind,
    ];
    append_source_free_traversal(&tool, &mut row);
    compare_oracle_snapshot(&row, NATIVE_OWNERSHIP.trim());
}
fn append_owned_edge_curve(tool: &StdPrsToolRFace, row: &mut Vec<f64>) {
    let edge = tool.edge_ref().unwrap().snapshot().unwrap();
    let curve = tool.value();
    let point = curve
        .value((curve.first_parameter().unwrap() + curve.last_parameter().unwrap()) * 0.5)
        .unwrap();
    row.extend([
        f64::from(edge.is_null()),
        edge.orientation() as i32 as f64,
        point.x(),
        point.y(),
    ]);
}

fn append_source_free_traversal(tool: &StdPrsToolRFace, row: &mut Vec<f64>) {
    let mut count = 0;
    while tool.more() {
        append_owned_edge_curve(tool, row);
        count += 1;
        tool.next().unwrap();
    }
    row.push(f64::from(count));
}
