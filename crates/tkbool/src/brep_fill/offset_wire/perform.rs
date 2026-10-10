// Rust translation of BRepFill_OffsetWire::Perform
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The offset at a distance: the edges too curved for it replaced by polylines (with a
//! new bisecting locus), any failure caught as not done, and a closed result checked.
use super::BRepFillOffsetWire;
use super::bad_edges::{check_bad_edges, perform_curve};
use super::kpart::k_part_circle;
use super::spine::edge_vertices;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::Substitution;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeVertex};
use rustycad_tktopalgo::brep_mat2d::{
    BRepMAT2dBisectingLocus, BRepMAT2dExplorer, BRepMAT2dLinkTopoBilo,
};
use rustycad_tktopalgo::mat::MatSide;

/// The polyline replacing a bad edge: its points within 1% of the offset, from its
/// first to its last vertex (one segment for two points).
fn polyline_of(an_e: &Shape, a_defl: f64) -> Result<Vec<Shape>, Failure> {
    let e: Edge = topods::edge(an_e)?;
    let (vf, vl) = topexp::vertices(&e, false);
    let (g3d, f, l) = BRepTool::curve(&e)?;
    let ac = GeomAdaptorCurve::with_range(&g3d, f, l)?;
    let points = perform_curve(&ac, a_defl, (f, l), precision::confusion(), 2)?.points;
    let mut al: Vec<Shape> = Vec::new();
    if points.len() <= 2 {
        al.push(
            BRepLibMakeEdge::from_vertices(&vf, &vl)?
                .edge()?
                .into_shape(),
        );
        return Ok(al);
    }
    let mut fv: Vertex = vf;
    for lp in &points[1..points.len() - 1] {
        let lv: Vertex = BRepLibMakeVertex::new(*lp)?.vertex()?;
        al.push(
            BRepLibMakeEdge::from_vertices(&fv, &lv)?
                .edge()?
                .into_shape(),
        );
        fv = lv;
    }
    al.push(
        BRepLibMakeEdge::from_vertices(&fv, &vl)?
            .edge()?
            .into_shape(),
    );
    Ok(al)
}

impl BRepFillOffsetWire {
    /// `Perform(Offset, Alt)` (natively defaulted to Alt 0): the offset wire at the
    /// distance, moved by `Alt` along the normal; any failure leaves a null shape not
    /// done; Standard_ConstructionError when a wire of a closed result is open;
    /// `ow.perform(0.5, 0.)?`.
    pub fn perform(&mut self, offset: f64, alt: f64) -> Result<(), Failure> {
        match self.perform_guarded(offset, alt) {
            Err(_) => {
                self.my_shape = Shape::default();
                self.my_is_done = false;
                return Ok(());
            }
            Ok(false) => return Ok(()),
            Ok(true) => {}
        }
        if self.my_is_open_result {
            return Ok(());
        }
        let mut an_exp = Explorer::new(&self.my_shape, ShapeEnum::Wire, ShapeEnum::Shape)?;
        while an_exp.more() {
            if !an_exp.current().closed()? {
                self.my_shape = Shape::default();
                self.my_is_done = false;
                return Err(Failure::with_type(
                    "Standard_ConstructionError",
                    "Offset wire is not closed.",
                    "BRepFill_OffsetWire",
                ));
            }
            an_exp.next()?;
        }
        Ok(())
    }

    /// The body of the native try block; false for its early returns (the particular
    /// case and a failed new locus), which skip the closure check.
    fn perform_guarded(&mut self, offset: f64, alt: f64) -> Result<bool, Failure> {
        self.my_call_gen = false;
        let work = self.my_work_spine.clone();
        if k_part_circle(
            &work,
            offset,
            self.my_is_open_result,
            alt,
            (&mut self.my_shape, &mut self.my_map, &mut self.my_is_done),
        )? {
            return Ok(false);
        }
        let old_work_spine: Face = self.my_work_spine.clone();
        let bad_edges = check_bad_edges(&self.my_work_spine, offset, &self.my_bilo, &self.my_link)?;
        if bad_edges.is_empty() {
            self.perform_with_own_bilo(offset, alt)?;
            return Ok(true);
        }
        let mut a_subst = Substitution::new();
        let a_defl: f64 = 0.01 * offset.abs();
        for an_e in &bad_edges {
            let al: Vec<Shape> = polyline_of(an_e, a_defl)?;
            self.remap_spine(an_e, &al)?;
            a_subst.substitute(an_e, &al)?;
        }
        let Some(new_work) = self.substituted_work_spine(&mut a_subst)? else {
            self.perform_with_own_bilo(offset, alt)?;
            return Ok(true);
        };
        self.my_work_spine = new_work;
        let done: bool = self.perform_with_new_bilo(offset, alt)?;
        if done {
            // A failed new locus returns before the work spine is restored.
            self.my_work_spine = old_work_spine;
        }
        Ok(done)
    }

    /// PerformWithBiLo on the work spine with the locus and links of Init.
    fn perform_with_own_bilo(&mut self, offset: f64, alt: f64) -> Result<(), Failure> {
        let bilo = std::mem::take(&mut self.my_bilo);
        let mut link = std::mem::replace(&mut self.my_link, BRepMAT2dLinkTopoBilo::new());
        let work = self.my_work_spine.clone();
        let result =
            self.perform_with_bilo(&work, offset, &bilo, &mut link, self.my_join_type, alt);
        (self.my_bilo, self.my_link) = (bilo, link);
        result
    }

    /// PerformWithBiLo with a new locus of the substituted work spine; false (null
    /// shape, not done) when the locus fails.
    fn perform_with_new_bilo(&mut self, offset: f64, alt: f64) -> Result<bool, Failure> {
        let mut new_exp = BRepMAT2dExplorer::new();
        new_exp.perform(&self.my_work_spine)?;
        let mut new_bilo = BRepMAT2dBisectingLocus::new();
        let mut new_link = BRepMAT2dLinkTopoBilo::new();
        new_bilo.compute(
            &mut new_exp,
            1,
            MatSide::Left,
            self.my_join_type,
            self.my_is_open_result,
        )?;
        if !new_bilo.is_done() {
            self.my_shape = Shape::default();
            self.my_is_done = false;
            return Ok(false);
        }
        new_link.perform(&new_exp, &new_bilo)?;
        let work = self.my_work_spine.clone();
        self.perform_with_bilo(
            &work,
            offset,
            &new_bilo,
            &mut new_link,
            self.my_join_type,
            alt,
        )?;
        Ok(true)
    }

    /// The parts of a replaced edge (and their vertices) mapped to its spine element.
    fn remap_spine(&mut self, an_e: &Shape, al: &[Shape]) -> Result<(), Failure> {
        let Some(original) = self.my_map_spine.seek(an_e).cloned() else {
            return Ok(());
        };
        for new in al {
            let new_edge: Edge = topods::edge(new)?;
            self.my_map_spine
                .bind(new_edge.as_ref().clone(), original.clone());
            let (new_v1, new_v2) = edge_vertices(&new_edge);
            for v in [new_v1, new_v2] {
                if !self.my_map_spine.is_bound(v.as_ref()) {
                    self.my_map_spine.bind(v.into_shape(), original.clone());
                }
            }
        }
        self.my_map_spine.unbind(an_e);
        Ok(())
    }

    /// The work spine with its wires rebuilt by the substitution of the bad edges,
    /// `None` when nothing was copied.
    fn substituted_work_spine(&self, a_subst: &mut Substitution) -> Result<Option<Face>, Failure> {
        let mut wwmap: DataMap<Shape, Vec<Shape>, ShapeMapHasher> = DataMap::default();
        let mut itws = ShapeIterator::new(self.my_work_spine.as_ref(), true, true);
        while itws.more() {
            let a_wire = itws.value()?.clone();
            a_subst.build(&a_wire)?;
            if a_subst.is_copied(&a_wire) {
                let new_wire: Shape = a_subst
                    .copy(&a_wire)?
                    .into_iter()
                    .next()
                    .ok_or_else(no_copy)?;
                new_wire.set_closed(a_wire.closed()?)?;
                wwmap.bind(a_wire, vec![new_wire]);
            }
            itws.next();
        }
        a_subst.clear();
        for (old, new) in wwmap.items() {
            a_subst.substitute(old, new)?;
        }
        a_subst.build(self.my_work_spine.as_ref())?;
        if !a_subst.is_copied(self.my_work_spine.as_ref()) {
            return Ok(None);
        }
        let copy: Shape = a_subst
            .copy(self.my_work_spine.as_ref())?
            .into_iter()
            .next()
            .ok_or_else(no_copy)?;
        Ok(Some(topods::face(&copy)?))
    }
}

/// Standard_NoSuchObject of an empty substitution copy.
fn no_copy() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "BRepFill_OffsetWire: empty substitution copy; expected one shape",
        "NCollection_List",
    )
}
