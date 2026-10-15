// Rust translation of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Tool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffset_Tool`: the topological and geometric services of the offset algorithm
//! (section orientation, face intersections, face enlargement and extension).
mod enlarge;
mod extent;
mod glue;
mod inter2d;
mod inter3d;
mod pcurves;
mod pipe_inter;
mod topology;

pub use enlarge::EnlargeFaceOptions;
pub use extent::ShapeImages;
pub use topology::{EdgeNeighbours, VertexEdges};

use rustycad_tkbo::bop_tools::algo_tools2d;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkgeombase::bnd_lib::Add3dCurve;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{self, Vector3};

/// `BRepOffset_Tool`: the static services of the offset algorithm, as associated
/// functions; `BRepOffsetTool::edge_vertices(&edge)`.
pub struct BRepOffsetTool;

/// The maximal value used to enlarge surfaces (`TheInfini`): 1e7, so that the
/// intersection of surfaces of such bounds keeps an inaccuracy below
/// Precision::Confusion() with fifteen significant digits.
pub(crate) const THE_INFINI: f64 = 1.0e7;

impl BRepOffsetTool {
    /// `EdgeVertices(E, V1, V2)`: the vertices of the edge in its sense;
    /// `let (v1, v2) = BRepOffsetTool::edge_vertices(&edge);`.
    pub fn edge_vertices(e: &Edge) -> (Vertex, Vertex) {
        let (first, last) = topexp::vertices(e, false);
        if e.orientation() == Orientation::Reversed {
            return (last, first);
        }
        (first, last)
    }

    /// `Gabarit(aCurve)`: the greatest side of the bounding box of the curve;
    /// `BRepOffsetTool::gabarit(&curve)?`.
    pub fn gabarit(a_curve: &Handle<dyn GeomCurve>) -> Result<f64, Failure> {
        let gc = GeomAdaptorCurve::with_curve(a_curve)?;
        let mut a_box = BoundingBox3::default();
        Add3dCurve::add(&gc, precision::confusion(), &mut a_box)?;
        let [x_min, y_min, z_min, x_max, y_max, z_max] = a_box.get()?;
        Ok((x_max - x_min).max(y_max - y_min).max(z_max - z_min))
    }

    /// `OrientSection(E, F1, F2, O1, O2)`: the orientations of a section edge in each
    /// face so that each face material lies on the left; `let (o1, o2) =
    /// BRepOffsetTool::orient_section(&edge, &f1, &f2)?;`.
    pub fn orient_section(
        e: &Edge,
        f1: &Face,
        f2: &Face,
    ) -> Result<(Orientation, Orientation), Failure> {
        let s1 = BRepTool::surface(f1)?;
        let s2 = BRepTool::surface(f2)?;
        let c1 = BRepTool::curve_on_face(e, f1)?.pcurve;
        let c2 = BRepTool::curve_on_face(e, f2)?.pcurve;
        let c = BRepTool::curve_with_location(e)?;
        let (f, l) = (c.first, c.last);
        let ba_curve = BRepAdaptorCurve::with_edge(e)?;
        let length = AbscissaPoint::length(&ba_curve as &dyn Adaptor3dCurve)?;
        let ap = AbscissaPoint::from_abscissa(&ba_curve as &dyn Adaptor3dCurve, length / 2.0, f)?;
        let par_on_c = if ap.is_done() {
            ap.parameter()?
        } else {
            algo_tools2d::intermediate_point(f, l)
        };
        let curve = live(&c.curve, "curve")?;
        let mut t1 = curve
            .dn(par_on_c, 1)?
            .transformed(c.location.transformation());
        if t1.square_magnitude() > gp::resolution() {
            t1.normalize()?;
        }
        let dn1 = face_normal(&s1, &c1, par_on_c, f1)?;
        let dn2 = face_normal(&s2, &c2, par_on_c, f2)?;
        let mut o1 = section_orientation(dn1.dot(dn2.crossed(t1)));
        let mut o2 = section_orientation(dn2.dot(dn1.crossed(t1)));
        if f1.orientation() == Orientation::Reversed {
            o1 = top_abs::reverse(o1);
        }
        if f2.orientation() == Orientation::Reversed {
            o2 = top_abs::reverse(o2);
        }
        Ok((o1, o2))
    }

    /// `FindCommonShapes(theF1, theF2, theLE, theLV)`: the common edges and vertices
    /// of two faces; whether there is any; `BRepOffsetTool::find_common_shapes_of_faces(&f1, &f2, &mut le, &mut lv)?`.
    pub fn find_common_shapes_of_faces(
        the_f1: &Face,
        the_f2: &Face,
        the_le: &mut List<Shape>,
        the_lv: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        let found_edges =
            Self::find_common_shapes(the_f1.as_ref(), the_f2.as_ref(), ShapeEnum::Edge, the_le)?;
        let found_verts =
            Self::find_common_shapes(the_f1.as_ref(), the_f2.as_ref(), ShapeEnum::Vertex, the_lv)?;
        Ok(found_edges || found_verts)
    }

    /// `FindCommonShapes(theS1, theS2, theType, theLSC)`: the sub-shapes of a type
    /// common to two shapes, each once in the order of the second shape;
    /// `BRepOffsetTool::find_common_shapes(&s1, &s2, ShapeEnum::Edge, &mut l)?`.
    pub fn find_common_shapes(
        the_s1: &Shape,
        the_s2: &Shape,
        the_type: ShapeEnum,
        the_lsc: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        the_lsc.clear();
        let mut a_ms = MapOfShape::default();
        let mut a_exp = Explorer::new(the_s1, the_type, ShapeEnum::Shape)?;
        while a_exp.more() {
            a_ms.add(a_exp.current().clone());
            a_exp.next()?;
        }
        if a_ms.is_empty() {
            return Ok(false);
        }
        let mut a_m_fence = MapOfShape::default();
        let mut a_exp = Explorer::new(the_s2, the_type, ShapeEnum::Shape)?;
        while a_exp.more() {
            let a_s2 = a_exp.current();
            if a_ms.contains(a_s2) && a_m_fence.add(a_s2.clone()) {
                the_lsc.append(a_s2.clone());
            }
            a_exp.next()?;
        }
        Ok(!the_lsc.is_empty())
    }

    /// `CheckPlanesNormals(theFace1, theFace2, theTolAng)` (natively defaulted to
    /// 1e-8): whether two plane faces have the same oriented normal;
    /// `BRepOffsetTool::check_planes_normals(&f1, &f2, 1e-8)?`.
    pub fn check_planes_normals(
        the_face1: &Face,
        the_face2: &Face,
        the_tol_ang: f64,
    ) -> Result<bool, Failure> {
        let a_bas1 = BRepAdaptorSurface::with_face(the_face1, false)?;
        let a_bas2 = BRepAdaptorSurface::with_face(the_face2, false)?;
        if a_bas1.get_type()? != SurfaceType::Plane || a_bas2.get_type()? != SurfaceType::Plane {
            return Ok(false);
        }
        let mut a_dn1 = a_bas1.plane()?.position().direction();
        if the_face1.orientation() == Orientation::Reversed {
            a_dn1.reverse();
        }
        let mut a_dn2 = a_bas2.plane()?.position().direction();
        if the_face2.orientation() == Orientation::Reversed {
            a_dn2.reverse();
        }
        Ok(a_dn1.angle(&a_dn2) < the_tol_ang)
    }
}

/// The section orientation of a sign product: FORWARD when negative.
fn section_orientation(product: f64) -> Orientation {
    if product < 0.0 {
        Orientation::Forward
    } else {
        Orientation::Reversed
    }
}

/// The normal `D1U ^ D1V` of the surface at the p-curve point, reversed for a
/// REVERSED face.
fn face_normal(
    s: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    c: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    par: f64,
    face: &Face,
) -> Result<Vector3, Failure> {
    let p = live(c, "pcurve")?.value(par)?;
    let d1 = live(s, "surface")?.d1(p.x(), p.y())?;
    let mut dn = d1.d1u.crossed(d1.d1v);
    if face.orientation() == Orientation::Reversed {
        dn.reverse();
    }
    Ok(dn)
}

/// The object of a live handle; Standard_NullObject otherwise.
pub(crate) fn live<'a, T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &'a Handle<T>,
    what: &str,
) -> Result<&'a T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("BRepOffset_Tool: null {what}; expected a geometry"),
        )
    })
}
