// Rust translation of BRepFill_Draft::Fuse
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Draft.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Boolean operation between the draft surface and the stop shape: the part of the
//! draft before the stop, with the history of its faces and sections.
use super::{BRepFillDraft, null};
use crate::brep_fill::location_law::BRepFillLocationLaw as _;
use crate::brep_fill::shape_table::{ShapeTable, cell, cell_mut};
use rustycad_tkbo::bop_algo::{BOPAlgoBuilder, BOPAlgoPaveFiller, Glue, Operation};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPISection;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{History, HistoryAlgorithm};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Shell, Solid};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tktopalgo::brep_extrema::BRepExtremaDistShapeShape;
use std::f64::consts::PI;

/// The history relations of a General Fuse builder (`BRepTools_History::Merge(theArguments,
/// theAlgo)` reads them from the algorithm).
struct BuilderRelations<'a>(&'a BOPAlgoBuilder);

impl HistoryAlgorithm for BuilderRelations<'_> {
    fn is_deleted(&mut self, shape: &Shape) -> Result<bool, Failure> {
        self.0.is_deleted(shape)
    }
    fn modified(&mut self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        self.0.modified(shape)
    }
    fn generated(&mut self, shape: &Shape) -> Result<Vec<Shape>, Failure> {
        self.0.generated(shape)
    }
}

/// The Boolean operation of the states (Fuse OUT/OUT, Common IN/IN, Cut OUT/IN, Cut21
/// IN/OUT).
fn operation_of(state1: State, state2: State) -> Operation {
    match (state1, state2) {
        (State::In, State::In) => Operation::Common,
        (State::Out, State::In) => Operation::Cut,
        (State::In, State::Out) => Operation::Cut21,
        _ => Operation::Fuse,
    }
}

/// A list of one shape.
fn one(shape: &Shape) -> List<Shape> {
    let mut list: List<Shape> = List::default();
    list.append(shape.clone());
    list
}

/// The solid of the stop shape (the first child of a compound is used instead, by
/// recursion of the caller); `None` when the shape cannot be one.
fn stop_solid(stop_shape: &Shape) -> Result<Option<Solid>, Failure> {
    let builder = BRepBuilder::default();
    let mut sol2 = Solid::default();
    match stop_shape.shape_type()? {
        ShapeEnum::Solid => return Ok(Some(topods::solid(stop_shape)?)),
        ShapeEnum::Shell => {
            builder.make_solid(&mut sol2);
            builder.add(&mut sol2, stop_shape)?;
        }
        ShapeEnum::Face => {
            let mut s = Shell::default();
            builder.make_shell(&mut s);
            builder.add(&mut s, stop_shape)?;
            s.set_closed(BRepTool::is_closed(&s)?)?;
            builder.make_solid(&mut sol2);
            builder.add(&mut sol2, &s)?;
        }
        _ => return Ok(None),
    }
    Ok(Some(sol2))
}

/// The distance from the wire to a shape, when computed.
fn distance_to(
    dist_tool: &mut BRepExtremaDistShapeShape,
    shape: &Shape,
) -> Result<Option<f64>, Failure> {
    dist_tool.load_s2(shape)?;
    dist_tool.perform(&ProgressRange::default())?;
    if !dist_tool.is_done() {
        return Ok(None);
    }
    Ok(Some(dist_tool.value()?))
}

impl BRepFillDraft {
    /// `Fuse(StopShape, KeepOutSide)` (private): the draft (as a solid) and the stop
    /// shape intersected and the part kept by the states (the side of the stop normal
    /// at the section edge nearest to the wire for a non-solid stop), the faces and
    /// sections of the draft replaced by their images; false when the operation fails.
    pub(super) fn fuse(
        &mut self,
        stop_shape: &Shape,
        keep_out_side: bool,
    ) -> Result<bool, Failure> {
        let builder = BRepBuilder::default();
        let issolid: bool = self.my_shape.shape_type()? == ShapeEnum::Solid;
        let sol1: Solid = if issolid {
            topods::solid(&self.my_shape)?
        } else {
            let mut sol1 = Solid::default();
            builder.make_solid(&mut sol1);
            builder.add(&mut sol1, &self.my_shape)?;
            sol1
        };
        if stop_shape.shape_type()? == ShapeEnum::Compound {
            return self.fuse(
                ShapeIterator::new(stop_shape, true, true).value()?,
                keep_out_side,
            );
        }
        let Some(sol2) = stop_solid(stop_shape)? else {
            return Ok(false);
        };
        let (sol1, sol2): (Shape, Shape) = (sol1.into_shape(), sol2.into_shape());
        let mut a_pf = BOPAlgoPaveFiller::new();
        let mut an_args: List<Shape> = one(&sol1);
        an_args.append(sol2.clone());
        a_pf.set_arguments(&an_args);
        a_pf.perform();
        if a_pf.options().has_errors() {
            return Ok(false);
        }
        let mut a_sec = BRepAlgoAPISection::from_shapes(&sol1, &sol2, Some(a_pf), true)?;
        let a_section: Shape = a_sec.shape().clone();
        let mut a_pf = a_sec.take_ds_filler().ok_or_else(|| null("pave filler"))?;
        if !Explorer::new(&a_section, ShapeEnum::Edge, ShapeEnum::Shape)?.more() {
            return Ok(false);
        }
        let mut state1 = State::Out;
        if stop_shape.shape_type()? != ShapeEnum::Solid {
            state1 = self.state_by_geometry(&a_section)?.unwrap_or(state1);
        }
        let state2: State = if keep_out_side { State::Out } else { State::In };
        let mut a_builder = BOPAlgoBuilder::new();
        a_builder.add_argument(&sol1);
        a_builder.add_argument(&sol2);
        a_builder.perform_with_filler(&mut a_pf);
        if a_builder.options().has_errors() {
            return Ok(false);
        }
        let a_history = History::new();
        let (lo, lt) = (one(&sol1), one(&sol2));
        let mut result: Option<Shape> = None;
        if state1 == State::Out {
            result = self.cut_then_glue(
                &mut a_builder,
                &a_pf,
                (&lo, &lt),
                (state1, state2),
                &a_history,
            )?;
        }
        let result: Shape = match result {
            Some(result) => result,
            None => {
                a_history.clear();
                a_builder.build_bop_operation_with_filler(
                    &a_pf,
                    &lo,
                    &lt,
                    operation_of(state1, state2),
                    None,
                )?;
                if a_builder.options().has_errors() {
                    return Ok(false);
                }
                if let Some(h) = a_builder.history() {
                    a_history.merge_handle(&h)?;
                }
                a_builder.shape().clone()
            }
        };
        self.take_result(result, issolid)?;
        self.update_history(&a_history)?;
        Ok(true)
    }

    /// The state of the draft by the stop normal at the section edge nearest to the
    /// wire (IN when the normal goes along the draft direction).
    fn state_by_geometry(&self, a_section: &Shape) -> Result<Option<State>, Failure> {
        let mut a_se_min = Edge::default();
        let mut dmin: f64 = precision::infinite();
        let mut dist_tool = BRepExtremaDistShapeShape::new();
        dist_tool.load_s1(self.my_wire.as_ref())?;
        for a_se in Explorer::new(a_section, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if let Some(d) = distance_to(&mut dist_tool, &a_se)?
                && d < dmin
            {
                dmin = d;
                a_se_min = topods::edge(&a_se)?;
                if dmin < precision::confusion() {
                    break;
                }
            }
        }
        if a_se_min.is_null() {
            return Ok(None);
        }
        let on_surface = BRepTool::curve_on_surface_by_index(&a_se_min, 2)?;
        let c2d = on_surface
            .pcurve
            .get()
            .ok_or_else(|| null("section p-curve"))?;
        let (f, l) = (on_surface.first, on_surface.last);
        let mut sp = SLProps::new(on_surface.surface.clone(), 1, 1.0e-12)?;
        let mut normal_defined = false;
        for t in [(f + l) / 2., (3. * f + l) / 4., (f + 3. * l) / 4.] {
            let p2d = c2d.d0(t)?;
            sp.set_parameters(p2d.x(), p2d.y())?;
            normal_defined = sp.is_normal_defined()?;
            if normal_defined {
                break;
            }
        }
        if !normal_defined {
            return Ok(None);
        }
        Ok(Some(if self.my_dir.angle(&sp.normal()?) < PI / 2. {
            State::In
        } else {
            State::Out
        }))
    }

    /// The cut of the draft by the stop (keeping the part nearest to the wire), glued
    /// back with the stop and operated with the states; `None` when the single
    /// operation is needed instead.
    fn cut_then_glue(
        &self,
        a_builder: &mut BOPAlgoBuilder,
        a_pf: &BOPAlgoPaveFiller,
        (lo, lt): (&List<Shape>, &List<Shape>),
        (state1, state2): (State, State),
        a_history: &History,
    ) -> Result<Option<Shape>, Failure> {
        a_builder.build_bop_operation_with_filler(a_pf, lo, lt, Operation::Cut, None)?;
        if a_builder.options().has_errors() {
            return Ok(None);
        }
        let Some(a_cut_min) = self.nearest_cut(a_builder.shape())? else {
            return Ok(None);
        };
        let arguments: Vec<Shape> = lo.iter().cloned().collect();
        a_history.merge_algorithm(&arguments, &mut BuilderRelations(a_builder))?;
        let mut a_gluer = BOPAlgoBuilder::new();
        a_gluer.add_argument(&a_cut_min);
        a_gluer.add_argument(lt.first().ok_or_else(|| null("stop solid"))?);
        a_gluer.set_glue(Glue::Shift);
        a_gluer.perform();
        a_gluer.build_bop(&one(&a_cut_min), state1, lt, state2, None)?;
        if a_gluer.options().has_errors() {
            return Ok(None);
        }
        if let Some(h) = a_gluer.history() {
            a_history.merge_handle(&h)?;
        }
        let result: Shape = a_gluer.shape().clone();
        if !Explorer::new(&result, ShapeEnum::Solid, ShapeEnum::Shape)?.more() {
            return Ok(None);
        }
        Ok(Some(result))
    }

    /// The solid of the cut nearest to the wire (the first one when it is alone).
    fn nearest_cut(&self, cut: &Shape) -> Result<Option<Shape>, Failure> {
        let solids: Vec<Shape> = Explorer::new(cut, ShapeEnum::Solid, ShapeEnum::Shape)?.collect();
        let Some(first) = solids.first() else {
            return Ok(None);
        };
        let mut a_cut_min: Shape = first.clone();
        if solids.len() > 1 {
            let mut a_d_min: f64 = precision::infinite();
            let mut dist_tool = BRepExtremaDistShapeShape::new();
            dist_tool.load_s1(self.my_wire.as_ref())?;
            for a_cut in &solids {
                if let Some(d) = distance_to(&mut dist_tool, a_cut)?
                    && d < a_d_min
                {
                    a_d_min = d;
                    a_cut_min = a_cut.clone();
                }
            }
        }
        Ok(Some(a_cut_min))
    }

    /// The draft becomes the result (its first shell when the draft was not a solid).
    fn take_result(&mut self, result: Shape, issolid: bool) -> Result<(), Failure> {
        if issolid {
            self.my_shape = result;
            return Ok(());
        }
        let exp = Explorer::new(&result, ShapeEnum::Shell, ShapeEnum::Shape)?;
        if exp.more() {
            self.my_shape = exp.current().clone();
        }
        Ok(())
    }

    /// The faces and sections of the draft replaced by their first modified shapes.
    fn update_history(&mut self, a_history: &History) -> Result<(), Failure> {
        let nb_law: i32 = self.loc()?.nb_law()?;
        let faces: &mut ShapeTable = self.my_faces.as_mut().ok_or_else(|| null("face table"))?;
        for ii in 1..=nb_law {
            replace_by_modified(a_history, faces, ii)?;
        }
        let sections: &mut ShapeTable = self
            .my_sections
            .as_mut()
            .ok_or_else(|| null("section table"))?;
        for ii in 1..=nb_law + 1 {
            replace_by_modified(a_history, sections, ii)?;
        }
        Ok(())
    }
}

/// The shape `(1, ii)` of the table replaced by its first modified shape, if any.
fn replace_by_modified(
    a_history: &History,
    table: &mut ShapeTable,
    ii: i32,
) -> Result<(), Failure> {
    let l: Vec<Shape> = a_history.modified(cell(table, 1, ii)?)?;
    if let Some(first) = l.first() {
        *cell_mut(table, 1, ii)? = first.clone();
    }
    Ok(())
}
