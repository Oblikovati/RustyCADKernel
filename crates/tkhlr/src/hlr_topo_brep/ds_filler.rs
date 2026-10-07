//! HLRTopoBRep_DSFiller.hxx/.cxx (Insert, the vertices and the edge splitting), OCCT
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{Data, FaceIsoLiner};
use crate::contap::{ContapContour, ContapPoint};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_top_adaptor::{BRepTopAdaptorHVertex, BRepTopAdaptorTool};

/// The surface tools of the faces, shared with HLRBRep (the native
/// `NCollection_DataMap<TopoDS_Shape, BRepTopAdaptor_Tool, TopTools_ShapeMapHasher>`).
pub type MapOfShapeTool = DataMap<Shape, BRepTopAdaptorTool, ShapeMapHasher>;

/// HLRTopoBRep_DSFiller: fills the data structure with the outlines of the faces
/// of a shape (Contap contours turned into edges and vertices), their isolines, and
/// the edges split at the outline vertices;
/// `DSFiller::insert(&shape, &mut contour, &mut data, &mut tools, 0)?`.
pub struct DSFiller;

pub(super) fn null_object(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("HLRTopoBRep_DSFiller: {what} is null; expected a BRep adaptor"),
        "",
    )
}

// The edge of a Contap arc (`(*(BRepAdaptor_Curve2d*)(Arc.get())).Edge()`).
pub(super) fn arc_edge(arc: &rustycad_tkgeomalgo::int_start::TheArc) -> Result<Edge, Failure> {
    let curve = arc.cast::<BRepAdaptorCurve2d>();
    Ok(curve.get().ok_or_else(|| null_object("the arc"))?.edge())
}

// The vertex of a Contap vertex (`down_cast<BRepTopAdaptor_HVertex>(V)->Vertex()`).
fn hvertex_vertex(v: &rustycad_tkgeomalgo::int_start::TheVertex) -> Result<Vertex, Failure> {
    let hv = v.cast::<BRepTopAdaptorHVertex>();
    Ok(hv
        .get()
        .ok_or_else(|| null_object("the vertex"))?
        .vertex()
        .clone())
}

impl DSFiller {
    /// Insert(S, FO, DS, MST, nbIso): the outlines of each face of `s` (once per
    /// face, FORWARD, with the surface tool of `mst`, created when missing), the
    /// isolines when `nb_iso` is not 0, then the split edges;
    /// `DSFiller::insert(&shape, &mut contour, &mut data, &mut tools, 0)?`.
    pub fn insert(
        s: &Shape,
        fo: &mut ContapContour,
        ds: &mut Data,
        mst: &mut MapOfShapeTool,
        nb_iso: i32,
    ) -> Result<(), Failure> {
        let mut shape_map = MapOfShape::default();
        let mut ex = Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?;
        ds.clear();
        let with_p_curve = true; // instead of nbIso != 0;
        let mut f = 0;
        while ex.more() {
            if shape_map.add(ex.current().clone()) {
                f += 1;
                let mut s1 = topods::face(ex.current())?;
                s1.set_orientation(Orientation::Forward);
                Self::insert_one_face(f, &s1, fo, ds, mst, with_p_curve)?;
                if nb_iso != 0 {
                    FaceIsoLiner::perform(f, &s1, ds, nb_iso)?;
                }
            }
            ex.next()?;
        }
        Self::process_edges(ds)
    }

    fn insert_one_face(
        f: i32,
        s1: &Face,
        fo: &mut ContapContour,
        ds: &mut Data,
        mst: &mut MapOfShapeTool,
        with_p_curve: bool,
    ) -> Result<(), Failure> {
        let key = Shape::from(s1.clone());
        if !mst.is_bound(&key) {
            mst.bind(
                key.clone(),
                BRepTopAdaptorTool::with_face(s1, precision::p_confusion())?,
            );
        }
        let brt = mst.find(&key)?;
        let (domain, surface) = (brt.get_topol_tool(), brt.get_surface());
        let domain = domain
            .get()
            .ok_or_else(|| null_object("the topological tool"))?;
        fo.perform(&surface, domain)?;
        if fo.is_done() && !fo.is_empty()? {
            Self::insert_face(f, s1, fo, ds, with_p_curve)?;
        }
        Ok(())
    }

    /// MakeVertex(P, tol, DS): the vertex of a contour point: the vertex of the
    /// shape, a vertex on an arc (shared with one already inserted at the same
    /// point, or inserted in parameter order), or a new internal or outline vertex;
    /// `DSFiller::make_vertex(&point, 1e-7, &mut data)?`.
    pub fn make_vertex(p: &ContapPoint, tol: f64, ds: &mut Data) -> Result<Vertex, Failure> {
        if p.is_vertex() {
            let v = hvertex_vertex(&p.vertex()?)?;
            ds.add_out_v(&v);
            return Ok(v);
        }
        if !p.is_on_arc() {
            let v = new_vertex(p, tol)?;
            if p.is_internal() {
                ds.add_int_v(&v);
            } else {
                ds.add_out_v(&v);
            }
            return Ok(v);
        }
        let e = arc_edge(&p.arc()?)?;
        let v = Self::vertex_on_edge(p, &e, p.parameter_on_arc()?, tol, ds)?;
        ds.add_out_v(&v);
        Ok(v)
    }

    // The vertex at the contour point among those inserted on `e`, or a new one
    // inserted before the first vertex of greater parameter, or appended.
    fn vertex_on_edge(
        p: &ContapPoint,
        e: &Edge,
        par: f64,
        tol: f64,
        ds: &mut Data,
    ) -> Result<Vertex, Failure> {
        ds.init_vertex(e);
        while ds.more_vertex() {
            let cur_v = ds.vertex()?;
            let cur_p = ds.parameter()?;
            if p.value()
                .is_equal(&BRepTool::pnt(&cur_v)?, BRepTool::vertex_tolerance(&cur_v)?)
            {
                return Ok(cur_v);
            }
            if par < cur_p {
                let v = new_vertex(p, tol)?;
                ds.insert_before(&v, par)?;
                return Ok(v);
            }
            ds.next_vertex();
        }
        let v = new_vertex(p, tol)?;
        ds.append(&v, par)?;
        Ok(v)
    }

    /// InsertVertex(P, tol, E, DS): an internal contour point on the restriction
    /// `e`: the vertex of the shape, or the vertex at the point on `e` (shared or
    /// inserted by the parameter on the line), recorded as internal;
    /// `DSFiller::insert_vertex(&point, 1e-7, &edge, &mut data)?`.
    pub fn insert_vertex(
        p: &ContapPoint,
        tol: f64,
        e: &Edge,
        ds: &mut Data,
    ) -> Result<(), Failure> {
        let v = if p.is_vertex() {
            hvertex_vertex(&p.vertex()?)?
        } else {
            Self::vertex_on_edge(p, e, p.parameter_on_line(), tol, ds)?
        };
        ds.add_int_v(&v);
        Ok(())
    }

    /// ProcessEdges(DS): splits each edge with inserted vertices into the edges
    /// between them; `DSFiller::process_edges(&mut data)?`.
    pub fn process_edges(ds: &mut Data) -> Result<(), Failure> {
        ds.init_edge();
        while ds.more_edge() {
            let e = ds.edge()?;
            Self::split_edge(&e, ds)?;
            ds.next_edge();
        }
        Ok(())
    }

    fn split_edge(e: &Edge, ds: &mut Data) -> Result<(), Failure> {
        let mut vf = topexp::first_vertex(e, false);
        let mut vl = topexp::last_vertex(e, false);
        let (mut pf, pl) = BRepTool::range(e)?;
        vf.set_orientation(Orientation::Forward);
        vl.set_orientation(Orientation::Reversed);
        let mut parts = Vec::new();
        ds.init_vertex(e);
        while ds.more_vertex() {
            let mut vi = ds.vertex()?;
            let pi = ds.parameter()?;
            vi.set_orientation(Orientation::Reversed);
            parts.push(split_part(e, (&vf, pf), (&vi, pi))?);
            vf = vi;
            pf = pi;
            vf.set_orientation(Orientation::Forward);
            ds.next_vertex();
        }
        parts.push(split_part(e, (&vf, pf), (&vl, pl))?);
        let spl_e = ds.add_spl_e(e);
        for part in parts {
            spl_e.append(Shape::from(part));
        }
        Ok(())
    }
}

// An empty copy of `e` between two vertices with their parameters, then given the
// orientation of `e`.
fn split_part(
    e: &Edge,
    (vf, pf): (&Vertex, f64),
    (vl, pl): (&Vertex, f64),
) -> Result<Edge, Failure> {
    let b = BRepBuilder::default();
    let mut new_e = e.clone();
    new_e.empty_copy()?;
    new_e.set_orientation(Orientation::Forward);
    b.add(&mut new_e, vf)?;
    b.update_vertex_on_edge(vf, pf, &new_e, BRepTool::vertex_tolerance(vf)?)?;
    b.add(&mut new_e, vl)?;
    b.update_vertex_on_edge(vl, pl, &new_e, BRepTool::vertex_tolerance(vl)?)?;
    new_e.set_orientation(e.orientation());
    Ok(new_e)
}

fn new_vertex(p: &ContapPoint, tol: f64) -> Result<Vertex, Failure> {
    let mut v = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut v, p.value(), tol)?;
    Ok(v)
}

mod insert_face;
