//! HLRTopoBRep_FaceIsoLiner.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::Data;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom2d_hatch::{Hatcher, Intersector};
use rustycad_tkmath::gp::Point3;

const INTERSECTOR_CONFUSION: f64 = 1.0e-10;
const INTERSECTOR_TANGENCY: f64 = 1.0e-10;
const HATCHER_CONFUSION_2D: f64 = 1.0e-8;
const HATCHER_CONFUSION_3D: f64 = 1.0e-8;
const INFINITE: f64 = 100.;

/// HLRTopoBRep_FaceIsoLiner: the isolines of a face, cut by its edges and internal
/// outlines with a 2D hatcher; `FaceIsoLiner::perform(1, &face, &mut data, 4)?`.
pub struct FaceIsoLiner;

// The faces' elements: the edge or outline of each hatcher element and whether it
// is an outline (the native `SH` and `IL` arrays).
struct Elements {
    shapes: Vec<Shape>,
    outlines: Vec<bool>,
}

impl Elements {
    fn set(&mut self, ind_e: i32, shape: &Shape, outline: bool) {
        let at = (ind_e - 1) as usize;
        if at >= self.shapes.len() {
            self.shapes.resize(at + 1, Shape::default());
            self.outlines.resize(at + 1, false);
        }
        self.shapes[at] = shape.clone();
        self.outlines[at] = outline;
    }
    fn edge(&self, ind_e: i32) -> Result<Edge, Failure> {
        let shape = self.shapes.get((ind_e - 1) as usize).ok_or_else(|| {
            Failure::out_of_range(&format!(
                "HLRTopoBRep_FaceIsoLiner: element {ind_e}; expected 1..={}",
                self.shapes.len()
            ))
        })?;
        topods::edge(shape)
    }
    fn is_outline(&self, ind_e: i32) -> bool {
        self.outlines
            .get((ind_e - 1) as usize)
            .copied()
            .unwrap_or(false)
    }
}

// The face bounds with the infinite ones replaced by `Infinite` around the other.
fn finite_bounds(min: f64, max: f64) -> (f64, f64) {
    let (inf_min, inf_max) = (
        precision::is_negative_infinite(min),
        precision::is_positive_infinite(max),
    );
    match (inf_min, inf_max) {
        (true, true) => (-INFINITE, INFINITE),
        (true, false) => (max - INFINITE, max),
        (false, true) => (min, min + INFINITE),
        _ => (min, max),
    }
}

// `Hatcher.AddElement(PC or its trimmed copy, orientation)`.
fn add_element(
    hatcher: &mut Hatcher,
    edge: &Edge,
    face: &Face,
    orientation: Orientation,
) -> Result<i32, Failure> {
    let pc = BRepTool::curve_on_face(edge, face)?;
    let curve = pc.pcurve.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "HLRTopoBRep_FaceIsoLiner: edge without p-curve; expected one",
            "",
        )
    })?;
    let (u1, u2) = (pc.first, pc.last);
    if (curve.first_parameter() - u1).abs() <= precision::p_confusion()
        && (curve.last_parameter() - u2).abs() <= precision::p_confusion()
    {
        return hatcher.add_element_curve(&pc.pcurve, orientation);
    }
    let tpc: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&pc.pcurve, u1, u2, true, true)?).cast();
    Ok(hatcher.add_element(Geom2dAdaptorCurve::with_curve(&tpc)?, orientation))
}

// The vertices of the iso lines, kept across the domains and the isos as the
// native locals `V1U`, `V2U` (or `V1V`, `V2V`).
#[derive(Default)]
struct IsoEnds {
    v1: Vertex,
    v2: Vertex,
}

// The values of Perform shared by its two iso loops.
struct IsoContext<'a> {
    face: &'a Face,
    surface: BRepAdaptorSurface,
    tolerance: f64,
    v_bounds: (f64, f64),
    elements: Elements,
}

impl FaceIsoLiner {
    /// Perform(FI, F, DS, nbIsos): `nb_isos` isolines in U then in V, at the middle
    /// of `nb_isos` equal steps of the face bounds (infinite bounds replaced by 100
    /// around the other bound), cut by the edges and the internal outlines of the
    /// face; `FaceIsoLiner::perform(1, &face, &mut data, 4)?`.
    pub fn perform(_fi: i32, f: &Face, ds: &mut Data, nb_isos: i32) -> Result<(), Failure> {
        let mut tf = f.clone();
        tf.set_orientation(Orientation::Forward);
        let intersector = Intersector::new(INTERSECTOR_CONFUSION, INTERSECTOR_TANGENCY);
        let mut hatcher = Hatcher::new(
            &intersector,
            HATCHER_CONFUSION_2D,
            HATCHER_CONFUSION_3D,
            true,
            false,
        );
        let (umin, umax, vmin, vmax) = uv_bounds(&tf)?;
        let (umin, umax) = finite_bounds(umin, umax);
        let (vmin, vmax) = finite_bounds(vmin, vmax);
        let elements = Self::add_elements(&mut hatcher, &tf, ds)?;
        let ctx = IsoContext {
            face: f,
            surface: BRepAdaptorSurface::with_face(&tf, true)?,
            tolerance: BRepTool::face_tolerance(&tf)?,
            v_bounds: (vmin, vmax),
            elements,
        };
        let (delta_u, delta_v) = ((umax - umin).abs(), (vmax - vmin).abs());
        let confusion = delta_u.min(delta_v) * HATCHER_CONFUSION_3D;
        hatcher.set_confusion3d(confusion);
        let step_u = delta_u / f64::from(nb_isos);
        if step_u > confusion {
            ctx.iso_lines(&mut hatcher, ds, (umin, step_u, nb_isos), true)?;
        }
        let step_v = delta_v / f64::from(nb_isos);
        if step_v > confusion {
            ctx.iso_lines(&mut hatcher, ds, (vmin, step_v, nb_isos), false)?;
        }
        Ok(())
    }

    // The edges of the face (outline when an outline on restriction of `ds`) then its
    // internal outlines, as hatcher elements.
    fn add_elements(hatcher: &mut Hatcher, tf: &Face, ds: &Data) -> Result<Elements, Failure> {
        let mut elements = Elements {
            shapes: Vec::new(),
            outlines: Vec::new(),
        };
        let mut ex = Explorer::new(tf, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while ex.more() {
            let new_e = topods::edge(ex.current())?;
            let ind_e = add_element(hatcher, &new_e, tf, new_e.orientation())?;
            elements.set(ind_e, ex.current(), ds.is_out_l_face_edge(tf, &new_e)?);
            ex.next()?;
        }
        if ds.face_has_int_l(tf) {
            for s in ds.face_int_l(tf)?.iter() {
                let new_e = topods::edge(s)?;
                let ind_e = add_element(hatcher, &new_e, tf, Orientation::Internal)?;
                elements.set(ind_e, s, true);
            }
        }
        Ok(elements)
    }

    /// MakeVertex(E, P, Par, Tol, DS): an end vertex of `e` at `p`, a vertex already
    /// inserted on `e` at `p`, or a new INTERNAL vertex inserted in parameter order;
    /// `FaceIsoLiner::make_vertex(&edge, p, 0.5, 1e-7, &mut data)?`.
    pub fn make_vertex(
        e: &Edge,
        p: Point3,
        par: f64,
        tol: f64,
        ds: &mut Data,
    ) -> Result<Vertex, Failure> {
        let (vf, vl) = topexp::vertices(e, false);
        if p.is_equal(&BRepTool::pnt(&vf)?, BRepTool::vertex_tolerance(&vf)?) {
            return Ok(vf);
        }
        if p.is_equal(&BRepTool::pnt(&vl)?, BRepTool::vertex_tolerance(&vl)?) {
            return Ok(vl);
        }
        ds.init_vertex(e);
        while ds.more_vertex() {
            let cur_v = ds.vertex()?;
            let cur_p = ds.parameter()?;
            if p.is_equal(&BRepTool::pnt(&cur_v)?, BRepTool::vertex_tolerance(&cur_v)?) {
                return Ok(cur_v);
            }
            if par < cur_p {
                let v = new_internal_vertex(p, tol)?;
                ds.insert_before(&v, par)?;
                return Ok(v);
            }
            ds.next_vertex();
        }
        let v = new_internal_vertex(p, tol)?;
        ds.append(&v, par)?;
        Ok(v)
    }

    /// MakeIsoLine(F, Iso, V1, V2, U1, U2, Tol, DS): the INTERNAL edge of the iso line
    /// on `f` between `v1` (made FORWARD) and `v2` (made REVERSED), added to the
    /// isolines of the face; `FaceIsoLiner::make_iso_line(&f, &iso, &mut v1, &mut v2, 0.0, 1.0, 1e-7, &mut data)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn make_iso_line(
        f: &Face,
        iso: &Handle<Geom2dLine>,
        v1: &mut Vertex,
        v2: &mut Vertex,
        u1: f64,
        u2: f64,
        tol: f64,
        ds: &mut Data,
    ) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        let mut e = Edge::default();
        e.set_orientation(Orientation::Internal);
        v1.set_orientation(Orientation::Forward);
        v2.set_orientation(Orientation::Reversed);
        b.make_edge(&mut e)?;
        b.update_edge_pcurve_on_face(&e, &iso.clone().cast(), f, tol)?;
        b.add(&mut e, &*v1)?;
        b.update_vertex_on_edge(v1, u1, &e, tol)?;
        b.add(&mut e, &*v2)?;
        b.update_vertex_on_edge(v2, u2, &e, tol)?;
        ds.add_iso_l(f).append(Shape::from(e));
        Ok(())
    }
}

fn new_internal_vertex(p: Point3, tol: f64) -> Result<Vertex, Failure> {
    let mut v = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut v, p, tol)?;
    v.set_orientation(Orientation::Internal);
    Ok(v)
}

mod iso_lines;
