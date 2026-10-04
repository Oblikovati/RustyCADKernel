//! Native most-derived exception names survive construction and copying through a base value.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors as plane_errors;
use rustycad_tkg3d::geom::errors as space_errors;

type GeometryExceptionFactory = fn(&str) -> Failure;

#[test]
fn geometry_exception_names_match_native_base_dispatch() {
    let factories: [GeometryExceptionFactory; 4] = [
        plane_errors::undefined_derivative,
        plane_errors::undefined_value,
        space_errors::undefined_derivative,
        space_errors::undefined_value,
    ];
    let observed: String = factories.into_iter().map(exception_type_receipt).collect();
    assert_eq!(
        observed,
        include_str!("../../../tools/oracle/geometry_exception_types.expected")
    );
}

fn exception_type_receipt(factory: GeometryExceptionFactory) -> String {
    let original = factory("geometry probe");
    let copied = original.clone();
    format!(
        "{} {}\n",
        original.exception_type(),
        copied.exception_type()
    )
}
