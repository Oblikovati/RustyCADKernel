use self::endpoints::join_bezier_ends;
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_lib::same_range;

mod endpoints;
type ReversedPcurve = (Handle<dyn Geom2dCurve>, f64, f64);

struct PcurveNormalization {
    is_seam: bool,
    pair_orientation: Orientation,
    range: (f64, f64),
}

pub(super) fn process_edge_pair(face: &Face, edge: &Edge, next: &Edge) -> Result<(), Failure> {
    let is_seam = BRepTool::is_closed_on_face(edge, face)?;
    let Some(curve) = normalize_current_pcurve(face, edge, is_seam)? else {
        return Ok(());
    };
    let Some(next_curve) = normalize_next_pcurve(face, edge, next, is_seam)? else {
        return Ok(());
    };
    join_bezier_ends(edge, next, &curve, &next_curve)
}

fn normalize_current_pcurve(
    face: &Face,
    edge: &Edge,
    is_seam: bool,
) -> Result<Option<Handle<Geom2dBezierCurve>>, Failure> {
    let Some((curve, first, last)) = bezier_pcurve(edge, face)? else {
        return Ok(None);
    };
    let reversed = reverse_pcurve(edge, face, is_seam)?;
    let range = reversed.as_ref().map_or((first, last), reverse_range);
    let reversed_curve = reversed.map(|(curve, _, _)| curve);
    normalize_pcurve(
        edge,
        face,
        &curve,
        &reversed_curve,
        normalization(is_seam, edge.orientation(), range),
    )?;
    Ok(Some(curve))
}

fn normalize_next_pcurve(
    face: &Face,
    current: &Edge,
    next: &Edge,
    is_seam: bool,
) -> Result<Option<Handle<Geom2dBezierCurve>>, Failure> {
    let Some((curve, first, last)) = bezier_pcurve(next, face)? else {
        return Ok(None);
    };
    let reversed = reverse_pcurve(next, face, is_seam)?;
    let reversed_curve = reversed.map(|(curve, _, _)| curve);
    normalize_pcurve(
        next,
        face,
        &curve,
        &reversed_curve,
        normalization(is_seam, current.orientation(), (first, last)),
    )?;
    Ok(Some(curve))
}

fn reverse_range((_, first, last): &(Handle<dyn Geom2dCurve>, f64, f64)) -> (f64, f64) {
    (*first, *last)
}

fn normalization(
    is_seam: bool,
    pair_orientation: Orientation,
    range: (f64, f64),
) -> PcurveNormalization {
    PcurveNormalization {
        is_seam,
        pair_orientation,
        range,
    }
}

fn bezier_pcurve(
    edge: &Edge,
    face: &Face,
) -> Result<Option<(Handle<Geom2dBezierCurve>, f64, f64)>, Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    let bezier = curve.cast::<Geom2dBezierCurve>();
    if bezier.is_null() {
        return Ok(None);
    }
    Ok(Some((bezier, first, last)))
}

fn reverse_pcurve(edge: &Edge, face: &Face, seam: bool) -> Result<Option<ReversedPcurve>, Failure> {
    if !seam {
        return Ok(None);
    }
    let reversed_shape = edge.reversed();
    let reversed = rustycad_tkbrep::topods::edge(&reversed_shape)?;
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(&reversed, face, false)?;
    Ok(Some((curve, first, last)))
}

fn normalize_pcurve(
    edge: &Edge,
    face: &Face,
    curve: &Handle<Geom2dBezierCurve>,
    reversed: &Option<Handle<dyn Geom2dCurve>>,
    config: PcurveNormalization,
) -> Result<(), Failure> {
    let (first, last) = config.range;
    if first == 0. && last == 1. {
        return Ok(());
    }
    if first > precision::p_confusion() || last < 1. - precision::p_confusion() {
        curve.segment(first, last)?;
        let current_curve: Handle<dyn Geom2dCurve> = curve.clone().cast();
        let reversed_curve = rescale_reverse(reversed, (first, last))?;
        if config.is_seam {
            update_seam_pcurves(
                edge,
                face,
                &current_curve,
                &reversed_curve,
                config.pair_orientation,
            )?;
        }
    }
    BRepBuilder::default().range_on_face(edge, face, 0., 1.)
}

fn update_seam_pcurves(
    edge: &Edge,
    face: &Face,
    current: &Handle<dyn Geom2dCurve>,
    reversed: &Option<Handle<dyn Geom2dCurve>>,
    orientation: Orientation,
) -> Result<(), Failure> {
    let Some(reverse) = reversed.as_ref().filter(|curve| !curve.is_null()) else {
        return Ok(());
    };
    let pair = match orientation {
        Orientation::Forward => (current, reverse),
        _ => (reverse, current),
    };
    BRepBuilder::default().update_edge_pcurves_on_face(edge, pair, face, 0.)
}

fn rescale_reverse(
    reversed: &Option<Handle<dyn Geom2dCurve>>,
    range: (f64, f64),
) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    let Some(curve) = reversed.as_ref().filter(|curve| !curve.is_null()) else {
        return Ok(None);
    };
    let bezier_curve = curve.cast::<Geom2dBezierCurve>();
    let Some(bezier) = bezier_curve.get() else {
        let mut normalized = Handle::default();
        same_range(
            precision::p_confusion(),
            curve,
            range,
            (0., 1.),
            &mut normalized,
        )?;
        return Ok(Some(normalized));
    };
    bezier.segment(range.0, range.1)?;
    Ok(Some(curve.clone()))
}

#[cfg(test)]
mod tests {
    use super::rescale_reverse;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
    use rustycad_tkmath::gp::Axis2d;

    #[test]
    fn non_bezier_reverse_curve_uses_same_range_on_unit_interval() {
        let source: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dLine::new(Axis2d::default())).cast();
        let expected_start = source.value(2.).unwrap();
        let expected_end = source.value(5.).unwrap();
        let result = rescale_reverse(&Some(source), (2., 5.)).unwrap().unwrap();
        let start = result.value(0.).unwrap();
        let end = result.value(1.).unwrap();
        assert_eq!(start, expected_start);
        assert_eq!(end, expected_end);
        assert_eq!(native_range_row(start, end), expected_range_row());
    }

    fn native_range_row(
        start: rustycad_tkmath::gp::Point2,
        end: rustycad_tkmath::gp::Point2,
    ) -> String {
        format!(
            "reverse_line_unit_range {} {} {} {}",
            start.x(),
            start.y(),
            end.x(),
            end.y()
        )
    }

    fn expected_range_row() -> &'static str {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tools/oracle/shape_upgrade_shape_convert_to_bezier.expected"
        ))
        .lines()
        .find(|line| line.starts_with("reverse_line_unit_range "))
        .unwrap()
    }
}
