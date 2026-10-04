use rustycad_tkbrep::{
    brep::{
        BRepBuilder, BRepCurveRepresentation, BRepGCurve, BRepTool,
        access::{edge_tshape, vertex_tshape},
    },
    topexp::Explorer,
    topods::{Edge, Shape, ShapeIterator, Vertex},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    geom::GeomCurve,
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::{gp::Point3, toploc::Location};
struct VertexCorrection<'a> {
    vertex: &'a Vertex,
    point: Point3,
    squared_tolerance: f64,
    margin: f64,
    maximum: f64,
    avoided: &'a IndexedMapOfShape,
}
/// Correct vertex discrepancies on spatial curve representations; `correct_point_on_curve(&shape,&avoided,0.1,false)?`.
pub fn correct_point_on_curve(
    shape: &Shape,
    avoided: &IndexedMapOfShape,
    maximum: f64,
    _run_parallel: bool,
) -> Result<(), Failure> {
    for shape in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        check_edge(&Edge::unchecked(shape), avoided, maximum)?;
    }
    Ok(())
}
fn check_edge(edge: &Edge, avoided: &IndexedMapOfShape, maximum: f64) -> Result<(), Failure> {
    let mut edge = edge.clone();
    edge.set_orientation(Orientation::Forward);
    let curves = edge_tshape(&edge, "CorrectPointOnCurve")?.curves();
    for shape in ShapeIterator::new(&edge, true, true) {
        let vertex = Vertex::unchecked(shape);
        let correction =
            VertexCorrection::new(&vertex, BRepTool::edge_tolerance(&edge)?, maximum, avoided)?;
        for curve in &curves {
            correction.check_representation(curve, edge.location())?;
        }
    }
    Ok(())
}
impl<'a> VertexCorrection<'a> {
    fn new(
        vertex: &'a Vertex,
        edge_tolerance: f64,
        maximum: f64,
        avoided: &'a IndexedMapOfShape,
    ) -> Result<Self, Failure> {
        let tolerance = BRepTool::vertex_tolerance(vertex)?;
        let tolerance = if tolerance < edge_tolerance {
            edge_tolerance
        } else {
            tolerance
        };
        Ok(Self {
            vertex,
            point: vertex_tshape(vertex, "CorrectPointOnCurve")?.pnt(),
            squared_tolerance: tolerance * tolerance,
            margin: 0.1 * tolerance,
            maximum,
            avoided,
        })
    }
    fn check_representation(
        &self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        edge_location: &Location,
    ) -> Result<(), Failure> {
        if !representation.is_curve3d() {
            return Ok(());
        }
        let curve = representation.curve3d()?;
        if curve.is_null() {
            return Ok(());
        }
        let relative = edge_location
            .multiplied(&representation.location())?
            .predivided(self.vertex.location())?;
        self.check_stored_points(&curve, &relative)?;
        let parametric: Handle<dyn BRepGCurve> = representation.cast();
        let parameter = match self.vertex.orientation() {
            Orientation::Forward => parametric.first(),
            Orientation::Reversed => parametric.last(),
            _ => return Ok(()),
        };
        self.check_parameter(&curve, &relative, parameter)
    }
    fn check_stored_points(
        &self,
        curve: &Handle<dyn GeomCurve>,
        relative: &Location,
    ) -> Result<(), Failure> {
        for point in vertex_tshape(self.vertex, "CorrectPointOnCurve")?.points() {
            if point.is_point_on_curve_at(curve, relative) {
                self.check_parameter(curve, relative, point.parameter())?;
            }
        }
        Ok(())
    }
    fn check_parameter(
        &self,
        curve: &Handle<dyn GeomCurve>,
        relative: &Location,
        parameter: f64,
    ) -> Result<(), Failure> {
        let point = curve
            .value(parameter)?
            .transformed(relative.transformation());
        let squared = self.point.square_distance(&point);
        if squared <= self.squared_tolerance {
            return Ok(());
        }
        let needed = squared.sqrt() + self.margin;
        if needed < self.maximum && !self.avoided.contains(self.vertex) {
            BRepBuilder::default().update_vertex_tolerance(self.vertex, needed)?;
        }
        Ok(())
    }
}
