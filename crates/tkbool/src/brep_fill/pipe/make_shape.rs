// Rust translation of BRepFill_Pipe::MakeShape and RebuildTopOrBottomFace and of the static UpdateTolFromTopOrBottomPCurve
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Pipe.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shape swept by each kind of profile: wires of vertices, shells of edges and
//! wires, solids of faces, and the recursion through shells and compounds.
use super::history::update_map;
use super::{BRepFillPipe, null};
use crate::brep_fill::section_law::{BRepFillSectionLaw, BRepFillShapeLaw};
use crate::brep_fill::shape_table::{ShapeTable, cell, cell_mut, copy_table, new_table};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{
    self, CompSolid, Compound, Edge, Face, Shape, ShapeIterator, Shell, Solid, Wire,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;

/// `UpdateTolFromTopOrBottomPCurve(aFace, anEdge)`: the edge tolerance (and its
/// vertices') raised to 1.00005 times the largest distance at 23 samples between its
/// curve and its p-curve on the face (sampled on the range of the 3d curve).
fn update_tol_from_top_or_bottom_pcurve(a_face: &Face, an_edge: &Edge) -> Result<(), Failure> {
    let found = BRepTool::curve_on_face(an_edge, a_face)?;
    if found.pcurve.is_null() {
        return Ok(());
    }
    let (a_curve, fpar, lpar) = BRepTool::curve(an_edge)?;
    let Some(curve) = a_curve.get() else {
        return Ok(());
    };
    let gahc2d: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(&found.pcurve, fpar, lpar)?).cast();
    let gahs: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(
        &BRepTool::surface(a_face)?,
    )?)
    .cast();
    let con_s = Adaptor3dCurveOnSurface::with_curve(&gahc2d, &gahs)?;
    let init_tol: f64 = BRepTool::edge_tolerance(an_edge)?;
    let mut tol_tol: f64 = init_tol * init_tol;
    const NCONTROL: i32 = 22;
    let delta: f64 = (lpar - fpar) / f64::from(NCONTROL);
    for i in 0..=NCONTROL {
        let par: f64 = fpar + f64::from(i) * delta;
        let sqdist: f64 = curve.value(par)?.square_distance(&con_s.value(par)?);
        if sqdist > tol_tol {
            tol_tol = sqdist;
        }
    }
    let tol: f64 = 1.00005 * tol_tol.sqrt();
    if tol < init_tol {
        return Ok(());
    }
    let builder = BRepBuilder::default();
    builder.update_edge_tolerance(an_edge, tol)?;
    let mut itv = ShapeIterator::new(an_edge, true, true);
    while itv.more() {
        builder.update_vertex_tolerance(&topods::vertex(itv.value()?)?, tol)?;
        itv.next();
    }
    Ok(())
}

/// A wire of one shape, closed as the shape is.
fn wire_of_one(s: &Shape) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut w = Wire::default();
    builder.make_wire(&mut w);
    builder.add(&mut w, s)?;
    w.set_closed(BRepTool::is_closed(s)?)?;
    Ok(w.into_shape())
}

/// The rows of `aux` stacked under those of `old` (over the columns of `old`; a column
/// past those of `aux` is Standard_OutOfRange, as natively).
fn stacked(old: &ShapeTable, aux: &ShapeTable) -> Result<ShapeTable, Failure> {
    let length: i32 = aux.col_length() + old.col_length();
    let mut somme: ShapeTable = new_table((1, length), (1, aux.row_length()))?;
    for jj in 1..=old.row_length() {
        for ii in 1..=old.col_length() {
            *cell_mut(&mut somme, ii, jj)? = cell(old, ii, jj)?.clone();
        }
        for (kk, ii) in (1..=aux.col_length()).zip(old.col_length() + 1..) {
            *cell_mut(&mut somme, ii, jj)? = cell(aux, kk, jj)?.clone();
        }
    }
    Ok(somme)
}

/// The empty result of a profile kind and whether its sub-shapes are swept in turn.
struct ProfileResult {
    result: Shape,
    explode: bool,
}

impl BRepFillPipe {
    /// `MakeShape(S, theOriginalS, FirstShape, LastShape)` (private): the shape swept by
    /// the placed profile `s` between the first and last shapes; Standard_DomainError for
    /// a solid profile.
    pub(super) fn make_shape(
        &mut self,
        s: &Shape,
        the_original_s: &Shape,
        first_shape: &Shape,
        last_shape: &Shape,
    ) -> Result<Shape, Failure> {
        let initial_length: i32 = self.my_faces.as_ref().map_or(0, |f| f.col_length());
        let (mut the_s, mut the_first, mut the_last) =
            (s.clone(), first_shape.clone(), last_shape.clone());
        let mut kind = self.empty_result(s, (&mut the_s, &mut the_first, &mut the_last))?;
        if kind.explode {
            self.sweep_children(
                s,
                the_original_s,
                (&the_s, &the_first, &the_last),
                &mut kind.result,
            )?;
        } else if the_s.shape_type()? == ShapeEnum::Vertex {
            kind.result = self.sweep_vertex(&the_s, the_original_s)?;
        } else if the_s.shape_type()? == ShapeEnum::Wire {
            kind.result = self.sweep_wire(&the_s, the_original_s, (&the_first, &the_last))?;
        }
        if the_s.shape_type()? == ShapeEnum::Face {
            return self.solid_of_face(kind.result, the_original_s, initial_length, &the_last);
        }
        Ok(kind.result)
    }

    /// The empty result of the profile kind (an edge is put in wires, with its first and
    /// last shapes); a face starts its shell with the reversed first face of an open
    /// spine.
    fn empty_result(
        &self,
        s: &Shape,
        (the_s, the_first, the_last): (&mut Shape, &mut Shape, &mut Shape),
    ) -> Result<ProfileResult, Failure> {
        let builder = BRepBuilder::default();
        let mut shell = Shell::default();
        match s.shape_type()? {
            ShapeEnum::Vertex => {
                let mut w = Wire::default();
                builder.make_wire(&mut w);
                Ok(ProfileResult {
                    result: w.into_shape(),
                    explode: false,
                })
            }
            ShapeEnum::Edge => {
                builder.make_shell(&mut shell);
                *the_s = wire_of_one(s)?;
                if !the_first.is_null() {
                    *the_first = wire_of_one(the_first)?;
                }
                if !the_last.is_null() {
                    *the_last = wire_of_one(the_last)?;
                }
                let result = shell.into_shape();
                result.set_closed(BRepTool::is_closed(&result)?)?;
                Ok(ProfileResult {
                    result,
                    explode: false,
                })
            }
            ShapeEnum::Wire => {
                builder.make_shell(&mut shell);
                Ok(ProfileResult {
                    result: shell.into_shape(),
                    explode: false,
                })
            }
            ShapeEnum::Face => {
                builder.make_shell(&mut shell);
                if !self.my_spine.closed()? && !the_first.is_null() {
                    builder.add(&mut shell, &the_first.reversed())?;
                }
                let result = shell.into_shape();
                result.set_closed(BRepTool::is_closed(&result)?)?;
                Ok(ProfileResult {
                    result,
                    explode: true,
                })
            }
            ShapeEnum::Shell => {
                let mut cs = CompSolid::default();
                builder.make_comp_solid(&mut cs);
                Ok(ProfileResult {
                    result: cs.into_shape(),
                    explode: true,
                })
            }
            ShapeEnum::Solid | ShapeEnum::CompSolid => Err(Failure::with_type(
                "Standard_DomainError",
                "BRepFill_Pipe::profile contains solids",
                "BRepFill_Pipe",
            )),
            ShapeEnum::Compound => {
                let mut c = Compound::default();
                builder.make_compound(&mut c);
                Ok(ProfileResult {
                    result: c.into_shape(),
                    explode: true,
                })
            }
            ShapeEnum::Shape => Ok(ProfileResult {
                result: Shape::default(),
                explode: false,
            }),
        }
    }

    /// The sub-shapes swept with their first and last counterparts (added to the result,
    /// except for a face whose wires give the faces through the table).
    fn sweep_children(
        &mut self,
        s: &Shape,
        the_original_s: &Shape,
        (the_s, the_first, the_last): (&Shape, &Shape, &Shape),
        result: &mut Shape,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut it_first = ShapeIterator::new(the_first, true, true);
        let mut it_last = ShapeIterator::new(the_last, true, true);
        let (mut first, mut last) = (Shape::default(), Shape::default());
        let mut it = ShapeIterator::new(s, true, true);
        let mut itorig = ShapeIterator::new(the_original_s, true, true);
        while it.more() {
            if !the_first.is_null() {
                first = it_first.value()?.clone();
            }
            if !the_last.is_null() {
                last = it_last.value()?.clone();
            }
            let swept: Shape = self.make_shape(it.value()?, itorig.value()?, &first, &last)?;
            if the_s.shape_type()? != ShapeEnum::Face {
                builder.add(result, &swept)?;
            }
            if !the_first.is_null() {
                it_first.next();
            }
            if !the_last.is_null() {
                it_last.next();
            }
            it.next();
            itorig.next();
        }
        Ok(())
    }

    /// The wire swept by a vertex; its first and last section vertices become the first
    /// and last shapes.
    fn sweep_vertex(&mut self, the_s: &Shape, the_original_s: &Shape) -> Result<Shape, Failure> {
        let section: Handle<dyn BRepFillSectionLaw> = Handle::new(BRepFillShapeLaw::with_vertex(
            &topods::vertex(the_s)?,
            true,
        )?)
        .cast();
        let mk_sw = self.swept(&section, None)?;
        let result: Shape = mk_sw.shape();
        update_map(the_original_s, &result, &mut self.my_gen_map)?;
        self.my_error_on_surf = mk_sw.error_on_surface();
        let a_sections = mk_sw.sections();
        if let Some(sections) = a_sections.get() {
            let t = sections.array2();
            self.my_first = cell(&t, 1, 1)?.clone();
            self.my_last = cell(&t, 1, t.upper_col())?.clone();
        }
        self.build_history(&mk_sw, the_original_s)?;
        Ok(result)
    }

    /// The shell swept by a wire between the first and last wires; its tables are
    /// stacked under those of the previous wires.
    fn sweep_wire(
        &mut self,
        the_s: &Shape,
        the_original_s: &Shape,
        bounds: (&Shape, &Shape),
    ) -> Result<Shape, Failure> {
        let section: Handle<dyn BRepFillSectionLaw> =
            Handle::new(BRepFillShapeLaw::with_wire(&topods::wire(the_s)?, true)?).cast();
        let mk_sw = self.swept(&section, Some(bounds))?;
        let result: Shape = mk_sw.shape();
        self.my_error_on_surf = mk_sw.error_on_surface();
        let (faces, sections, edges) = (
            copy_table(&mk_sw.sub_shape(), "BRepFill_Pipe")?,
            copy_table(&mk_sw.sections(), "BRepFill_Pipe")?,
            copy_table(&mk_sw.inter_faces(), "BRepFill_Pipe")?,
        );
        match (
            self.my_sections.take(),
            self.my_faces.take(),
            self.my_edges.take(),
        ) {
            (Some(old_sections), Some(old_faces), Some(old_edges)) => {
                self.my_faces = Some(stacked(&old_faces, &faces)?);
                if old_sections.row_length() >= 1 {
                    self.my_cur_index_of_section_edge = old_sections.col_length() + 1;
                }
                self.my_sections = Some(stacked(&old_sections, &sections)?);
                self.my_edges = Some(stacked(&old_edges, &edges)?);
            }
            _ => {
                (self.my_faces, self.my_sections, self.my_edges) =
                    (Some(faces), Some(sections), Some(edges))
            }
        }
        self.build_history(&mk_sw, the_original_s)?;
        Ok(result)
    }

    /// The solid of a face profile: the top faces rebuilt, the swept faces of the face's
    /// wires and (open spine) the rebuilt last face added, reversed when its infinite
    /// point is inside.
    fn solid_of_face(
        &mut self,
        mut result: Shape,
        the_original_s: &Shape,
        initial_length: i32,
        the_last: &Shape,
    ) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        for a_face in Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape)? {
            self.rebuild_top_or_bottom_face(&a_face.reversed(), true)?;
        }
        let faces = self.my_faces.as_ref().ok_or_else(|| null("face table"))?;
        let mut swept_faces: Vec<Shape> = Vec::new();
        for ii in initial_length + 1..=faces.col_length() {
            for jj in 1..=faces.row_length() {
                let f: &Shape = cell(faces, ii, jj)?;
                if f.shape_type()? == ShapeEnum::Face && !f.is_null() {
                    swept_faces.push(f.clone());
                }
            }
        }
        for f in &swept_faces {
            builder.add(&mut result, f)?;
        }
        if !self.my_spine.closed()? {
            self.rebuild_top_or_bottom_face(the_last, false)?;
            let last_face = topods::face(the_last)?;
            builder.add(&mut result, &last_face)?;
        }
        let mut solid = Solid::default();
        builder.make_solid(&mut solid);
        result.set_closed(true)?;
        let shell = topods::shell(&result)?;
        builder.add(&mut solid, &shell)?;
        let mut sc = BRepClass3dSolidClassifier::with_shape(&solid)?;
        sc.perform_infinite_point(precision::confusion())?;
        if sc.state() == State::In {
            builder.make_solid(&mut solid);
            let reversed = topods::shell(&result.reversed())?;
            builder.add(&mut solid, &reversed)?;
        }
        let solid: Shape = solid.into_shape();
        update_map(the_original_s, &solid, &mut self.my_gen_map)?;
        Ok(solid)
    }

    /// `RebuildTopOrBottomFace(aFace, IsTop)` (private): the edges of the face's wires
    /// that are v-iso edges of the first (top) or last section replaced by them, their
    /// tolerances raised to their p-curves on the face.
    fn rebuild_top_or_bottom_face(&self, a_face: &Shape, is_top: bool) -> Result<(), Failure> {
        let sections = self
            .my_sections
            .as_ref()
            .ok_or_else(|| null("section table"))?;
        let index_of_section: i32 = if is_top { 1 } else { sections.row_length() };
        let builder = BRepBuilder::default();
        let face: Face = topods::face(a_face)?;
        let mut itf = ShapeIterator::new(a_face, true, true);
        while itf.more() {
            let mut a_wire: Shape = itf.value()?.clone();
            itf.next();
            let mut pairs: Vec<(Shape, Shape)> = Vec::new();
            let mut itw = ShapeIterator::new(&a_wire, true, true);
            while itw.more() {
                let an_edge: &Shape = itw.value()?;
                for ii in self.my_cur_index_of_section_edge..=sections.col_length() {
                    let a_viso_edge: &Shape = cell(sections, ii, index_of_section)?;
                    if an_edge.is_same(a_viso_edge) {
                        pairs.push((an_edge.clone(), a_viso_edge.clone()));
                        break;
                    }
                }
                itw.next();
            }
            a_wire.set_free(true)?;
            for (init_edge, res_edge) in &pairs {
                builder.remove(&mut a_wire, init_edge)?;
                update_tol_from_top_or_bottom_pcurve(&face, &topods::edge(res_edge)?)?;
                builder.add(&mut a_wire, res_edge)?;
            }
        }
        Ok(())
    }
}
