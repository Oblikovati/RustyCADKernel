// Rust translation of BRepPrimAPI_MakeBox.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrimAPI_MakeBox`: the maker of boxes.
use super::make_one_axis::{add_shell_done, reset_solid};
use crate::brep_prim::{BRepPrimDirection, BRepPrimWedge, standard_frame};
use rustycad_tkbrep::topods::{self, Face, Shell, Solid};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};

/// `std::min(a, b)`: `b` when `b < a`, else `a` (so a NaN in `a` stays).
fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `pmin(p, dx, dy, dz)`: the corner of the box of `p` and the signed sizes with the
/// smallest coordinates.
fn pmin_sizes(p: Point3, dx: f64, dy: f64, dz: f64) -> Point3 {
    let mut corner = p;
    if dx < 0.0 {
        corner.set_x(corner.x() + dx);
    }
    if dy < 0.0 {
        corner.set_y(corner.y() + dy);
    }
    if dz < 0.0 {
        corner.set_z(corner.z() + dz);
    }
    corner
}

/// `pmin(p1, p2)`: the componentwise `std::min` of two points.
fn pmin_points(p1: Point3, p2: Point3) -> Point3 {
    Point3::new(
        std_min(p1.x(), p2.x()),
        std_min(p1.y(), p2.y()),
        std_min(p1.z(), p2.z()),
    )
}

/// The wedge of a box with the corner `corner` and the sizes `|dx|`, `|dy|`, `|dz|` on
/// the frame `gp_Ax2(corner, Z, X)`.
fn corner_wedge(corner: Point3, dx: f64, dy: f64, dz: f64) -> Result<BRepPrimWedge, Failure> {
    Ok(BRepPrimWedge::with_box(
        standard_frame(corner)?,
        dx.abs(),
        dy.abs(),
        dz.abs(),
    ))
}

/// `BRepPrimAPI_MakeBox`: a box parallel to the axes of a frame, from its sizes (signed:
/// a negative size extends toward the negative side), a corner and sizes, two corners,
/// or a frame and sizes (docs/prim-design.md section 4). A flat box has no shell
/// (Standard_DomainError from the wedge).
///
/// ```
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let mut maker = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0)?;
/// assert!(!maker.shape()?.is_null());
/// assert!(maker.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct BRepPrimAPIMakeBox {
    base: BRepBuilderAPIMakeShapeBase,
    /// `myWedge` (protected): the wedge of the box.
    pub my_wedge: BRepPrimWedge,
}

impl BRepPrimAPIMakeBox {
    /// `BRepPrimAPI_MakeBox()`: an empty (degenerated) box; `BRepPrimAPIMakeBox::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The maker of a wedge.
    fn of(my_wedge: BRepPrimWedge) -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_wedge,
        }
    }

    /// `BRepPrimAPI_MakeBox(dx, dy, dz)`: the box from the origin;
    /// `BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0)?`.
    pub fn from_sizes(dx: f64, dy: f64, dz: f64) -> Result<Self, Failure> {
        Self::at_point(Point3::new(0.0, 0.0, 0.0), dx, dy, dz)
    }

    /// `BRepPrimAPI_MakeBox(P, dx, dy, dz)`: the box from `point`;
    /// `BRepPrimAPIMakeBox::at_point(p, 1.0, 2.0, 3.0)?`.
    pub fn at_point(point: Point3, dx: f64, dy: f64, dz: f64) -> Result<Self, Failure> {
        Ok(Self::of(corner_wedge(
            pmin_sizes(point, dx, dy, dz),
            dx,
            dy,
            dz,
        )?))
    }

    /// `BRepPrimAPI_MakeBox(P1, P2)`: the box of two corners;
    /// `BRepPrimAPIMakeBox::from_points(p1, p2)?`.
    pub fn from_points(p1: Point3, p2: Point3) -> Result<Self, Failure> {
        let wedge = corner_wedge(
            pmin_points(p1, p2),
            p2.x() - p1.x(),
            p2.y() - p1.y(),
            p2.z() - p1.z(),
        )?;
        Ok(Self::of(wedge))
    }

    /// `BRepPrimAPI_MakeBox(Axes, dx, dy, dz)`: the box on `axes` (sizes used as given);
    /// `BRepPrimAPIMakeBox::with_axes(axes, 1.0, 2.0, 3.0)`.
    pub fn with_axes(axes: Axis2, dx: f64, dy: f64, dz: f64) -> Self {
        Self::of(BRepPrimWedge::with_box(axes, dx, dy, dz))
    }

    /// `Init(theDX, theDY, theDZ)`: replaces the wedge (the shape and the done flag of a
    /// previous build stay); `maker.init_sizes(1.0, 2.0, 3.0)?`.
    pub fn init_sizes(&mut self, dx: f64, dy: f64, dz: f64) -> Result<(), Failure> {
        self.init_at_point(Point3::new(0.0, 0.0, 0.0), dx, dy, dz)
    }

    /// `Init(thePnt, theDX, theDY, theDZ)`; `maker.init_at_point(p, 1.0, 2.0, 3.0)?`.
    pub fn init_at_point(
        &mut self,
        point: Point3,
        dx: f64,
        dy: f64,
        dz: f64,
    ) -> Result<(), Failure> {
        self.my_wedge = corner_wedge(pmin_sizes(point, dx, dy, dz), dx, dy, dz)?;
        Ok(())
    }

    /// `Init(thePnt1, thePnt2)`; `maker.init_points(p1, p2)?`.
    pub fn init_points(&mut self, p1: Point3, p2: Point3) -> Result<(), Failure> {
        self.my_wedge = corner_wedge(
            pmin_points(p1, p2),
            p2.x() - p1.x(),
            p2.y() - p1.y(),
            p2.z() - p1.z(),
        )?;
        Ok(())
    }

    /// `Init(theAxes, theDX, theDY, theDZ)`; `maker.init_axes(axes, 1.0, 2.0, 3.0)`.
    pub fn init_axes(&mut self, axes: Axis2, dx: f64, dy: f64, dz: f64) {
        self.my_wedge = BRepPrimWedge::with_box(axes, dx, dy, dz);
    }

    /// `Wedge()`: the wedge of the box; `maker.wedge()`.
    pub fn wedge(&mut self) -> &mut BRepPrimWedge {
        &mut self.my_wedge
    }

    /// `Shell()` and `operator TopoDS_Shell()`: the shell of the wedge becomes the
    /// shape, marked done; `maker.shell()?`.
    pub fn shell(&mut self) -> Result<Shell, Failure> {
        self.base.my_shape = self.my_wedge.shell()?.into();
        self.base.set_done();
        topods::shell(&self.base.my_shape)
    }

    /// `Solid()` and `operator TopoDS_Solid()`: a new solid holding the shell of the
    /// wedge becomes the shape, marked done; Standard_TypeMismatch when the shape is a
    /// shell of an earlier `Shell()`; `maker.solid()?`.
    pub fn solid(&mut self) -> Result<Solid, Failure> {
        reset_solid(&mut self.base)?;
        let shell = self.my_wedge.shell()?;
        add_shell_done(&mut self.base, &shell)?;
        topods::solid(&self.base.my_shape)
    }

    /// `BottomFace()`: the ZMin face; `maker.bottom_face()?`.
    pub fn bottom_face(&mut self) -> Result<Face, Failure> {
        self.my_wedge.face(BRepPrimDirection::ZMin)
    }

    /// `BackFace()`: the XMin face; `maker.back_face()?`.
    pub fn back_face(&mut self) -> Result<Face, Failure> {
        self.my_wedge.face(BRepPrimDirection::XMin)
    }

    /// `FrontFace()`: the XMax face; `maker.front_face()?`.
    pub fn front_face(&mut self) -> Result<Face, Failure> {
        self.my_wedge.face(BRepPrimDirection::XMax)
    }

    /// `LeftFace()`: the YMin face; `maker.left_face()?`.
    pub fn left_face(&mut self) -> Result<Face, Failure> {
        self.my_wedge.face(BRepPrimDirection::YMin)
    }

    /// `RightFace()`: the YMax face; `maker.right_face()?`.
    pub fn right_face(&mut self) -> Result<Face, Failure> {
        self.my_wedge.face(BRepPrimDirection::YMax)
    }

    /// `TopFace()`: the ZMax face; `maker.top_face()?`.
    pub fn top_face(&mut self) -> Result<Face, Failure> {
        self.my_wedge.face(BRepPrimDirection::ZMax)
    }
}

impl BRepBuilderAPIMakeShape for BRepPrimAPIMakeBox {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Build()`: `Solid()`.
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        self.solid().map(|_| ())
    }
}
