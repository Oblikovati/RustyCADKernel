//! HLRAppli_ReflectLines.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::hlr_algo::Projector;
use crate::hlr_brep::{Algo, HLRToShape, TypeOfResultingEdge};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Transform3};
use rustycad_tktopalgo::brep_lib;

/// HLRAppli_ReflectLines: the reflect lines of a shape seen along a direction: the
/// exact hidden line algorithm on the shape, and its edges of a type as 3D edges.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkhlr::hlr_appli::ReflectLines;
/// let mut lines = ReflectLines::new(&Shape::default())?;
/// lines.set_axes(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0)?;
/// lines.perform()?;
/// assert!(lines.get_result()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct ReflectLines {
    projector: Projector,
    hlr_algo: Handle<Algo>,
    shape: Shape,
}

impl ReflectLines {
    /// HLRAppli_ReflectLines(aShape); `ReflectLines::new(&shape)?`.
    pub fn new(a_shape: &Shape) -> Result<Self, Failure> {
        Ok(Self {
            projector: Projector::new()?,
            hlr_algo: Handle::default(),
            shape: a_shape.clone(),
        })
    }

    /// SetAxes(Nx, Ny, Nz, XAt, YAt, ZAt, XUp, YUp, ZUp): the parallel projection
    /// along `N` through the point `At`, with `Up` as the view Y axis;
    /// `lines.set_axes(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn set_axes(
        &mut self,
        nx: f64,
        ny: f64,
        nz: f64,
        x_at: f64,
        y_at: f64,
        z_at: f64,
        x_up: f64,
        y_up: f64,
        z_up: f64,
    ) -> Result<(), Failure> {
        let is_perspective = false;
        let a_focus = 1.;
        let at = Point3::new(x_at, y_at, z_at);
        let zpers = Direction3::new(nx, ny, nz)?;
        let ypers = Direction3::new(x_up, y_up, z_up)?;
        let xpers = ypers.crossed(zpers)?;
        let axe = Axis3::new(at, zpers, xpers)?;
        let mut t = Transform3::default();
        t.set_transformation_to(axe);
        self.projector = Projector::with_transformation(t, is_perspective, a_focus)?;
        Ok(())
    }

    /// Perform(): the hidden line algorithm on the shape; `lines.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        let algo = Handle::new(Algo::new()?);
        self.hlr_algo = algo.clone();
        let a = algo
            .get()
            .ok_or_else(|| Failure::new("HLRAppli_ReflectLines::Perform: no algorithm"))?;
        a.add(&self.shape, 0);
        a.set_projector(&self.projector);
        a.update()?;
        a.hide()
    }

    /// GetCompoundOf3dEdges(type, visible, In3d): the edges of the type, made same
    /// parameter; `lines.get_compound_of_3d_edges(TypeOfResultingEdge::OutLine, true, true)?`.
    pub fn get_compound_of_3d_edges(
        &self,
        ty: TypeOfResultingEdge,
        visible: bool,
        in_3d: bool,
    ) -> Result<Shape, Failure> {
        let a_hlr_to_shape = HLRToShape::new(&self.hlr_algo);
        let the_compound = a_hlr_to_shape.compound_of_edges(ty, visible, in_3d)?;
        brep_lib::same_parameter_shape(&the_compound, precision::p_confusion(), false)?;
        Ok(the_compound)
    }

    /// GetResult(): the visible outlines as 3D edges; `lines.get_result()?`.
    pub fn get_result(&self) -> Result<Shape, Failure> {
        self.get_compound_of_3d_edges(TypeOfResultingEdge::OutLine, true, true)
    }
}
