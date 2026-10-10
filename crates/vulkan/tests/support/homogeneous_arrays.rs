//! Full position weights mirror the native Graphic3d buffer oracle, including directions at infinity.
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};
pub const MODES: [&str; 21] = [
    "hom-triangle-unit",
    "hom-triangle-double",
    "hom-triangle-mixed",
    "hom-triangle-infinite",
    "hom-triangle-negative",
    "hom-triangle-behind",
    "hom-triangle-clipped",
    "hom-line-unit",
    "hom-line-double",
    "hom-line-mixed",
    "hom-line-infinite",
    "hom-line-negative",
    "hom-line-behind",
    "hom-line-clipped",
    "hom-point-unit",
    "hom-point-double",
    "hom-point-mixed",
    "hom-point-infinite",
    "hom-point-negative",
    "hom-point-behind",
    "hom-point-clipped",
];
/// Replace geometry with live Vec4 positions; `replace_homogeneous(group, "hom-triangle-double")`.
pub fn replace_homogeneous(group: &dyn Graphic3dGroup, mode: &str) -> Handle<Graphic3dBuffer> {
    assert!(
        MODES.contains(&mode),
        "received {mode}; expected a homogeneous capture mode"
    );
    prepare_group(group);
    let buffer = weighted_buffer(mode);
    group.add_primitive_array_buffers(
        homogeneous_topology(mode),
        &Handle::default(),
        &buffer.cast(),
        &Handle::default(),
        true,
    );
    buffer
}
fn prepare_group(group: &dyn Graphic3dGroup) {
    let aspects = group.aspects();
    aspects.set_marker_type(AspectTypeOfMarker::Plus);
    aspects.set_marker_scale(2.).unwrap();
    group.clear(false);
    group.set_group_primitives_aspect(&aspects);
}
fn homogeneous_topology(mode: &str) -> Graphic3dTypeOfPrimitiveArray {
    if mode.starts_with("hom-line-") {
        return Graphic3dTypeOfPrimitiveArray::Polylines;
    }
    if mode.starts_with("hom-point-") {
        return Graphic3dTypeOfPrimitiveArray::Points;
    }
    Graphic3dTypeOfPrimitiveArray::Triangles
}

fn weighted_buffer(mode: &str) -> Handle<Graphic3dBuffer> {
    let buffer =
        Handle::new(Graphic3dBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    let attributes = [Graphic3dAttribute {
        id: Graphic3dTypeOfAttribute::Pos,
        data_type: Graphic3dTypeOfData::Vec4,
    }];
    assert!(buffer.init(3, &attributes).unwrap());
    let weights = position_weights(mode);
    for (index, [x, y, z]) in [[-8., -8., 0.], [8., -8., 0.], [0., 8., 0.]]
        .into_iter()
        .enumerate()
    {
        assert!(buffer.write_at(index * 16, Vec4::new(x, y, z, weights[index])));
    }
    buffer
}
fn position_weights(mode: &str) -> [f32; 3] {
    match mode.rsplit('-').next().unwrap() {
        "unit" => [1.; 3],
        "double" => [2.; 3],
        "infinite" => [1., 0., 0.],
        // The original negative case collapses to a line after division by w.
        // Keep it as a diagnostic alongside a nondegenerate negative-weight triangle.
        "negative" => [1., 1., -1.],
        "behind" => [1., 1., -2.],
        "mixed" | "clipped" => [0.5, 2., 1.],
        _ => unreachable!("validated homogeneous capture mode"),
    }
}
