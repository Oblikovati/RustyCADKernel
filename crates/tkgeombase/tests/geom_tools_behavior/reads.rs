//! GetReal and the single-record readers: round trips, the failure-to-null contract and
//! where a failure leaves the stream.
use crate::fakes::{bspline, circle, output_stream, text_stream, unread, written_text};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::storage::state;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCircle, GeomPlane};
use rustycad_tkgeombase::geom_tools;

#[test]
fn get_real_reads_one_word_and_converts_its_numeric_prefix() {
    let mut stream = text_stream(" 12.5 1.5abc xyz 1e400 0x10");
    let values: Vec<f64> = (0..5).map(|_| geom_tools::get_real(&mut stream)).collect();
    assert_eq!(values, [12.5, 1.5, 0.0, f64::INFINITY, 16.0]);
}

#[test]
fn get_real_splits_a_word_longer_than_thirty_characters_and_restores_the_width() {
    let mut stream = text_stream("0.1234567890123456789012345678901234 ");
    stream.set_width(7);
    let first = geom_tools::get_real(&mut stream);
    assert_eq!(stream.width(), 7, "the caller's width comes back");
    assert_eq!(
        first,
        "0.1234567890123456789012345678".parse::<f64>().unwrap()
    );
    assert_eq!(geom_tools::get_real(&mut stream), 901234.0);
}

#[test]
fn get_real_returns_zero_at_end_of_file_without_touching_the_stream() {
    let mut stream = text_stream("1");
    assert_eq!(geom_tools::get_real(&mut stream), 1.0);
    assert!(stream.eof());
    assert_eq!(geom_tools::get_real(&mut stream), 0.0);
    assert_eq!(
        stream.rdstate(),
        state::EOF,
        "no failbit: the extraction never ran"
    );
}

#[test]
fn a_bspline_round_trips_bit_for_bit_at_precision_17() {
    let curve = bspline();
    let mut stream = output_stream();
    stream.set_precision(17);
    geom_tools::write_curve(&curve, &mut stream).unwrap();
    let mut input = text_stream(&written_text(stream));
    let back: Handle<GeomBSplineCurve> = geom_tools::read_curve(&mut input).cast();
    let original: Handle<GeomBSplineCurve> = curve.cast();
    let bits = |c: &Handle<GeomBSplineCurve>| -> Vec<u64> {
        let poles = c
            .poles()
            .iter()
            .flat_map(|p| [p.x(), p.y(), p.z()])
            .collect::<Vec<_>>();
        poles
            .into_iter()
            .chain(c.knots().iter().copied())
            .map(f64::to_bits)
            .collect()
    };
    assert_eq!(bits(&back), bits(&original));
}

#[test]
fn a_record_that_fails_to_build_reads_as_null_after_its_radius() {
    // gp_Ax2 raises for parallel axes only once the radius is read, as natively.
    let mut stream = text_stream("2 0 0 0 0 0 1 0 0 1 0 1 0 7 rest");
    assert!(geom_tools::read_curve(&mut stream).is_null());
    assert_eq!(unread(&mut stream), " rest");
}

#[test]
fn a_null_direction_stops_the_record_where_it_is_read() {
    let mut stream = text_stream("2 0 0 0 0 0 0 1 0 0 0 1 0 7");
    assert!(geom_tools::read_curve(&mut stream).is_null());
    assert_eq!(unread(&mut stream), " 1 0 0 0 1 0 7");
}

#[test]
fn an_unknown_type_code_reads_nothing_more() {
    let _guard = crate::fakes::handler_lock();
    let mut stream = text_stream("42 1 2");
    assert!(geom_tools::read_curve(&mut stream).is_null());
    assert_eq!(unread(&mut stream), " 1 2");
}

#[test]
fn a_boolean_other_than_zero_or_one_fails_the_stream() {
    let mut stream = text_stream("6 2 1 0 0 0 1 1 1");
    assert!(geom_tools::read_curve(&mut stream).is_null());
    assert!(stream.fail());
}

#[test]
fn a_trimmed_curve_on_a_null_basis_reads_as_null() {
    let _guard = crate::fakes::handler_lock();
    let mut stream = text_stream("8 0 1 42 5");
    assert!(geom_tools::read_curve(&mut stream).is_null());
    assert_eq!(unread(&mut stream), " 5");
}

#[test]
fn an_indirect_surface_frame_is_restored_from_its_y_direction() {
    let mut stream = text_stream("1 0 0 0 0 0 1 1 0 0 0 -1 0 ");
    let plane: Handle<GeomPlane> = geom_tools::read_surface(&mut stream).cast();
    assert!(
        !plane.pln().direct().unwrap(),
        "AY.DotCross(A, AX) < 0 reverses Y"
    );
}

#[test]
fn a_circle_round_trips_through_the_2d_and_3d_readers_independently() {
    let mut stream = output_stream();
    geom_tools::write_curve(&circle(2.5), &mut stream).unwrap();
    let mut input = text_stream(&written_text(stream));
    let back: Handle<GeomCircle> = geom_tools::read_curve(&mut input).cast();
    assert_eq!(back.radius(), 2.5);
    let mut input = text_stream("2 0 0 1 0 0 -1 2.5 ");
    let back = geom_tools::read_curve2d(&mut input);
    assert!(back.is_instance_name("Geom2d_Circle"));
}
