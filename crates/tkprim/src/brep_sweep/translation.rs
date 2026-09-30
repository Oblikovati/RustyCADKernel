// Rust translation of BRepSweep_Translation.hxx/.cxx (the constructor, the empty
// shapes, the parameters and the topological tests) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepSweep_Translation`: the sweep of a shape along a vector. The p-curves and the
//! solid orientation are in `translation_geometry.rs`.
use super::num_linear_regular_sweep::{BRepSweepNumLinearRegularSweepBase, SweepAddress};
use super::translation_geometry as geometry;
use super::trsf::{BRepSweepTrsf, BRepSweepTrsfBase, trsf_set_continuity};
use super::{BRepSweepBuilder, BRepSweepNumLinearRegularSweep};
use crate::sweep::SweepNumShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Line3, Vector3};
use rustycad_tkmath::toploc::Location;

/// The Standard_ConstructionError of the translation.
pub(crate) fn construction_error(member: &str, detail: &str) -> Failure {
    Failure::with_message(
        "Standard_ConstructionError",
        &format!("BRepSweep_Translation::{member}: {detail}"),
    )
}

/// `BRepSweep_Translation`: the translation sweep of a shape by a vector; the shapes
/// at the last directing vertex are the generating shapes moved by the location (or
/// copies of them) and the side faces are extrusions, canonized into planes and
/// cylinders on request (docs/prim-design.md section 3).
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
/// use rustycad_tkmath::toploc::Location;
/// use rustycad_tkprim::brep_sweep::{BRepSweepNumLinearRegularSweep, BRepSweepTranslation};
/// use rustycad_tkprim::sweep::SweepNumShape;
/// let mut vertex = Vertex::default();
/// BRepBuilder::default().make_vertex_at(&mut vertex, Point3::new(0.0, 0.0, 0.0), 1e-7)?;
/// let v = Vector3::new(0.0, 0.0, 1.0);
/// let mut t = Transform3::default();
/// t.set_translation(v);
/// let location = Location::new(t);
/// let mut sweep = BRepSweepTranslation::new(&vertex, SweepNumShape::new(2, ShapeEnum::Edge), location, v, false, true)?;
/// assert_eq!(sweep.shape()?.shape_type()?, ShapeEnum::Edge);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepSweepTranslation {
    base: BRepSweepNumLinearRegularSweepBase,
    trsf: BRepSweepTrsfBase,
    my_vec: Vector3,
    my_canonize: bool,
}

impl BRepSweepTranslation {
    /// `BRepSweep_Translation(S, N, L, V, C, Canonize)`: builds the topology of the
    /// translation of `shape` by `vector` (copying the sub-shapes when `copy`);
    /// Standard_ConstructionError for a vector shorter than Precision::Confusion();
    /// `BRepSweepTranslation::new(&shape, edge, location, v, false, true)?`.
    pub fn new(
        shape: &Shape,
        dir: SweepNumShape,
        location: Location,
        vector: Vector3,
        copy: bool,
        canonize: bool,
    ) -> Result<Self, Failure> {
        let builder = BRepSweepBuilder::new(BRepBuilder::default());
        let mut translation = Self {
            base: BRepSweepNumLinearRegularSweepBase::new(builder, shape, dir)?,
            trsf: BRepSweepTrsfBase::new(location, copy),
            my_vec: vector,
            my_canonize: canonize,
        };
        if vector.magnitude() < precision::confusion() {
            return Err(construction_error(
                "Constructor",
                &format!(
                    "vector of magnitude {}; expected at least Precision::Confusion()",
                    vector.magnitude()
                ),
            ));
        }
        translation.init()?;
        Ok(translation)
    }

    /// `Vec()`: the vector of the prism (unit for an infinite prism);
    /// `translation.vec()`.
    pub fn vec(&self) -> Vector3 {
        self.my_vec
    }

    /// The canonize flag of the side surfaces.
    pub(crate) fn canonize(&self) -> bool {
        self.my_canonize
    }

    /// The BRep builder of the sweep.
    pub(crate) fn brep_builder(&self) -> BRepBuilder {
        *self.base.my_builder.builder()
    }

    /// Standard_ConstructionError from `member` when the sweep does not copy.
    fn check_copy(&self, member: &str) -> Result<(), Failure> {
        if !self.trsf.my_copy {
            return Err(construction_error(
                member,
                "called without copy; expected a copying translation",
            ));
        }
        Ok(())
    }
}

impl BRepSweepTrsf for BRepSweepTranslation {
    fn trsf_base(&self) -> &BRepSweepTrsfBase {
        &self.trsf
    }
}

impl BRepSweepNumLinearRegularSweep for BRepSweepTranslation {
    fn sweep_base(&self) -> &BRepSweepNumLinearRegularSweepBase {
        &self.base
    }

    fn sweep_base_mut(&mut self) -> &mut BRepSweepNumLinearRegularSweepBase {
        &mut self.base
    }

    /// `MakeEmptyVertex`: the generating point, moved at the last directing vertex,
    /// with the generating tolerance (jgv 5.10.01, buc61008); only with copy.
    fn make_empty_vertex(
        &mut self,
        gen_v: &Shape,
        dir_v: &SweepNumShape,
    ) -> Result<Shape, Failure> {
        self.check_copy("MakeEmptyVertex")?;
        let gen_v = topods::vertex(gen_v)?;
        let mut point = BRepTool::pnt(&gen_v)?;
        if dir_v.index() == 2 {
            point.transform(self.trsf.my_location.transformation());
        }
        let mut vertex = Vertex::default();
        self.brep_builder().make_vertex_at(
            &mut vertex,
            point,
            BRepTool::vertex_tolerance(&gen_v)?,
        )?;
        Ok(vertex.into())
    }

    /// `MakeEmptyDirectingEdge`: the line from the generating point along the vector.
    fn make_empty_directing_edge(
        &mut self,
        gen_v: &Shape,
        _dir_e: &SweepNumShape,
    ) -> Result<Shape, Failure> {
        let gen_v = topods::vertex(gen_v)?;
        let point = BRepTool::pnt(&gen_v)?;
        let line = Line3::new(
            point,
            rustycad_tkmath::gp::Direction3::from_vector(self.my_vec)?,
        );
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(line)).cast();
        let mut edge = Edge::default();
        self.brep_builder().make_edge_with_curve(
            &mut edge,
            &curve,
            BRepTool::vertex_tolerance(&gen_v)?,
        )?;
        Ok(edge.into())
    }

    /// `MakeEmptyGeneratingEdge`: a degenerated edge, or a copy of the located curve of
    /// the generating edge, moved at the last directing vertex; only with copy.
    fn make_empty_generating_edge(
        &mut self,
        gen_e: &Shape,
        dir_v: &SweepNumShape,
    ) -> Result<Shape, Failure> {
        // The native message names MakeEmptyVertex here.
        self.check_copy("MakeEmptyVertex")?;
        let location = self.trsf.my_location.clone();
        geometry::copied_generating_edge(self.brep_builder(), gen_e, dir_v, &location)
    }

    /// `SetParameters`: the UV of the generating vertex on the generating face.
    fn set_parameters(
        &mut self,
        new_face: &Shape,
        new_vertex: &Shape,
        (gen_f, gen_v, _): SweepAddress<'_>,
    ) -> Result<(), Failure> {
        geometry::set_cap_parameters(self.brep_builder(), new_face, new_vertex, gen_f, gen_v)
    }

    /// `SetDirectingParameter`: 0 at the first vertex, the vector length at the last.
    fn set_directing_parameter(
        &mut self,
        new_edge: &Shape,
        new_vertex: &Shape,
        _gen_v: &Shape,
        _dir_e: &SweepNumShape,
        dir_v: &SweepNumShape,
    ) -> Result<(), Failure> {
        let param = if dir_v.index() == 2 {
            self.my_vec.magnitude()
        } else {
            0.0
        };
        let (vertex, edge) = (topods::vertex(new_vertex)?, topods::edge(new_edge)?);
        self.brep_builder()
            .update_vertex_on_edge(&vertex, param, &edge, precision::p_confusion())
    }

    /// `SetGeneratingParameter`: the parameter of the generating vertex on the
    /// generating edge, with the orientation of the generating vertex.
    fn set_generating_parameter(
        &mut self,
        new_edge: &Shape,
        new_vertex: &Shape,
        (gen_e, gen_v, _): SweepAddress<'_>,
    ) -> Result<(), Failure> {
        geometry::set_generating_vertex_parameter(
            self.brep_builder(),
            new_edge,
            new_vertex,
            gen_e,
            gen_v,
        )
    }

    fn make_empty_face(&mut self, gen_s: &Shape, dir_s: &SweepNumShape) -> Result<Shape, Failure> {
        geometry::make_empty_face(self, gen_s, dir_s)
    }

    fn set_pcurve(
        &mut self,
        new_face: &Shape,
        new_edge: &Shape,
        (gen_f, gen_e, _): SweepAddress<'_>,
        _orien: Orientation,
    ) -> Result<(), Failure> {
        geometry::set_cap_pcurve(self.brep_builder(), new_face, new_edge, gen_f, gen_e)
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
        geometry::set_directing_pcurve(
            self.brep_builder(),
            new_face,
            new_edge,
            (gen_e, gen_v),
            orien,
        )
    }

    fn direct_solid(
        &mut self,
        gen_s: &Shape,
        _dir_s: &SweepNumShape,
    ) -> Result<Orientation, Failure> {
        geometry::direct_solid(self.my_vec, gen_s)
    }

    /// `GGDShapeIsToAdd`: always true.
    fn ggd_shape_is_to_add(
        &self,
        _new_shape: &Shape,
        _new_sub_shape: &Shape,
        _address: SweepAddress<'_>,
    ) -> Result<bool, Failure> {
        Ok(true)
    }

    /// `GDDShapeIsToAdd`: always true.
    fn gdd_shape_is_to_add(
        &self,
        _: &Shape,
        _: &Shape,
        _: &Shape,
        _: (&SweepNumShape, &SweepNumShape),
    ) -> Result<bool, Failure> {
        Ok(true)
    }

    /// `SeparatedWires`: always false.
    fn separated_wires(
        &self,
        _new_shape: &Shape,
        _new_sub_shape: &Shape,
        _address: SweepAddress<'_>,
    ) -> Result<bool, Failure> {
        Ok(false)
    }

    fn set_continuity(&mut self, gen_s: &Shape, dir_s: &SweepNumShape) -> Result<(), Failure> {
        trsf_set_continuity(self, gen_s, dir_s)
    }

    /// `HasShape`: along the edge, a degenerated generating edge or a seam of one of
    /// the generating faces (modified by NIZHNY-EAP Fri Dec 24 1999) sweeps nothing.
    fn has_shape(&self, gen_s: &Shape, dir_s: &SweepNumShape) -> Result<bool, Failure> {
        if self.base.my_dir_shape_tool.kind(dir_s) != ShapeEnum::Edge
            || gen_s.shape_type()? != ShapeEnum::Edge
        {
            return Ok(true);
        }
        let edge = topods::edge(gen_s)?;
        if BRepTool::degenerated(&edge)? {
            return Ok(false);
        }
        Ok(!is_seam_of_a_face(&self.base.my_gen_shape, &edge)?)
    }

    /// `IsInvariant`: always false, a translation moves everything.
    fn is_invariant(&self, _gen_s: &Shape) -> Result<bool, Failure> {
        Ok(false)
    }
}

/// Whether `edge` is really closed (a seam) on one of the faces of `gen_shape`.
pub(crate) fn is_seam_of_a_face(gen_shape: &Shape, edge: &Edge) -> Result<bool, Failure> {
    let mut faces = Explorer::new(gen_shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        let face = topods::face(faces.current())?;
        if brep_tools::is_really_closed(edge, &face)? {
            return Ok(true);
        }
        faces.next()?;
    }
    Ok(false)
}
