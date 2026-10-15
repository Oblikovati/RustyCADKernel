// Rust translation of the sliding base of BRepFeat_MakeLinearForm::Init and
// BRepFeat_MakeRevolutionForm::Init of ModelingAlgorithms/TKFeat/BRepFeat/
// BRepFeat_MakeLinearForm.cxx and BRepFeat_MakeRevolutionForm.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The base of a sliding rib: the edges of the wire from the first point to the last
//! one, then the section edges of the sliding faces chained back from the last point to
//! the first one; the sliding faces are re-bound to the edges of the base they carry.
//! The two forms differ natively: the revolution form takes the first edge on its
//! untrimmed curve, keeps the vertex tolerances and walks the section edges again from
//! the start without removing the ones taken.
use super::{Extremes, RibSlot, int_par};
use crate::errors::no_such_object;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeVertex};

/// The base wire being built: its last edge, its first vertex and the edge count.
struct Base {
    b: BRepBuilder,
    w: Wire,
    the_previous_edge: Edge,
    the_fv: Vertex,
    counter: i32,
}

impl Base {
    /// The edge oriented as `orientation` added to the wire.
    fn push(&mut self, e: Edge, orientation: Option<Orientation>) -> Result<Edge, Failure> {
        let e = match orientation {
            Some(o) => topods::edge(&e.oriented(o))?,
            None => e,
        };
        self.b.add(&mut self.w, &e)?;
        if self.counter == 1 {
            self.the_fv = topexp::first_vertex(&e, true);
        }
        self.the_previous_edge = e.clone();
        Ok(e)
    }
}

/// The trimmed curve of an edge (`new Geom_TrimmedCurve(C, f, l)`).
fn trimmed(e: &Edge) -> Result<Handle<dyn GeomCurve>, Failure> {
    let (c, f, l) = BRepTool::curve(e)?;
    Ok(Handle::new(GeomTrimmedCurve::new(&c, f, l, true, true)?).cast())
}

/// The vertex at a point.
fn vertex(p: rustycad_tkmath::gp::Point3) -> Result<Vertex, Failure> {
    BRepLibMakeVertex::new(p)?.vertex()
}

impl SlidingBase<'_> {
    /// The first edge of the wire, from the first point (to the last one when it is
    /// the only edge).
    fn base_first_edge(&mut self, e: &Edge, x: &Extremes, base: &mut Base) -> Result<(), Failure> {
        let mut cc = if self.revol {
            BRepTool::curve(e)?.0
        } else {
            trimmed(e)?
        };
        let r = &self.rib;
        let pt = if !x.first_edge.is_same(&x.last_edge) {
            BRepTool::pnt(&topexp::last_vertex(e, true))?
        } else {
            let fpar = int_par(&cc, r.first_pnt)?;
            let lpar = int_par(&cc, r.last_pnt)?;
            if fpar > lpar {
                cc = cc
                    .get()
                    .ok_or_else(|| no_such_object("BRepFeat_RibSlot sliding base", "null curve"))?
                    .reversed()?;
            }
            r.last_pnt
        };
        let v1 = if base.the_previous_edge.is_null() {
            vertex(r.first_pnt)?
        } else {
            topexp::last_vertex(&base.the_previous_edge, true)
        };
        let ee1 = BRepLibMakeEdge::from_curve_vertices(&cc, &v1, &vertex(pt)?)?.edge()?;
        let ee1 = base.push(ee1, Some(e.orientation()))?;
        self.rib
            .lf_map
            .try_bound(e.clone().into_shape(), List::default())
            .append(ee1.into_shape());
        base.counter += 1;
        Ok(())
    }

    /// The edge `e` of the wire after the first one (the last one ends at the last
    /// point); true when it is the last one.
    fn base_next_edge(&mut self, e: &Edge, x: &Extremes, base: &mut Base) -> Result<bool, Failure> {
        let (ccc, _, _) = BRepTool::curve(e)?;
        let last = e.is_same(&x.last_edge);
        let ee = if !last {
            let v1 = if base.the_previous_edge.is_null() {
                topexp::first_vertex(e, true)
            } else {
                topexp::last_vertex(&base.the_previous_edge, true)
            };
            BRepLibMakeEdge::from_curve_vertices(&ccc, &v1, &topexp::last_vertex(e, true))?
                .edge()?
        } else if base.the_previous_edge.is_null() {
            let pf = BRepTool::pnt(&topexp::first_vertex(e, true))?;
            BRepLibMakeEdge::from_curve_points(&ccc, pf, self.rib.last_pnt)?.edge()?
        } else {
            let v1 = topexp::last_vertex(&base.the_previous_edge, true);
            BRepLibMakeEdge::from_curve_vertices(&ccc, &v1, &vertex(self.rib.last_pnt)?)?.edge()?
        };
        let ee = base.push(ee, Some(e.orientation()))?;
        self.rib
            .lf_map
            .try_bound(e.clone().into_shape(), List::default())
            .append(ee.into_shape());
        base.counter += 1;
        Ok(last)
    }

    /// The edges of the wire from the first edge to the last one.
    fn base_wire_edges(&mut self, x: &Extremes, base: &mut Base) -> Result<(), Failure> {
        let mut ex1 = WireExplorer::with_wire(&self.rib.wire)?;
        while ex1.more() {
            let e = ex1.current().clone();
            self.rib
                .lf_map
                .try_bound(e.clone().into_shape(), List::default());
            if e.is_same(&x.first_edge) {
                self.base_first_edge(&e, x, base)?;
                ex1.next()?;
                break;
            }
            ex1.next()?;
        }
        // Case of several edges
        if x.first_edge.is_same(&x.last_edge) {
            return Ok(());
        }
        while ex1.more() {
            let e = ex1.current().clone();
            self.rib
                .lf_map
                .try_bound(e.clone().into_shape(), List::default());
            if self.base_next_edge(&e, x, base)? {
                break;
            }
            ex1.next()?;
        }
        Ok(())
    }
}

/// The sliding base of a linear (`revol` false) or revolution form: the rib data, the
/// tolerance, the section edges and the sliding faces of the form.
pub(crate) struct SlidingBase<'a> {
    pub(crate) rib: &'a mut RibSlot,
    pub(crate) tol: f64,
    pub(crate) list_of_edges: &'a mut List<Shape>,
    pub(crate) slface: &'a mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
    pub(crate) revol: bool,
}

/// The walk back along the section edges: the edges taken, the last point reached, and
/// the edge count after them.
struct Back {
    the_last_pnt: rustycad_tkmath::gp::Point3,
    counter1: i32,
    new_list: Vec<Shape>,
}

impl SlidingBase<'_> {
    /// The section edge `edg` when it starts (or ends) at the last point reached: added
    /// to the base, closed on the first vertex when it reaches the first point. Returns
    /// whether it was taken and whether it closes the base.
    fn back_edge(
        &mut self,
        edg: &Edge,
        base: &mut Base,
        back: &mut Back,
    ) -> Result<(bool, bool), Failure> {
        let (ccc, f, l) = BRepTool::curve(edg)?;
        let cc = GeomTrimmedCurve::new(&ccc, f, l, true, true)?;
        if edg.orientation() == Orientation::Reversed {
            cc.reverse()?;
        }
        let fp = cc.value(cc.first_parameter())?;
        let lp = cc.value(cc.last_parameter())?;
        let (tol, first_pnt) = (self.tol, self.rib.first_pnt);
        let mut dist = fp.distance(&back.the_last_pnt);
        let mut sens = 0;
        if dist <= tol {
            sens = 1;
        } else {
            dist = lp.distance(&back.the_last_pnt);
            if dist <= tol {
                sens = 2;
                cc.reverse()?;
            }
        }
        let first_ok = (sens == 1 && lp.distance(&first_pnt) <= tol)
            || (sens == 2 && fp.distance(&first_pnt) <= tol);
        if sens == 0 {
            return Ok((false, false));
        }
        let (fpar, lpar) = (cc.first_parameter(), cc.last_parameter());
        let cc: Handle<dyn GeomCurve> = Handle::new(cc).cast();
        let c = cc
            .get()
            .ok_or_else(|| no_such_object("BRepFeat_RibSlot sliding base", "null curve"))?;
        let eeee = if !first_ok {
            if base.the_previous_edge.is_null() {
                BRepLibMakeEdge::from_curve_params(&cc, fpar, lpar)?.edge()?
            } else {
                let v1 = topexp::last_vertex(&base.the_previous_edge, true);
                if !self.revol {
                    base.b.update_vertex_tolerance(&v1, dist)?;
                }
                BRepLibMakeEdge::from_curve_vertices(&cc, &v1, &vertex(c.value(lpar)?)?)?.edge()?
            }
        } else {
            let v1 = if base.the_previous_edge.is_null() {
                vertex(c.value(fpar)?)?
            } else {
                topexp::last_vertex(&base.the_previous_edge, true)
            };
            BRepLibMakeEdge::from_curve_vertices(&cc, &v1, &base.the_fv)?.edge()?
        };
        // the first vertex is kept at `counter == 1` (natively not the count of these)
        let the_edge = base.push(eeee, None)?;
        back.counter1 += 1;
        back.new_list.push(edg.clone().into_shape());
        back.the_last_pnt = BRepTool::pnt(&topexp::last_vertex(&the_edge, true))?;
        Ok((true, first_ok))
    }

    /// The section edges chained back from the last point to the first one; false (no
    /// sliding) when the chain breaks.
    fn back_edges(&mut self, base: &mut Base, back: &mut Back) -> Result<bool, Failure> {
        let mut list: Vec<Shape> = self.list_of_edges.iter().cloned().collect();
        let mut it = 0;
        let mut first_ok = false;
        while !first_ok {
            // natively the iterator past the end is dereferenced
            let edg = topods::edge(list.get(it).ok_or_else(|| {
                no_such_object(
                    "BRepFeat_RibSlot sliding base",
                    "the section edges do not chain",
                )
            })?)?;
            let (taken, closes) = self.back_edge(&edg, base, back)?;
            first_ok = closes;
            if taken {
                if !self.revol {
                    list.remove(it);
                }
                it = 0;
            } else {
                it += 1;
            }
        }
        *self.list_of_edges = list.into_iter().collect();
        Ok(true)
    }

    /// The sliding faces bound to the edges of the base made of their section edges.
    fn rebind_sliding_faces(
        &mut self,
        base: &Base,
        counter: i32,
        back: &Back,
    ) -> Result<(), Failure> {
        let mut slid_map: DataMap<Shape, List<Shape>, ShapeMapHasher> = DataMap::default();
        for (ii, e) in Explorer::new(&base.w, ShapeEnum::Edge, ShapeEnum::Shape)?.enumerate() {
            let ii = ii as i32 + 1;
            if ii < counter || ii > back.counter1 {
                continue;
            }
            let Some(e2) = back.new_list.get((ii - counter) as usize) else {
                continue;
            };
            for (fac, ledg) in self.slface.items() {
                for e1 in ledg.iter() {
                    if e1.is_same(e2) {
                        slid_map
                            .try_bound(fac.clone(), List::default())
                            .append(e.clone());
                    }
                }
            }
        }
        *self.slface = slid_map;
        Ok(())
    }

    /// The base of a sliding rib (the wire stays empty otherwise); `sliding` turns false
    /// when the section edges do not chain.
    pub(crate) fn sliding_base(
        &mut self,
        sliding: &mut bool,
        x: &Extremes,
    ) -> Result<Wire, Failure> {
        let b = BRepBuilder::default();
        let mut w = Wire::default();
        b.make_wire(&mut w);
        let mut base = Base {
            b,
            w,
            the_previous_edge: Edge::default(),
            the_fv: Vertex::default(),
            counter: 1,
        };
        if !*sliding || self.list_of_edges.is_empty() {
            return Ok(base.w);
        }
        self.base_wire_edges(x, &mut base)?;
        let counter = base.counter;
        let mut back = Back {
            the_last_pnt: self.rib.last_pnt,
            counter1: counter,
            new_list: Vec::new(),
        };
        *sliding = self.back_edges(&mut base, &mut back)?;
        if *sliding && back.counter1 > counter {
            self.rebind_sliding_faces(&base, counter, &back)?;
        } else {
            self.slface.clear(false);
        }
        Ok(base.w)
    }
}
