use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve2d,
    topods::{Edge, Face, Shape, ShapeIterator},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_int::Geom2dCurveTool;
/// Detect a hole from the native sampled pcurve area; `is_hole(&wire,&face)?`.
pub fn is_hole(wire: &Shape, face: &Face) -> Result<bool, Failure> {
    let mut forward = face.clone();
    forward.set_orientation(Orientation::Forward);
    let mut area = 0.;
    for shape in ShapeIterator::new(wire, true, true) {
        let edge = Edge::unchecked(shape);
        if !matches!(
            edge.orientation(),
            Orientation::Forward | Orientation::Reversed
        ) {
            continue;
        }
        let found = BRepTool::curve_on_face(&edge, &forward)?;
        if found.pcurve.is_null() {
            break;
        }
        accumulate_edge_area(&edge, &forward, (found.first, found.last), &mut area)?;
    }
    Ok(area > 0.)
}
fn accumulate_edge_area(
    edge: &Edge,
    face: &Face,
    range: (f64, f64),
    area: &mut f64,
) -> Result<(), Failure> {
    let adaptor = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
    let mut count = Geom2dCurveTool::nb_samples(&adaptor)?;
    if count > 2 {
        count *= 4;
    }
    let mut step = (range.1 - range.0) / f64::from(count - 1);
    let start = if edge.orientation() == Orientation::Reversed {
        step = -step;
        range.1
    } else {
        range.0
    };
    let mut previous = adaptor.d0(start)?;
    for index in 2..=count {
        let next = adaptor.d0(start + f64::from(index - 1) * step)?;
        *area += (previous.y() + next.y()) * (next.x() - previous.x());
        previous = next;
    }
    Ok(())
}
