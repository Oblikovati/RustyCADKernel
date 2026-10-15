// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_FindEdgesInFace.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_FindEdgesInFace`: the edges of a shape lying in a planar or cylindrical face:
//! its own edges, and the lines and circles on its surface whose ends and middle are
//! not outside it.
use crate::errors::{not_done, null_object};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane,
    GeomRectangularTrimmedSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis3, Cylinder3, Plane3, Point2, Point3};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;

/// The supported surfaces of the face.
enum FaceSurface {
    Plane(Plane3),
    Cylinder(Cylinder3),
}

impl FaceSurface {
    /// The plane or the cylinder of the (located) surface of `f`, a rectangular trimmed
    /// surface standing for its basis; None for the other surfaces (pour le moment).
    fn of(f: &Face) -> Result<Option<Self>, Failure> {
        let mut s = BRepTool::surface(f)?;
        let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
        if let Some(trimmed) = trimmed.get() {
            s = trimmed.basis_surface();
        }
        if let Some(plane) = s.cast::<GeomPlane>().get() {
            return Ok(Some(Self::Plane(plane.pln())));
        }
        if let Some(cylinder) = s.cast::<GeomCylindricalSurface>().get() {
            return Ok(Some(Self::Cylinder(cylinder.cylinder()?)));
        }
        Ok(None)
    }

    /// Whether the line or circle `c` lies on the surface.
    fn holds(&self, c: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
        let tol = precision::confusion();
        let tol_ang = precision::angular();
        if let Some(line) = c.cast::<GeomLine>().get() {
            let li = line.lin();
            return Ok(match self {
                Self::Plane(pl) => pl.contains_line(li, tol, tol_ang),
                Self::Cylinder(cy) => {
                    cy.axis().is_parallel(li.position(), tol_ang)
                        && (li.distance(cy.location()) - cy.radius()).abs() < tol
                }
            });
        }
        let circle = c.cast::<GeomCircle>();
        let ci = circle
            .get()
            .ok_or_else(|| null_object("LocOpe_FindEdgesInFace::Set", "null circle"))?
            .circ()?;
        Ok(match self {
            Self::Plane(pl) => {
                pl.position()
                    .is_coplanar(Axis3::from_axis2(ci.position()), tol, tol_ang)
            }
            Self::Cylinder(cy) => {
                (cy.radius() - ci.radius()).abs() < tol
                    && cy.axis().is_coaxial(ci.axis(), tol_ang, tol)
            }
        })
    }

    /// The parameters of `p` on the surface.
    fn parameters(&self, p: Point3) -> (f64, f64) {
        match self {
            Self::Plane(pl) => elslib::parameters(pl, p),
            Self::Cylinder(cy) => elslib::parameters(cy, p),
        }
    }
}

/// The located 3D curve of `edg` (a trimmed curve standing for its basis) and its range.
fn located_basis(edg: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let located = BRepTool::curve_with_location(edg)?;
    let curve = located
        .curve
        .get()
        .ok_or_else(|| null_object("LocOpe_FindEdgesInFace::Set", "edge without 3D curve"))?;
    let mut c: Handle<dyn GeomCurve> = curve.transformed(located.location.transformation())?.cast();
    let trimmed: Handle<GeomTrimmedCurve> = c.cast();
    if let Some(trimmed) = trimmed.get() {
        c = trimmed.basis_curve();
    }
    Ok((c, located.first, located.last))
}

/// `LocOpe_FindEdgesInFace`: the edges of a shape in a face.
///
/// ```
/// use rustycad_tkfeat::loc_ope::FindEdgesInFace;
/// let mut find = FindEdgesInFace::new();
/// find.init();
/// assert!(!find.more());
/// ```
#[derive(Clone, Default)]
pub struct FindEdgesInFace {
    shape: Shape,
    face: Face,
    list: Vec<Shape>,
    it: usize,
}

impl FindEdgesInFace {
    /// `LocOpe_FindEdgesInFace()`; `FindEdgesInFace::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_FindEdgesInFace(S, F)`: set; `FindEdgesInFace::with_shape(&s, &f)?`.
    pub fn with_shape(s: &Shape, f: &Face) -> Result<Self, Failure> {
        let mut find = Self::new();
        find.set(s, f)?;
        Ok(find)
    }

    /// Whether the edge `edg` (a line or circle on the surface) has its ends and its
    /// middle not OUT of the face (On classifie 3 points).
    fn inside(
        surface: &FaceSurface,
        tpt: &BRepTopAdaptorTopolTool,
        c: &Handle<dyn GeomCurve>,
        f: f64,
        l: f64,
    ) -> Result<bool, Failure> {
        let curve = c
            .get()
            .ok_or_else(|| null_object("LocOpe_FindEdgesInFace::Set", "null curve"))?;
        for prm in [f, l, (f + l) / 2.] {
            let (u, v) = surface.parameters(curve.value(prm)?);
            if tpt.classify(Point2::new(u, v), precision::confusion(), true)? == State::Out {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Whether `edg`, not an edge of the face, lies in it.
    fn lies_in(
        surface: &FaceSurface,
        tpt: &BRepTopAdaptorTopolTool,
        edg: &Edge,
    ) -> Result<bool, Failure> {
        let (c, f, l) = located_basis(edg)?;
        if c.cast::<GeomLine>().is_null() && c.cast::<GeomCircle>().is_null() {
            return Ok(false); // pour le moment
        }
        if !surface.holds(&c)? {
            return Ok(false);
        }
        Self::inside(surface, tpt, &c, f, l)
    }

    /// `Set(S, F)`: on a planar or cylindrical face, the edges of `s` shared with the
    /// face and the lines and circles of `s` lying in it, each once; `find.set(&s, &f)?`.
    pub fn set(&mut self, sh: &Shape, f: &Face) -> Result<(), Failure> {
        self.shape = sh.clone();
        self.face = f.clone();
        self.list.clear();
        let Some(surface) = FaceSurface::of(f)? else {
            return Ok(()); // pour le moment
        };
        let hs: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&self.face, true)?).cast();
        let tpt = BRepTopAdaptorTopolTool::with_surface(&hs)?;
        let mut m = MapOfShape::default();
        for e in Explorer::new(&self.shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edg = topods::edge(&e)?;
            if m.contains(&e) {
                continue;
            }
            let mut face_edges = Explorer::new(&self.face, ShapeEnum::Edge, ShapeEnum::Shape)?;
            // partage d`edge
            if face_edges.any(|x| x.is_same(&e)) || Self::lies_in(&surface, &tpt, &edg)? {
                m.add(e.clone());
                self.list.push(e);
            }
        }
        Ok(())
    }

    /// `Init()`; `find.init()`.
    pub fn init(&mut self) {
        self.it = 0;
    }

    /// `More()`; `find.more()`.
    pub fn more(&self) -> bool {
        self.it < self.list.len()
    }

    /// `Edge()`: the current edge; `find.edge()?`.
    pub fn edge(&self) -> Result<Edge, Failure> {
        let e = self
            .list
            .get(self.it)
            .ok_or_else(|| not_done("LocOpe_FindEdgesInFace::Edge"))?;
        topods::edge(e)
    }

    /// `Next()`; `find.next()`.
    pub fn next(&mut self) {
        self.it += 1;
    }
}
