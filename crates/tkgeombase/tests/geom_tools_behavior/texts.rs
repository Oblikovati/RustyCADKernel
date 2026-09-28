//! The dump and compact layouts, precision handling and the printer quirks.
use crate::fakes::{bspline, circle, line, output_stream, written_text};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomOffsetCurve, GeomSurface, GeomToroidalSurface};
use rustycad_tkgeombase::geom_tools::{self, CurveSet};
use rustycad_tkmath::gp::{Axis3, Point2};

fn printed(print: impl FnOnce(&mut IoStream)) -> String {
    let mut stream = output_stream();
    print(&mut stream);
    written_text(stream)
}

#[test]
fn line_dump_labels_every_field_and_compact_writes_the_type_code() {
    let curve = line(0.1);
    let dump = printed(|s| geom_tools::dump_curve(&curve, s).unwrap());
    assert_eq!(dump, "Line\n  Origin :0.1, 0, 0 \n  Axis   :0, 0, 1 \n\n");
    let compact = printed(|s| geom_tools::write_curve(&curve, s).unwrap());
    assert_eq!(compact, "1 0.1 0 0 0 0 1 \n");
}

#[test]
fn geomtools_write_keeps_the_stream_precision_while_the_set_uses_17_and_restores_it() {
    let curve = line(0.1);
    let short = printed(|s| geom_tools::write_curve(&curve, s).unwrap());
    assert!(short.contains("0.1 "), "precision 6: {short}");
    let mut set = CurveSet::new();
    set.add(&curve);
    let mut stream = output_stream();
    stream.set_precision(4);
    set.write(&mut stream, &Default::default()).unwrap();
    assert_eq!(
        stream.precision(),
        4,
        "Write restores the caller's precision"
    );
    assert_eq!(
        written_text(stream),
        "Curves 1\n1 0.10000000000000001 0 0 0 0 1 \n"
    );
}

#[test]
fn circle_dump_prints_the_frame_and_radius() {
    let dump = printed(|s| geom_tools::dump_curve(&circle(2.5), s).unwrap());
    let expected = "Circle\n  Center :0, 0, 0 \n  Axis   :0, 0, 1 \n  XAxis  :1, 0, 0 \n  YAxis  :0, 1, 0 \n  Radius :2.5\n\n";
    assert_eq!(dump, expected);
}

#[test]
fn three_d_bspline_dump_puts_two_spaces_before_each_knot() {
    let dump = printed(|s| geom_tools::dump_curve(&bspline(), s).unwrap());
    assert!(
        dump.starts_with("BSplineCurve\n  Degree 3, 5 Poles, 3  Knots\nPoles :\n"),
        "{dump}"
    );
    assert!(dump.contains("\n   1 :  0 4"), "{dump}");
}

#[test]
fn two_d_bspline_dump_runs_knots_into_poles_and_uses_one_space_before_knots() {
    let poles = [
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(2.0, 0.0),
    ];
    let mut pole_array = rustycad_tkernel::ncollection::Array1::with_bounds(1, 3).unwrap();
    for (index, pole) in poles.into_iter().enumerate() {
        pole_array.set_value(index as i32 + 1, pole).unwrap();
    }
    let knots = crate::fakes::array(&[0.0, 1.0]);
    let mut mults = rustycad_tkernel::ncollection::Array1::with_bounds(1, 2).unwrap();
    mults.set_value(1, 3).unwrap();
    mults.set_value(2, 3).unwrap();
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBSplineCurve::new(&pole_array, &knots, &mults, 2, false).unwrap())
            .into_cast();
    let dump = printed(|s| geom_tools::dump_curve2d(&curve, s).unwrap());
    assert!(dump.contains("3 Poles, 2  KnotsPoles :\n"), "{dump}");
    assert!(dump.contains("\n   1 : 0 3"), "{dump}");
}

#[test]
fn offset_curve_dump_has_no_newline_after_its_tag() {
    let offset: Handle<dyn GeomCurve> = Handle::new(
        GeomOffsetCurve::new(&circle(1.0), 0.5, crate::fakes::z_direction(), false).unwrap(),
    )
    .into_cast();
    let dump = printed(|s| geom_tools::dump_curve(&offset, s).unwrap());
    assert!(
        dump.starts_with("OffsetCurveOffset : 0.5\nDirection : 0, 0, 1 \nBasis curve :\nCircle"),
        "{dump}"
    );
}

#[test]
fn torus_dump_separates_its_radii_without_a_comma() {
    let torus: Handle<dyn GeomSurface> =
        Handle::new(GeomToroidalSurface::new(Axis3::default(), 5.0, 1.0).unwrap()).into_cast();
    let dump = printed(|s| geom_tools::dump_surface(&torus, s).unwrap());
    assert!(dump.ends_with("\n  Radii  :5 1\n\n"), "{dump}");
    let compact = printed(|s| geom_tools::write_surface(&torus, s).unwrap());
    assert_eq!(compact, "5 0 0 0 0 0 1 1 0 0 0 1 0 5 1\n");
}

#[test]
fn a_null_handle_is_refused_where_the_native_code_dereferences_it() {
    let mut stream = output_stream();
    let error = geom_tools::dump_curve(&Handle::default(), &mut stream).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    let error = geom_tools::write_surface(&Handle::default(), &mut stream).unwrap_err();
    assert!(
        error.message().contains("Geom_Surface"),
        "{}",
        error.message()
    );
    let error = geom_tools::write_curve2d(&Handle::default(), &mut stream).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
}

#[test]
fn numbers_follow_printf_g_at_the_stream_precision() {
    let curve = line(123456.5);
    assert_eq!(
        printed(|s| geom_tools::write_curve(&curve, s).unwrap()),
        "1 123456 0 0 0 0 1 \n"
    );
    let curve = line(1e-5);
    assert_eq!(
        printed(|s| geom_tools::write_curve(&curve, s).unwrap()),
        "1 1e-05 0 0 0 0 1 \n"
    );
}
