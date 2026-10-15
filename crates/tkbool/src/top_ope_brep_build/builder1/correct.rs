//! `CorrectResult2d` of `TopOpeBRepBuild_Builder1` (Builder1_2.cxx): a result solid
//! rebuilt with the edges corrected in 2d, the edges of the operands copied (on their
//! vertices) so that the operands keep their p-curves, and the new edges and section
//! splits updated with the copies.
use crate::top_ope_brep_build::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::builder::curve_indices;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedMapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{IndexedDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

/// The copies of the edges (`EdMap`).
type EdgeCopies = IndexedDataMap<Shape, Shape, ShapeMapHasher>;

/// The list with each edge replaced by its copy.
fn replaced(l: &List<Shape>, ed_map: &EdgeCopies) -> Result<List<Shape>, Failure> {
    let mut corr = List::default();
    for e in l.iter() {
        corr.append(if ed_map.contains(e) {
            ed_map.find_from_key(e)?.clone()
        } else {
            e.clone()
        });
    }
    Ok(corr)
}

impl TopOpeBRepBuildBuilder {
    /// The edge of the rebuilt wire: its known copy, its 2d correction (noted as its
    /// copy), a copy on its vertices for an operand edge (noted), else itself; oriented
    /// as the edge.
    fn corrected_edge(
        &mut self,
        e: &Shape,
        sources: &IndexedMapOfShape,
        ed_map: &mut EdgeCopies,
    ) -> Result<Shape, Failure> {
        if ed_map.contains(e) {
            return Ok(ed_map.find_from_key(e)?.oriented(e.orientation()));
        }
        let corr = &self.b1d()?.my_map_of_correct2d_edges;
        if corr.contains(e) {
            let new_e = corr.find_from_key(e)?.oriented(e.orientation());
            ed_map.add(e.clone(), new_e.clone());
            return Ok(new_e);
        }
        if !sources.contains(e) {
            return Ok(e.clone());
        }
        let mut new_e = e.empty_copied()?;
        ed_map.add(e.clone(), new_e.clone());
        let free = new_e.free()?;
        new_e.set_free(true)?;
        let bb = BRepBuilder::default();
        for v in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            bb.add(&mut new_e, &v)?;
        }
        new_e.set_free(free)?;
        Ok(new_e.oriented(e.orientation()))
    }

    /// A copy of a shape filled with the corrected copies of its sub-shapes of a kind,
    /// oriented as it and closed by its closure for wires and shells.
    fn rebuilt(
        &mut self,
        s: &Shape,
        kind: ShapeEnum,
        sources: &IndexedMapOfShape,
        ed_map: &mut EdgeCopies,
    ) -> Result<Shape, Failure> {
        let mut copy = s.empty_copied()?;
        let bb = BRepBuilder::default();
        for sub in Explorer::new(s, kind, ShapeEnum::Shape)? {
            let c = match kind {
                ShapeEnum::Edge => self.corrected_edge(&sub, sources, ed_map)?,
                ShapeEnum::Wire => self.rebuilt(&sub, ShapeEnum::Edge, sources, ed_map)?,
                ShapeEnum::Face => self.rebuilt(&sub, ShapeEnum::Wire, sources, ed_map)?,
                _ => self.rebuilt(&sub, ShapeEnum::Face, sources, ed_map)?,
            };
            bb.add(&mut copy, &c)?;
        }
        copy.set_orientation(s.orientation());
        if matches!(kind, ShapeEnum::Edge | ShapeEnum::Face) {
            copy.set_closed(BRepTool::is_closed(&copy)?)?;
        }
        Ok(copy)
    }

    /// `CorrectResult2d(aResult)` (Builder1): with edges corrected in 2d, the result
    /// solid rebuilt shell by shell with the corrected edges and the copies of the
    /// operand edges, the new edges of the curves and the splits of the section edges
    /// replaced by the copies; returns 1 then, else 0; `b.correct_result2d(&mut s)?`.
    pub fn correct_result2d(&mut self, a_result: &mut Shape) -> Result<i32, Failure> {
        if self.b1d()?.my_map_of_correct2d_edges.extent() == 0
            || a_result.shape_type()? != ShapeEnum::Solid
        {
            return Ok(0);
        }
        let mut sources = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&self.my_shape1, ShapeEnum::Edge, &mut sources)?;
        topexp::map_shapes_of_type(&self.my_shape2, ShapeEnum::Edge, &mut sources)?;
        let mut ed_map = EdgeCopies::default();
        let mut solid = a_result.empty_copied()?;
        let bb = BRepBuilder::default();
        for shell in Explorer::new(a_result, ShapeEnum::Shell, ShapeEnum::Shape)? {
            let s = self.rebuilt(&shell, ShapeEnum::Face, &sources, &mut ed_map)?;
            bb.add(&mut solid, &s)?;
        }
        *a_result = solid;
        let ics = curve_indices(self.hds()?, true);
        for ic in ics {
            let corr = replaced(self.change_new_edges(ic), &ed_map)?;
            *self.change_new_edges(ic) = corr;
        }
        let n = self.hds()?.ds().nb_section_edges();
        for i in 1..=n {
            let es: Shape = self.hds()?.ds().section_edge(i, true)?.into();
            if es.is_null() {
                continue;
            }
            for st in [State::In, State::Out, State::On] {
                let corr = replaced(self.change_split(&es, st), &ed_map)?;
                *self.change_split(&es, st) = corr;
            }
        }
        Ok(1)
    }
}
