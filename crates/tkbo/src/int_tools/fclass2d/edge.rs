use super::*;
use rustycad_tkgeomalgo::geom2d_int::Geom2dCurveTool;
pub(super) struct EdgeSampling {
    pub(super) curve: BRepAdaptorCurve2d,
    pub(super) spatial: BRepAdaptorCurve,
    pub(super) degenerated: bool,
    pub(super) parameters: Vec<f64>,
}
impl EdgeSampling {
    pub(super) fn new(edge: &Edge, face: &Face) -> Result<Option<Self>, Failure> {
        let found = BRepTool::curve_on_face(edge, face)?;
        if found.pcurve.is_null() {
            return Ok(None);
        }
        let curve = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
        let (spatial, degenerated) = spatial_curve(edge, face, found.first, found.last)?;
        BRepTool::edge_tolerance(edge)?;
        let parameters = oriented_parameters(edge, &curve, found.first, found.last)?;
        Ok(Some(Self {
            curve,
            spatial,
            degenerated,
            parameters,
        }))
    }
    pub(super) fn is_repeated(
        &self,
        index: usize,
        p: Point3,
        previous: Option<Point3>,
    ) -> Result<bool, Failure> {
        if self.degenerated || index == 0 {
            return Ok(false);
        }
        let Some(previous) = previous else {
            return Ok(false);
        };
        if p.square_distance(&previous) >= precision::square_confusion() {
            return Ok(false);
        }
        let middle = self
            .spatial
            .value(0.5 * (self.parameters[index] + self.parameters[index - 1]))?;
        Ok(p.square_distance(&middle) < precision::square_confusion())
    }
}
fn edge_parameters(first: f64, last: f64, mut count: i32) -> Vec<f64> {
    if count == 2 {
        return vec![first, first + 0.0025 * (last - first), last];
    }
    if count > 2 {
        count *= 4;
    }
    let step = (last - first) / f64::from(count - 1);
    let mut values = vec![first];
    for i in 1..count - 1 {
        values.push(first + f64::from(i) * step);
    }
    values.push(last);
    values
}
fn vertex_degenerate(edge: &Edge, face: &Face) -> Result<bool, Failure> {
    let (first, last) = topexp::vertices(edge, false);
    if !first.is_null() {
        BRepTool::vertex_tolerance(&first)?;
    }
    if !last.is_null() {
        BRepTool::vertex_tolerance(&last)?;
    }
    Ok(BRepTool::degenerated(edge)?
        || BRepTool::is_closed_on_face(edge, face)?
        || first.is_null()
        || last.is_null())
}
fn curve_degenerate(curve: &BRepAdaptorCurve, first: f64, last: f64) -> Result<bool, Failure> {
    let middle = curve.value(0.5 * (first + last))?;
    let span = last - first;
    let tolerance = 0.25 * precision::confusion() * precision::confusion();
    for i in 0..=10 {
        if middle.square_distance(&curve.value(first + f64::from(i) * span / 10.)?) > tolerance {
            return Ok(false);
        }
    }
    Ok(true)
}

fn spatial_curve(
    edge: &Edge,
    face: &Face,
    first: f64,
    last: f64,
) -> Result<(BRepAdaptorCurve, bool), Failure> {
    let spatial = BRepAdaptorCurve::default();
    let mut degenerated = vertex_degenerate(edge, face)?;
    if !degenerated {
        spatial.initialize_on_face(edge, face)?;
        degenerated = curve_degenerate(&spatial, first, last)?;
    }
    Ok((spatial, degenerated))
}

fn oriented_parameters(
    edge: &Edge,
    curve: &BRepAdaptorCurve2d,
    first: f64,
    last: f64,
) -> Result<Vec<f64>, Failure> {
    let count = Geom2dCurveTool::nb_samples(curve)?;
    let (first, last) = if edge.orientation() == Orientation::Forward {
        (first, last)
    } else {
        (last, first)
    };
    Ok(edge_parameters(first, last, count))
}
