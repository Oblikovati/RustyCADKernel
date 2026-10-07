//! Graphic3d primitive arrays and buffers; the byte layout is compared with the native
//! classes by the sv_array probe.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;

#[test]
fn triangles_store_interleaved_vertices() {
    let array = Graphic3dArrayOfTriangles::with_options(3, 0, true, true, false).unwrap();
    assert_eq!(
        array
            .add_vertex_normal_f64([1.0, 2.0, 3.0], [0.0, 0.0, 1.0])
            .unwrap(),
        1
    );
    array.set_vertex_color_rgb(1, 1.0, 0.5, 0.0).unwrap();
    let buffer = array.attributes();
    assert_eq!(buffer.stride(), 12 + 12 + 4);
    assert_eq!(buffer.read_at::<f32>(4), Some(2.0));
    assert_eq!(array.vertex_color4(1).unwrap().y, 127);
    assert_eq!(array.vertice_xyz(1).unwrap(), [1.0, 2.0, 3.0]);
    assert_eq!(
        array
            .set_vertice(4, 0.0, 0.0, 0.0)
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn large_arrays_use_32_bit_indices() {
    let small = Graphic3dArrayOfTriangles::new(65534, 3, 0).unwrap();
    assert_eq!(small.indices().stride(), 2);
    let large = Graphic3dArrayOfTriangles::new(65535, 3, 0).unwrap();
    assert_eq!(large.indices().stride(), 4);
    large.add_vertex(0.0, 0.0, 0.0).unwrap();
    assert_eq!(large.add_edge(1).unwrap(), 1);
    assert_eq!(large.edge(1).unwrap(), 1);
}

#[test]
fn create_array_returns_the_class_of_the_type() {
    let array =
        Graphic3dArrayOfPrimitives::create_array(Graphic3dTypeOfPrimitiveArray::Segments, 4, 4, 0)
            .unwrap();
    assert_eq!(
        array
            .dynamic_type()
            .map(|kind| kind.name().to_owned())
            .as_deref(),
        Some("Graphic3d_ArrayOfSegments")
    );
    assert_eq!(array.string_type(), "ArrayOfSegments");
    assert!(
        Graphic3dArrayOfPrimitives::create_array(Graphic3dTypeOfPrimitiveArray::Undefined, 4, 0, 0)
            .unwrap()
            .is_null()
    );
    let error = array.add_triangle_edges(1, 2, 3).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_TypeMismatch");
}

#[test]
fn is_valid_truncates_partial_triangles() {
    let array = Graphic3dArrayOfTriangles::new(5, 0, 0).unwrap();
    (0..5).for_each(|i| {
        array.add_vertex(i as f32, 0.0, 0.0).unwrap();
    });
    assert!(array.is_valid());
    assert_eq!(array.vertex_number(), 3);
    assert_eq!(array.item_number(), 1);
}

#[test]
fn bounds_track_counts_and_colors() {
    let array = Graphic3dArrayOfPolylines::with_options(4, 2, 0, false, true).unwrap();
    assert_eq!(array.add_bound_rgb(2, 1.0, 0.0, 0.0).unwrap(), 1);
    assert_eq!(array.bound_color_rgb(1).unwrap(), [1.0, 0.0, 0.0]);
    assert_eq!(array.add_bound(2).unwrap(), 2);
    assert_eq!(
        array.add_bound(1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(array.item_number(), 2);
}

#[test]
fn mutable_buffers_track_invalidated_ranges() {
    let array = Graphic3dArrayOfPoints::new(4, GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_MUTABLE).unwrap();
    let buffer = array.attributes();
    assert!(buffer.is_mutable());
    assert_eq!(array.vertex_number(), 4);
    buffer.invalidate().unwrap();
    assert_eq!(buffer.invalidated_range(), Graphic3dBufferRange::new(0, 48));
    buffer.validate();
    assert!(buffer.invalidated_range().is_empty());
    let indices =
        Graphic3dMutableIndexBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap();
    indices.init_indices::<u16>(4).unwrap();
    assert!(indices.invalidate_indices(2, 1).is_err());
    indices.invalidate_indices(1, 2).unwrap();
    assert_eq!(indices.invalidated_range(), Graphic3dBufferRange::new(2, 4));
}

#[test]
fn deinterleaved_buffers_pack_each_attribute() {
    let array = Graphic3dArrayOfTriangles::new(
        3,
        0,
        GRAPHIC3D_ARRAY_FLAGS_ATTRIBS_DEINTERLEAVED | GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
    )
    .unwrap();
    let buffer = array.attributes();
    assert!(!buffer.is_interleaved());
    assert_eq!(
        buffer.attribute_data(Graphic3dTypeOfAttribute::Norm),
        Some((1, 36, 12))
    );
    let packed =
        Handle::new(Graphic3dAttribBuffer::new(Graphic3dBufferBase::default_allocator()).unwrap());
    let attributes = [Graphic3dAttribute {
        id: Graphic3dTypeOfAttribute::Pos,
        data_type: Graphic3dTypeOfData::Vec3,
    }];
    assert!(packed.init_checked(2, &attributes).unwrap());
    assert!(packed.set_interleaved(false).is_err());
}
