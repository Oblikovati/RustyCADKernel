// Rust translation of BRepSweep_Rotation.hxx/.cxx (the constructor, the empty shapes,
// the parameters, the topological tests and the accessors) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepSweep_Rotation`: the sweep of a shape around an axis. The faces, p-curves and
//! the solid orientation are in `rotation_geometry.rs`.
use super::num_linear_regular_sweep::{BRepSweepNumLinearRegularSweepBase, SweepAddress};
use super::rotation_geometry as geometry;
use super::sweep_geometry::{
    copied_generating_edge, set_cap_parameters, set_generating_vertex_parameter,
};
use super::translation::is_seam_of_a_face;
use super::trsf::{BRepSweepTrsf, BRepSweepTrsfBase, trsf_set_continuity};
use super::{BRepSweepBuilder, BRepSweepNumLinearRegularSweep};
use crate::sweep::SweepNumShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::Quilt;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis1, Line3, Point3};
use rustycad_tkmath::toploc::Location;
use std::f64::consts::PI;

/// `BRepSweep_Rotation`: the rotation sweep of a shape around an axis by an angle; the
/// shapes on the axis are invariant and shared, the side faces are surfaces of
/// revolution canonized into elementary surfaces (docs/prim-design.md section 3).
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3};
/// use rustycad_tkmath::toploc::Location;
/// use rustycad_tkprim::brep_sweep::{BRepSweepNumLinearRegularSweep, BRepSweepRotation};
/// use rustycad_tkprim::sweep::SweepNumShape;
/// let mut vertex = Vertex::default();
/// BRepBuilder::default().make_vertex_at(&mut vertex, Point3::new(1.0, 0.0, 0.0), 1e-7)?;
/// let axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0)?);
/// let mut t = Transform3::default();
/// t.set_rotation(axis, 1.0)?;
/// let location = Location::new(t);
/// let mut sweep = BRepSweepRotation::new(&vertex, SweepNumShape::new(2, ShapeEnum::Edge), location, axis, 1.0, false)?;
/// assert_eq!(sweep.shape()?.shape_type()?, ShapeEnum::Edge);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepSweepRotation {
    base: BRepSweepNumLinearRegularSweepBase,
    trsf: BRepSweepTrsfBase,
    my_ang: f64,
    my_axe: Axis1,
}

impl BRepSweepRotation {
    /// `BRepSweep_Rotation(S, N, L, A, D, C)`: builds the topology of the rotation of
    /// `shape` around `axis` by `angle` (copying the sub-shapes when `copy`);
    /// Standard_ConstructionError for an angle below Precision::Angular();
    /// `BRepSweepRotation::new(&shape, edge, location, axis, 1.0, false)?`.
    pub fn new(
        shape: &Shape,
        dir: SweepNumShape,
        location: Location,
        axis: Axis1,
        angle: f64,
        copy: bool,
    ) -> Result<Self, Failure> {
        let builder = BRepSweepBuilder::new(BRepBuilder::default());
        let mut rotation = Self {
            base: BRepSweepNumLinearRegularSweepBase::new(builder, shape, dir)?,
            trsf: BRepSweepTrsfBase::new(location, copy),
            my_ang: angle,
            my_axe: axis,
        };
        if angle < precision::angular() {
            return Err(rotation_error(
                "Constructor",
                &format!("angle {angle}; expected at least Precision::Angular()"),
            ));
        }
        rotation.init()?;
        Ok(rotation)
    }

    /// `Angle()`: the angle of the rotation; `rotation.angle()`.
    pub fn angle(&self) -> f64 {
        self.my_ang
    }

    /// `Axe()`: the axis of the rotation; `rotation.axe()`.
    pub fn axe(&self) -> Axis1 {
        self.my_axe
    }

    /// The location of the rotation (`myLocation`).
    pub(crate) fn trsf_location(&self) -> &Location {
        &self.trsf.my_location
    }

    /// The BRep builder of the sweep.
    pub(crate) fn brep_builder(&self) -> BRepBuilder {
        *self.base.my_builder.builder()
    }

    /// The axis line (`gp_Lin(myAxe.Location(), myAxe.Direction())`).
    fn axis_line(&self) -> Line3 {
        Line3::new(self.my_axe.location(), self.my_axe.direction())
    }

    /// The copy of an invariant shape at the first vertex of an open rotation is also
    /// its shape at the last vertex (index 3), shared by the two.
    fn share_invariant_copy(
        &mut self,
        gen_s: &Shape,
        dir_v: &SweepNumShape,
        new_shape: &Shape,
    ) -> Result<(), Failure> {
        if dir_v.index() == 1
            && self.is_invariant(gen_s)?
            && self.base.my_dir_shape_tool.nb_shapes() == 3
        {
            let i = self.base.my_gen_shape_tool.index(gen_s);
            self.base.set_built(i, 3, true)?;
            self.base.set_cell(i, 3, new_shape.clone())?;
        }
        Ok(())
    }

    /// Whether the edge `gen_s` stays on the axis: a line, B-spline or Bezier curve
    /// whose vertices are invariant and whose poles are within the vertex tolerance of
    /// the axis.
    fn is_invariant_edge(&self, gen_s: &Shape) -> Result<bool, Failure> {
        let edge = topods::edge(gen_s)?;
        let curve = BRepAdaptorCurve::with_edge(&edge)?;
        let kind = curve.get_type()?;
        if !matches!(
            kind,
            CurveType::Line | CurveType::BSplineCurve | CurveType::BezierCurve
        ) {
            return Ok(false);
        }
        let (v1, v2) = topexp::vertices(&edge, false);
        if !(self.is_invariant(&v1)? && self.is_invariant(&v2)?) {
            return Ok(false);
        }
        if kind == CurveType::Line {
            return Ok(true);
        }
        let tol = BRepTool::vertex_tolerance(&v1)?.max(BRepTool::vertex_tolerance(&v2)?);
        let line = self.axis_line();
        for pole in geometry::curve_poles(&curve, kind)? {
            if line.distance(pole) > tol {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// The Standard_ConstructionError of the rotation.
pub(crate) fn rotation_error(member: &str, detail: &str) -> Failure {
    Failure::with_message(
        "Standard_ConstructionError",
        &format!("BRepSweep_Rotation::{member}: {detail}"),
    )
}

impl BRepSweepTrsf for BRepSweepRotation {
    fn trsf_base(&self) -> &BRepSweepTrsfBase {
        &self.trsf
    }
}

impl BRepSweepNumLinearRegularSweep for BRepSweepRotation {
    fn sweep_base(&self) -> &BRepSweepNumLinearRegularSweepBase {
        &self.base
    }

    fn sweep_base_mut(&mut self) -> &mut BRepSweepNumLinearRegularSweepBase {
        &mut self.base
    }

    /// `MakeEmptyVertex`: the generating point, rotated at the last vertex, with the
    /// generating tolerance (jgv 1.10.01, buc61005); only with copy.
    fn make_empty_vertex(
        &mut self,
        gen_v: &Shape,
        dir_v: &SweepNumShape,
    ) -> Result<Shape, Failure> {
        if !self.trsf.my_copy {
            return Err(rotation_error(
                "MakeEmptyVertex",
                "called without copy; expected a copying rotation",
            ));
        }
        let vertex_in = topods::vertex(gen_v)?;
        let mut point: Point3 = BRepTool::pnt(&vertex_in)?;
        if dir_v.index() == 2 {
            point.transform(self.trsf.my_location.transformation());
        }
        let mut vertex = Vertex::default();
        self.brep_builder().make_vertex_at(
            &mut vertex,
            point,
            BRepTool::vertex_tolerance(&vertex_in)?,
        )?;
        let vertex: Shape = vertex.into();
        self.share_invariant_copy(gen_v, dir_v, &vertex)?;
        Ok(vertex)
    }

    /// `MakeEmptyDirectingEdge`: the circle of the generating point around the axis, or
    /// a degenerated edge on a null circle for a point on the axis.
    fn make_empty_directing_edge(
        &mut self,
        gen_v: &Shape,
        _dir_e: &SweepNumShape,
    ) -> Result<Shape, Failure> {
        geometry::directing_circle_edge(self.brep_builder(), self.my_axe, &topods::vertex(gen_v)?)
    }

    /// `MakeEmptyGeneratingEdge`: as the translation's, without the copy check (also
    /// called when the meridian touches the axis).
    fn make_empty_generating_edge(
        &mut self,
        gen_e: &Shape,
        dir_v: &SweepNumShape,
    ) -> Result<Shape, Failure> {
        let location = self.trsf.my_location.clone();
        let edge = copied_generating_edge(self.brep_builder(), gen_e, dir_v, &location)?;
        self.share_invariant_copy(gen_e, dir_v, &edge)?;
        Ok(edge)
    }

    fn set_parameters(
        &mut self,
        new_face: &Shape,
        new_vertex: &Shape,
        (gen_f, gen_v, _): SweepAddress<'_>,
    ) -> Result<(), Failure> {
        set_cap_parameters(self.brep_builder(), new_face, new_vertex, gen_f, gen_v)
    }

    /// `SetDirectingParameter`: 0 (FORWARD) at the first vertex, the angle (REVERSED) at
    /// the last.
    fn set_directing_parameter(
        &mut self,
        new_edge: &Shape,
        new_vertex: &Shape,
        _gen_v: &Shape,
        _dir_e: &SweepNumShape,
        dir_v: &SweepNumShape,
    ) -> Result<(), Failure> {
        let (param, ori) = if dir_v.index() == 2 {
            (self.my_ang, Orientation::Reversed)
        } else {
            (0.0, Orientation::Forward)
        };
        let vertex = topods::vertex(&new_vertex.oriented(ori))?;
        self.brep_builder().update_vertex_on_edge(
            &vertex,
            param,
            &topods::edge(new_edge)?,
            precision::p_confusion(),
        )
    }

    fn set_generating_parameter(
        &mut self,
        new_edge: &Shape,
        new_vertex: &Shape,
        (gen_e, gen_v, _): SweepAddress<'_>,
    ) -> Result<(), Failure> {
        set_generating_vertex_parameter(self.brep_builder(), new_edge, new_vertex, gen_e, gen_v)
    }

    fn make_empty_face(&mut self, gen_s: &Shape, dir_s: &SweepNumShape) -> Result<Shape, Failure> {
        geometry::make_empty_face(self, gen_s, dir_s)
    }

    fn set_pcurve(
        &mut self,
        new_face: &Shape,
        new_edge: &Shape,
        (gen_f, gen_e, _): SweepAddress<'_>,
        orien: Orientation,
    ) -> Result<(), Failure> {
        geometry::set_cap_pcurve(
            self.brep_builder(),
            new_face,
            new_edge,
            (gen_f, gen_e),
            orien,
        )
    }

    fn set_generating_pcurve(
        &mut self,
        new_face: &Shape,
        new_edge: &Shape,
        _gen_e: &Shape,
        (_, dir_v): (&SweepNumShape, &SweepNumShape),
        orien: Orientation,
    ) -> Result<(), Failure> {
        geometry::set_generating_pcurve(self, new_face, new_edge, dir_v, orien)
    }

    fn set_directing_pcurve(
        &mut self,
        new_face: &Shape,
        new_edge: &Shape,
        (gen_e, gen_v, _): SweepAddress<'_>,
        orien: Orientation,
    ) -> Result<(), Failure> {
        geometry::set_directing_pcurve(self, new_face, new_edge, (gen_e, gen_v), orien)
    }

    fn direct_solid(
        &mut self,
        gen_s: &Shape,
        _dir_s: &SweepNumShape,
    ) -> Result<Orientation, Failure> {
        geometry::direct_solid(self.my_axe, gen_s)
    }

    /// `GGDShapeIsToAdd`: on a planar side face, the new edge of a vertex is added only
    /// when the vertex is not on the axis.
    fn ggd_shape_is_to_add(
        &self,
        new_shape: &Shape,
        new_sub_shape: &Shape,
        (gen_s, sub_gen_s, dir_s): SweepAddress<'_>,
    ) -> Result<bool, Failure> {
        if !geometry::is_side_face_of_vertex(new_shape, new_sub_shape, gen_s, sub_gen_s, dir_s)? {
            return Ok(true);
        }
        if geometry::raw_surface_type(new_shape)? == rustycad_tkmath::geom_abs::SurfaceType::Plane {
            return Ok(!self.is_invariant(sub_gen_s)?);
        }
        Ok(true)
    }

    /// `GDDShapeIsToAdd`: the caps of a solid and the directing edges of a planar side
    /// face exist only when the rotation is not complete.
    fn gdd_shape_is_to_add(
        &self,
        new_shape: &Shape,
        new_sub_shape: &Shape,
        gen_s: &Shape,
        (dir_s, sub_dir_s): (&SweepNumShape, &SweepNumShape),
    ) -> Result<bool, Failure> {
        let partial = (self.my_ang - 2.0 * PI).abs() > precision::angular();
        let dirs = |kind: ShapeEnum| dir_s.kind() == ShapeEnum::Edge && sub_dir_s.kind() == kind;
        let shapes = [new_shape, new_sub_shape, gen_s];
        if geometry::kinds_are(shapes, [ShapeEnum::Solid, ShapeEnum::Face, ShapeEnum::Face])?
            && dirs(ShapeEnum::Vertex)
        {
            return Ok(partial);
        }
        if geometry::kinds_are(shapes, [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Edge])?
            && dirs(ShapeEnum::Vertex)
        {
            let planar = geometry::raw_surface_type(new_shape)?
                == rustycad_tkmath::geom_abs::SurfaceType::Plane;
            return Ok(!planar || partial);
        }
        Ok(true)
    }

    /// `SeparatedWires`: only a planar side face of a complete rotation (a disc with a
    /// hole) is made of independent wires.
    fn separated_wires(
        &self,
        new_shape: &Shape,
        new_sub_shape: &Shape,
        (gen_s, sub_gen_s, dir_s): SweepAddress<'_>,
    ) -> Result<bool, Failure> {
        if !geometry::is_side_face_of_vertex(new_shape, new_sub_shape, gen_s, sub_gen_s, dir_s)? {
            return Ok(false);
        }
        if geometry::raw_surface_type(new_shape)? == rustycad_tkmath::geom_abs::SurfaceType::Plane {
            return Ok((self.my_ang - 2.0 * PI).abs() <= precision::angular());
        }
        Ok(false)
    }

    /// `SplitShell`: the shells of the new shell, through BRepTools_Quilt.
    fn split_shell(&self, new_shape: &Shape) -> Result<Shape, Failure> {
        let mut quilt = Quilt::new();
        quilt.add(new_shape)?;
        quilt.shells()
    }

    fn set_continuity(&mut self, gen_s: &Shape, dir_s: &SweepNumShape) -> Result<(), Failure> {
        trsf_set_continuity(self, gen_s, dir_s)
    }

    /// `HasShape`: along the edge, a degenerated edge, an edge without 3D curve, an
    /// invariant edge or a seam of a generating face sweeps nothing.
    fn has_shape(&self, gen_s: &Shape, dir_s: &SweepNumShape) -> Result<bool, Failure> {
        if dir_s.kind() != ShapeEnum::Edge || gen_s.shape_type()? != ShapeEnum::Edge {
            return Ok(true);
        }
        let edge = topods::edge(gen_s)?;
        if BRepTool::degenerated(&edge)? || BRepTool::curve_with_location(&edge)?.curve.is_null() {
            return Ok(false);
        }
        if self.is_invariant(gen_s)? {
            return Ok(false);
        }
        Ok(!is_seam_of_a_face(&self.base.my_gen_shape, &edge)?)
    }

    /// `IsInvariant`: a vertex within its tolerance of the axis, or an edge on the axis.
    fn is_invariant(&self, gen_s: &Shape) -> Result<bool, Failure> {
        match gen_s.shape_type()? {
            ShapeEnum::Edge => self.is_invariant_edge(gen_s),
            ShapeEnum::Vertex => {
                let vertex = topods::vertex(gen_s)?;
                let point = BRepTool::pnt(&vertex)?;
                Ok(self.axis_line().distance(point) <= BRepTool::vertex_tolerance(&vertex)?)
            }
            _ => Ok(false),
        }
    }
}
