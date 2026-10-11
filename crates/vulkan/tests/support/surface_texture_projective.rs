//! Vec4 UV attributes exercise projective Q interpolation through the actual retained buffer adapter.
use rustycad_tkernel::{
    ncollection::{Vec3, Vec4},
    standard::Handle,
};
use rustycad_tkservice::graphic3d::*;
pub fn projective_surface_group(group: &dyn Graphic3dGroup) -> Handle<Graphic3dBuffer> {
    let buffer = projective_surface_buffer();
    let aspects = group.aspects();
    group.clear(false);
    group.set_group_primitives_aspect(&aspects);
    group.add_primitive_array_buffers(
        Graphic3dTypeOfPrimitiveArray::Triangles,
        &Handle::default(),
        &buffer.cast(),
        &Handle::default(),
        true,
    );
    buffer
}
fn projective_surface_buffer() -> Handle<Graphic3dBuffer> {
    let buffer =
        Handle::new(Graphic3dBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    let attributes = [
        projective_attribute(Graphic3dTypeOfAttribute::Pos, Graphic3dTypeOfData::Vec3),
        projective_attribute(Graphic3dTypeOfAttribute::Norm, Graphic3dTypeOfData::Vec3),
        projective_attribute(Graphic3dTypeOfAttribute::Uv, Graphic3dTypeOfData::Vec4),
    ];
    assert!(buffer.init(6, &attributes).unwrap());
    for (index, rank) in [0, 1, 2, 0, 2, 3].into_iter().enumerate() {
        write_projective_vertex(&buffer, index, rank);
    }
    buffer
}
fn projective_attribute(
    id: Graphic3dTypeOfAttribute,
    data_type: Graphic3dTypeOfData,
) -> Graphic3dAttribute {
    Graphic3dAttribute { id, data_type }
}
fn write_projective_vertex(buffer: &Graphic3dBuffer, index: usize, rank: usize) {
    let points = [[-8., -8., 0.], [8., -8., 0.], [8., 8., 0.], [-8., 8., 0.]];
    let uv = [
        [0., 0., 0., 0.5],
        [1., 0., 0., 1.],
        [1., 1., 0., 2.],
        [0., 1., 0., 0.75],
    ];
    let [x, y, z] = points[rank];
    let [s, t, r, q] = uv[rank];
    assert!(buffer.write_at(index * 40, Vec3::new(x, y, z)));
    assert!(buffer.write_at(index * 40 + 12, Vec3::new(0., 0., 1.)));
    assert!(buffer.write_at(index * 40 + 24, Vec4::new(s, t, r, q)));
}
