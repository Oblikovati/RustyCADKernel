// Rust translation of BRepFeat_MakeRevolutionForm::Propagate (the first face) of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeRevolutionForm.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Propagate`: the faces of the basis the profile crosses. On the first face the
//! profile cuts one or two section edges (a rib of revolution may cross it twice); the
//! walk then goes on from both ends of the section until they reach the end points of
//! the wire.
use super::BRepFeatMakeRevolutionForm;
use crate::brep_feat::make_linear_form::section_edges;
use crate::errors::no_such_object;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// The state of the propagation: the end points (moved along the second section edge),
/// whether each end is reached, and the vertices the walks start from.
pub(super) struct Prop {
    pub(super) firstpoint: Point3,
    pub(super) lastpoint: Point3,
    pub(super) first_ok: bool,
    pub(super) last_ok: bool,
    pub(super) v1: Vertex,
    pub(super) v2: Vertex,
}

/// Whether the vertex lies at one of the end points within its tolerance.
pub(super) fn at_ends(v: &Vertex, prop: &Prop) -> Result<bool, Failure> {
    let (p, t) = (BRepTool::pnt(v)?, BRepTool::vertex_tolerance(v)?);
    Ok(p.distance(&prop.firstpoint) <= t || p.distance(&prop.lastpoint) <= t)
}

/// Whether the vertex is within its tolerance of `target`.
fn near(v: &Vertex, target: Point3) -> Result<bool, Failure> {
    Ok(BRepTool::pnt(v)?.distance(&target) <= BRepTool::vertex_tolerance(v)?)
}

impl BRepFeatMakeRevolutionForm {
    /// The section edge `e` alone on the face, the list of edges restarted with it.
    fn only_edge(&mut self, f: &Face, e: &Edge) {
        self.list_of_edges.clear();
        let mut thelist = List::default();
        thelist.append(e.clone().into_shape());
        self.slface.bind(f.clone().into_shape(), thelist);
        self.list_of_edges.append(e.clone().into_shape());
    }

    /// The ends of the edge `e` checked against the end points.
    fn ends_of(e: &Edge, prop: &mut Prop) -> Result<(), Failure> {
        prop.v1 = topexp::first_vertex(e, true);
        prop.v2 = topexp::last_vertex(e, true);
        if at_ends(&prop.v1, prop)? {
            prop.first_ok = true;
        }
        if at_ends(&prop.v2, prop)? {
            prop.last_ok = true;
        }
        Ok(())
    }

    /// The second section edge `e1` continuing `e` at one end point: the other end
    /// point moved to its far end (natively), else `e1` dropped (or taking the place of
    /// `e` when `e` touches no end point).
    fn pair(
        &mut self,
        f: &Face,
        e: &mut Edge,
        e1: &mut Edge,
        prop: &mut Prop,
    ) -> Result<(), Failure> {
        self.only_edge(f, e);
        self.slface
            .change_find(&f.clone().into_shape())?
            .append(e1.clone().into_shape());
        prop.v1 = topexp::first_vertex(e, true);
        prop.v2 = topexp::last_vertex(e, true);
        let v3 = topexp::first_vertex(e1, true);
        let v4 = topexp::last_vertex(e1, true);
        let (p3, p4) = (BRepTool::pnt(&v3)?, BRepTool::pnt(&v4)?);
        let follow = |target: Point3| -> Result<Option<Point3>, Failure> {
            if near(&v3, target)? {
                return Ok(Some(p4));
            }
            if near(&v4, target)? {
                return Ok(Some(p3));
            }
            Ok(None)
        };
        let (on_first, on_last) = (
            near(&prop.v1, prop.firstpoint)?,
            near(&prop.v1, prop.lastpoint)?,
        );
        let (on2_first, on2_last) = (
            near(&prop.v2, prop.firstpoint)?,
            near(&prop.v2, prop.lastpoint)?,
        );
        let moved = if on_first {
            follow(prop.lastpoint)?.map(|p| (true, false, p))
        } else if on_last {
            follow(prop.firstpoint)?.map(|p| (true, true, p))
        } else if on2_first {
            follow(prop.lastpoint)?.map(|p| (false, false, p))
        } else if on2_last {
            follow(prop.firstpoint)?.map(|p| (false, true, p))
        } else {
            *e = e1.clone();
            e1.nullify();
            return Ok(());
        };
        match moved {
            Some((first, move_first, p)) => {
                if first {
                    prop.first_ok = true;
                } else {
                    prop.last_ok = true;
                }
                if move_first {
                    prop.firstpoint = p;
                } else {
                    prop.lastpoint = p;
                }
            }
            None => e1.nullify(),
        }
        Ok(())
    }

    /// `Propagate(SliList, fac, Firstpnt, Lastpnt, falseside)`: see the module;
    /// `falseside` turns false when the profile misses the first face or the walk turns
    /// back; returns whether the rib can slide.
    pub fn propagate(
        &mut self,
        sli_list: &mut List<Shape>,
        fac: &Face,
        firstpnt: Point3,
        lastpnt: Point3,
        falseside: &mut bool,
    ) -> Result<bool, Failure> {
        let current_face = topods::face(
            sli_list
                .first()
                .ok_or_else(|| no_such_object("Propagate", "no face"))?,
        )?;
        let mut prop = Prop {
            firstpoint: firstpnt,
            lastpoint: lastpnt,
            first_ok: false,
            last_ok: false,
            v1: Vertex::default(),
            v2: Vertex::default(),
        };
        let edges = section_edges(fac, &current_face)?;
        let mut e = edges
            .first()
            .map(topods::edge)
            .transpose()?
            .unwrap_or_default();
        let mut e1 = edges
            .get(1)
            .map(topods::edge)
            .transpose()?
            .unwrap_or_default();
        if e.is_null() {
            *falseside = false;
            return Ok(false);
        }
        if !e1.is_null() {
            self.only_edge(&current_face, &e1);
            Self::ends_of(&e1, &mut prop)?;
            if prop.last_ok && prop.first_ok {
                return Ok(true);
            }
            self.list_of_edges.clear();
            self.pair(&current_face, &mut e, &mut e1, &mut prop)?;
        }
        if e1.is_null() {
            self.only_edge(&current_face, &e);
            Self::ends_of(&e, &mut prop)?;
            if prop.last_ok && prop.first_ok {
                return Ok(true);
            }
        }
        let result = match self.walk_both(sli_list, fac, &current_face, &mut prop, falseside)? {
            Some(r) => r,
            None => return Ok(false),
        };
        if !e1.is_null() {
            self.list_of_edges.append(e1.into_shape());
        }
        Ok(result)
    }
}
