// Rust translation of LocOpe_WiresOnShape::Add of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_WiresOnShape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Add`: binds splitting edges to the faces of the shape whose interior holds the
//! middle of their curve.
use super::WiresOnShape;
use crate::errors::out_of_range;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::ncollection::{Map, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeombase::extrema::{ExtFlag, ExtPS};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// The edges of the splitting shapes and the boxes of the splitting shapes. Native
/// quirk kept: the box of the input shape `i` (the box of its last edge) sits at index
/// `i`, while the edges are numbered across all inputs, so the two only correspond
/// when every input holds one edge.
fn edges_and_boxes(
    the_edges: &Sequence<Shape>,
) -> Result<(Vec<Shape>, Vec<BoundingBox3>), Failure> {
    let mut an_edges = Vec::new();
    let mut boxes = vec![BoundingBox3::default(); the_edges.len()];
    for (i, cur_split) in the_edges.iter().enumerate() {
        for cur_e in Explorer::new(cur_split, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let mut box_e = BoundingBox3::default();
            BRepBndLib::add_close(&cur_e, &mut box_e)?;
            if box_e.is_void() {
                continue;
            }
            box_e.set_gap(BRepTool::edge_tolerance(&topods::edge(&cur_e)?)?);
            boxes[i] = box_e;
            an_edges.push(cur_e);
        }
    }
    Ok((an_edges, boxes))
}

/// The classification tool of a face, built on first use.
struct LazyClassifier<'a> {
    face: &'a Face,
    tool: Option<BRepTopAdaptorFClass2d>,
}

impl LazyClassifier<'_> {
    fn is_in(&mut self, u: f64, v: f64) -> Result<bool, Failure> {
        if self.tool.is_none() {
            self.tool = Some(BRepTopAdaptorFClass2d::new(
                self.face,
                precision::p_confusion(),
            )?);
        }
        let tool = self
            .tool
            .as_ref()
            .ok_or_else(|| out_of_range("LocOpe_WiresOnShape::Add", 0, 0))?;
        Ok(tool.perform(Point2::new(u, v), true)? == State::In)
    }
}

impl WiresOnShape {
    /// Binds to `cur_f` the edge `i` when an extremum of the middle of its curve on the
    /// face lies within its tolerance and inside the face; marks it used.
    fn bind_if_on_face(
        &mut self,
        cur_e: &Shape,
        cur_f: &Face,
        an_extr: &mut ExtPS<'_>,
        classifier: &mut LazyClassifier<'_>,
    ) -> Result<bool, Failure> {
        let cur_edge = topods::edge(cur_e)?;
        let (a_c, a_f, a_l) = BRepTool::curve(&cur_edge)?;
        let Some(curve) = a_c.get() else {
            return Ok(true);
        };
        let a_p = curve.value((a_f + a_l) * 0.5)?;
        an_extr.perform(a_p)?;
        if !an_extr.is_done() || an_extr.nb_ext()? == 0 {
            return Ok(false);
        }
        let tol_e = BRepTool::edge_tolerance(&cur_edge)?;
        let tol2 = (tol_e + precision::confusion()).powi(2);
        let mut used = false;
        for n in 1..=an_extr.nb_ext()? {
            if an_extr.square_distance(n)? > tol2 {
                continue;
            }
            let (u, v) = an_extr.point(n)?.parameter();
            if classifier.is_in(u, v)? {
                self.bind_edge(&cur_edge, cur_f)?;
                used = true;
            }
        }
        Ok(used)
    }

    /// The edges of the inputs bound to one face of the shape.
    fn add_on_face(
        &mut self,
        cur_f: &Face,
        edges: &[Shape],
        boxes: &[BoundingBox3],
        used: &mut Map<i32>,
    ) -> Result<(), Failure> {
        let mut box_f = BoundingBox3::default();
        BRepBndLib::add(cur_f, &mut box_f, true)?;
        if box_f.is_void() {
            return Ok(());
        }
        let ad_f = BRepAdaptorSurface::with_face(cur_f, false)?;
        // Initialize extrema projector once per face
        let mut an_extr = ExtPS::new();
        an_extr.initialize(
            &ad_f,
            (ad_f.first_u_parameter()?, ad_f.last_u_parameter()?),
            (ad_f.first_v_parameter()?, ad_f.last_v_parameter()?),
            (precision::confusion(), precision::confusion()),
        )?;
        an_extr.set_flag(ExtFlag::Min);
        let mut classifier = LazyClassifier {
            face: cur_f,
            tool: None,
        };
        for (k, edge_box) in boxes.iter().enumerate() {
            let i = k as i32 + 1;
            if used.contains(&i) || box_f.is_out_box(edge_box) {
                continue;
            }
            let cur_e = edges
                .get(k)
                .ok_or_else(|| out_of_range("LocOpe_WiresOnShape::Add", i, edges.len() as i32))?;
            if self.bind_if_on_face(cur_e, cur_f, &mut an_extr, &mut classifier)? {
                used.add(i);
            }
        }
        Ok(())
    }

    /// `Add(theEdges)`: binds each edge of the splitting shapes to the faces of the
    /// shape holding the middle of its curve in their interior (within the edge
    /// tolerance); an edge without 3D curve is dropped. True when an edge was bound or
    /// dropped; `wires.add(&edges)?`.
    pub fn add(&mut self, the_edges: &Sequence<Shape>) -> Result<bool, Failure> {
        let (edges, boxes) = edges_and_boxes(the_edges)?;
        let mut used: Map<i32> = Map::default();
        let shape = self.shape.clone();
        for f in Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            self.add_on_face(&topods::face(&f)?, &edges, &boxes, &mut used)?;
        }
        Ok(!used.is_empty())
    }
}
