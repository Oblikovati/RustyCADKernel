// Rust translation of BRepFill_PipeShell::Prepare, Place and ResetLoc and of the static
// ComputeSection and PerformTransition (ModelingAlgorithms/TKBool/BRepFill/
// BRepFill_PipeShell.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The laws of a pipe shell: the location law transformed for the transition and the
//! section law of the placed profiles.
use super::{BRepFillPipeShell, null, trihedron_law};
use crate::brep_fill::compatible_wires::BRepFillCompatibleWires;
use crate::brep_fill::location_law::BRepFillLocationLaw;
use crate::brep_fill::section::BRepFillSection;
use crate::brep_fill::section_law::{BRepFillNSections, BRepFillSectionLaw, BRepFillShapeLaw};
use crate::brep_fill::section_placement::BRepFillSectionPlacement;
use crate::brep_fill::transition_style::TransitionStyle;
use crate::brep_fill::trim_shell_corner::HistoryMap;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillCorrectedFrenet, GeomFillLocationGuide, GeomFillPipeError, GeomFillTrihedron,
    GeomFillTrihedronLaw,
};
use rustycad_tkmath::gp::Transform3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};

/// `ComputeSection(W1, W2, p1, p2, Wres)`: the section between two wires made
/// compatible, at the ratio `p1 / (p1 + p2)`; StdFail_NotDone when they are not.
fn compute_section(w1: &Wire, w2: &Wire, p1: f64, p2: f64) -> Result<Shape, Failure> {
    let mut cw =
        BRepFillCompatibleWires::with_sections(&[w1.as_ref().clone(), w2.as_ref().clone()]);
    cw.set_percent(0.1);
    cw.perform(true)?;
    if !cw.is_done() {
        return Err(Failure::with_type(
            "StdFail_NotDone",
            "Uncompatible wires",
            "BRepFill_PipeShell",
        ));
    }
    let sl = BRepFillNSections::with_placements(cw.shape(), &[], &[0., 1.], 0., 1., true)?;
    sl.d0(p1 / (p1 + p2))
}

/// `PerformTransition(Mode, Loc, angmin)`: the transformations of the location law
/// deleted, then the law made G0 (modified transition) or compatible.
fn perform_transition(
    mode: TransitionStyle,
    loc: &Handle<dyn BRepFillLocationLaw>,
    angmin: f64,
) -> Result<(), Failure> {
    let Some(loc) = loc.get() else {
        return Ok(());
    };
    loc.delete_transform()?;
    if mode == TransitionStyle::Modified {
        return loc.transform_in_g0_law();
    }
    loc.transform_in_compatible_law(angmin)
}

/// Whether every edge of the wire is degenerated (true for an empty wire).
fn all_degenerated(w: &Wire, mut wdeg: bool) -> Result<bool, Failure> {
    let mut iter = ShapeIterator::new(w, true, true);
    while iter.more() {
        wdeg = wdeg && BRepTool::degenerated(&topods::edge(iter.value()?)?)?;
        iter.next();
    }
    Ok(wdeg)
}

/// The placed sections sorted by their parameters, with their original indices.
struct PlacedSections {
    param: Vec<f64>,
    ind_sec: Vec<i32>,
    transformations: Vec<Transform3>,
}

impl BRepFillPipeShell {
    /// `Prepare()` (private): the location law (corrected Frenet when none is set) and
    /// the section law of the placed profiles (a shape law for one profile, else the
    /// N-sections of the compatible wires, looped on a closed spine); StdFail_NotDone
    /// without profile, Standard_Failure for a punctual section inside the sequence or
    /// for punctual sections only.
    pub(super) fn prepare(&mut self) -> Result<(), Failure> {
        self.w_seq.clear();
        self.my_edge_new_edges.clear(true);
        if !self.is_ready() {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                "PipeShell",
                "BRepFill_PipeShell",
            ));
        }
        if !self.my_location.is_null() && !self.my_section.is_null() {
            return Ok(());
        }
        self.check_punctual_sections()?;
        if self.my_location.is_null() {
            if self.my_trihedron != GeomFillTrihedron::IsCorrectedFrenet {
                return Err(Failure::with_type(
                    "Standard_ConstructionError",
                    "PipeShell",
                    "BRepFill_PipeShell",
                ));
            }
            let tlaw: Handle<dyn GeomFillTrihedronLaw> =
                Handle::new(GeomFillCorrectedFrenet::new()).cast();
            self.my_location = trihedron_law(&self.my_spine, tlaw)?;
        }
        perform_transition(self.my_transition, &self.my_location, self.angmin)?;
        if self.my_seq.len() == 1 {
            self.single_section_law()?;
        } else {
            self.multi_section_law()?;
        }
        if matches!(
            self.my_trihedron,
            GeomFillTrihedron::IsGuidePlanWithContact | GeomFillTrihedron::IsGuideACWithContact
        ) {
            self.force_guide_rotation()?;
        }
        self.my_status = self.location()?.get_status()?;
        if !self.section()?.is_done() {
            self.my_status = GeomFillPipeError::PipeNotOk;
        }
        Ok(())
    }

    /// Standard_Failure for punctual sections inside the sequence, or when all of at
    /// most two sections are punctual.
    fn check_punctual_sections(&self) -> Result<(), Failure> {
        let wrong = || {
            Failure::with_type(
                "Standard_Failure",
                "Wrong usage of punctual sections",
                "BRepFill_PipeShell",
            )
        };
        let n: usize = self.my_seq.len();
        for section in self.my_seq.iter().take(n.saturating_sub(1)).skip(1) {
            if all_degenerated(section.wire(), true)? {
                return Err(wrong());
            }
        }
        if n <= 2 {
            let mut wdeg: bool = true;
            for section in &self.my_seq {
                wdeg = all_degenerated(section.wire(), wdeg)?;
            }
            if wdeg {
                return Err(wrong());
            }
        }
        Ok(())
    }

    /// The shape law of the only placed profile (scaled by the law when it has one);
    /// each of its edges is its own split.
    fn single_section_law(&mut self) -> Result<(), Failure> {
        let (the_sect, _, _) = self.place(&self.my_seq[0])?;
        let law: Handle<dyn BRepFillSectionLaw> = if self.my_seq[0].is_law() {
            Handle::new(BRepFillShapeLaw::with_wire_and_law(
                &the_sect,
                &self.my_law,
                true,
            )?)
            .cast()
        } else {
            Handle::new(BRepFillShapeLaw::with_wire(&the_sect, true)?).cast()
        };
        self.my_section = law;
        self.w_seq.push(the_sect.as_ref().clone());
        self.my_ind_of_sec.push(1);
        let mut itw = ShapeIterator::new(&the_sect, true, true);
        while itw.more() {
            let an_edge: Shape = itw.value()?.clone();
            let mut elist: List<Shape> = List::default();
            elist.append(an_edge.clone());
            self.my_edge_new_edges.bind(an_edge, elist);
            itw.next();
        }
        Ok(())
    }

    /// The N-sections law of the placed profiles: looped on a closed spine, sorted by
    /// parameter, made compatible (their edge splits recorded).
    fn multi_section_law(&mut self) -> Result<(), Failure> {
        let nb_l: i32 = self.location()?.nb_law()?;
        let (_, v2) = self.location()?.curvilinear_bounds(nb_l)?;
        let v1: f64 = 0.;
        let mut placed = self.place_all((v1, v2))?;
        self.sort_sections(&mut placed)?;
        for ii in 1..=self.my_seq.len() as i32 {
            if let Some(jj) = placed.ind_sec.iter().position(|&i| i == ii) {
                self.my_ind_of_sec.push(jj as i32 + 1);
            }
        }
        let mut georges = BRepFillCompatibleWires::with_sections(&self.w_seq);
        georges.set_percent(0.1);
        georges.perform(false)?;
        if !georges.is_done() {
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                "PipeShell : uncompatible wires",
                "BRepFill_PipeShell",
            ));
        }
        let working_sections: Vec<Shape> = georges.shape().to_vec();
        record_splits(&self.w_seq, &georges, &mut self.my_edge_new_edges)?;
        let law = BRepFillNSections::with_placements(
            &working_sections,
            &placed.transformations,
            &placed.param,
            v1,
            v2,
            true,
        )?;
        self.my_section = Handle::new(law).cast();
        Ok(())
    }

    /// The profiles placed in the local frame (`Place`), then on a closed spine the
    /// sections looping the ends: the first or last one repeated, else a medium section
    /// imposed at both ends.
    fn place_all(&mut self, (v1, v2): (f64, f64)) -> Result<PlacedSections, Failure> {
        let mut placed = PlacedSections {
            param: Vec::new(),
            ind_sec: Vec::new(),
            transformations: Vec::new(),
        };
        let (mut ideb, mut ifin) = (0usize, 0usize);
        for iseq in 1..=self.my_seq.len() {
            placed.ind_sec.push(iseq as i32);
            let (the_sect, a_trsf, param) = self.place(&self.my_seq[iseq - 1])?;
            placed.param.push(param);
            self.w_seq.push(the_sect.into_shape());
            placed.transformations.push(a_trsf);
            if param == v1 {
                ideb = iseq;
            }
            if param == v2 {
                ifin = iseq;
            }
        }
        if self.location()?.is_closed()? {
            self.loop_sections(&mut placed, (v1, v2), (ideb, ifin))?;
        }
        Ok(placed)
    }

    /// The looping sections of a closed spine.
    fn loop_sections(
        &mut self,
        placed: &mut PlacedSections,
        (v1, v2): (f64, f64),
        (ideb, ifin): (usize, usize),
    ) -> Result<(), Failure> {
        if ideb > 0 {
            placed.param.push(v2);
            self.w_seq.push(self.w_seq[ideb - 1].clone());
            return Ok(());
        }
        if ifin > 0 {
            placed.param.push(v1);
            self.w_seq.push(self.w_seq[ifin - 1].clone());
            return Ok(());
        }
        let (mut pmin, mut pmax) = (f64::MAX, f64::MIN);
        let (mut wmin, mut wmax) = (Wire::default(), Wire::default());
        for (iseq, w) in self.w_seq.iter().enumerate() {
            let p: f64 = placed
                .param
                .get(iseq)
                .copied()
                .ok_or_else(|| null("section parameter"))?;
            if p < pmin {
                pmin = p;
                wmin = topods::wire(w)?;
            }
            if p > pmax {
                pmax = p;
                wmax = topods::wire(w)?;
            }
        }
        let wres: Shape = compute_section(&wmin, &wmax, (pmin - v1).abs(), (pmax - v2).abs())?;
        for v in [v1, v2] {
            placed.param.push(v);
            self.w_seq.push(wres.clone());
            placed.ind_sec.push(self.w_seq.len() as i32);
        }
        Ok(())
    }

    /// The sections sorted by increasing parameter with the native exchange sort; the
    /// index sequence is shorter when an end section was repeated, and exchanging past
    /// its end is Standard_OutOfRange, as natively.
    fn sort_sections(&mut self, placed: &mut PlacedSections) -> Result<(), Failure> {
        let n: usize = self.w_seq.len();
        let mut play_again: bool = true;
        while play_again {
            play_again = false;
            for iseq in 0..n {
                for jseq in iseq + 1..n {
                    if at(&placed.param, iseq)? > at(&placed.param, jseq)? {
                        exchange(&mut placed.param, iseq, jseq)?;
                        exchange(&mut self.w_seq, iseq, jseq)?;
                        exchange(&mut placed.ind_sec, iseq, jseq)?;
                        play_again = true;
                    }
                }
            }
        }
        Ok(())
    }

    /// The rotation of the guide laws forced along the section (contact modes).
    fn force_guide_rotation(&self) -> Result<(), Failure> {
        let loc = self.location()?;
        let sec = self.section()?.concatened_law()?;
        let (_, length) = loc.curvilinear_bounds(loc.nb_law()?)?;
        let (fs, l) = sec.get().ok_or_else(|| null("section law"))?.get_domain()?;
        let delta: f64 = (l - fs) / length;
        let mut old_angle: f64 = 0.;
        for ipath in 1..=loc.nb_law()? {
            let (f, l) = loc.curvilinear_bounds(ipath)?;
            let guide: Handle<GeomFillLocationGuide> = loc.law(ipath)?.cast();
            let guide = guide.get().ok_or_else(|| null("guide location law"))?;
            old_angle = guide.set(&sec, true, fs + f * delta, fs + l * delta, old_angle)?;
        }
        Ok(())
    }

    /// `Place(Sec, W, Trsf, param)` (private): the copy of the section wire moved to the
    /// local frame, the transformation and the abscissa of the section on the path.
    pub(super) fn place(&self, sec: &BRepFillSection) -> Result<(Wire, Transform3, f64), Failure> {
        let place = BRepFillSectionPlacement::with_vertex(
            &self.my_location,
            sec.wire(),
            sec.vertex(),
            sec.with_contact(),
            sec.with_correction(),
        )?;
        let a_trsf: Transform3 = *place.transformation();
        let mut transform = BRepBuilderAPITransform::with_shape(sec.wire(), a_trsf, true, false)?;
        let w: Wire = topods::wire(transform.shape()?)?;
        Ok((w, a_trsf, place.abscissa_on_path()?))
    }

    /// `ResetLoc()` (private): the rotations of the guide laws erased (contact modes).
    pub(super) fn reset_loc(&self) -> Result<(), Failure> {
        if !matches!(
            self.my_trihedron,
            GeomFillTrihedron::IsGuidePlanWithContact | GeomFillTrihedron::IsGuideACWithContact
        ) {
            return Ok(());
        }
        let loc = self.location()?;
        for isec in 1..=loc.nb_law()? {
            let guide: Handle<GeomFillLocationGuide> = loc.law(isec)?.cast();
            guide
                .get()
                .ok_or_else(|| null("guide location law"))?
                .erase_rotation();
        }
        Ok(())
    }
}

/// Standard_OutOfRange for an index past the end of a sequence.
fn sequence_error(index: usize, len: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!(
            "BRepFill_PipeShell: sequence index {}; expected 1..={len}",
            index + 1
        ),
        "NCollection_Sequence",
    )
}

/// The item `index` (0-based) of a sequence.
fn at(v: &[f64], index: usize) -> Result<f64, Failure> {
    v.get(index)
        .copied()
        .ok_or_else(|| sequence_error(index, v.len()))
}

/// `NCollection_Sequence::Exchange` of two 0-based items.
fn exchange<T>(v: &mut [T], i: usize, j: usize) -> Result<(), Failure> {
    let len: usize = v.len();
    if i >= len || j >= len {
        return Err(sequence_error(i.max(j), len));
    }
    v.swap(i, j);
    Ok(())
}

/// The splits of every edge of the sections, from the compatible wires.
fn record_splits(
    w_seq: &[Shape],
    georges: &BRepFillCompatibleWires,
    map: &mut HistoryMap,
) -> Result<(), Failure> {
    for w in w_seq {
        for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let an_edge = topods::edge(&e)?;
            let a_new_edges: List<Shape> = georges.generated_shapes(&an_edge).clone();
            map.bind(e, a_new_edges);
        }
    }
    Ok(())
}
