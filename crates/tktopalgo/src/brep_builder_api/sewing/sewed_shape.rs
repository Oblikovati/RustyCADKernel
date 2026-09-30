// Rust translation of BRepBuilderAPI_Sewing::CreateSewedShape, CreateOutputInformations
// and SameParameterShape (BRepBuilderAPI_Sewing.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sewn shape (the shells quilted from the modified faces), the classification of
//! its edges (free, contiguous, multiple, degenerated), and the final SameParameter.
use super::{BRepBuilderAPISewing, live_reshape};
use crate::brep_lib;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::Quilt;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Shape, ShapeIterator, Shell};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The first face of a shell and whether it has more than one (`numf` up to 2).
fn first_faces(shell: &Shape) -> Result<(Shape, i32), Failure> {
    let (mut face, mut numf) = (Shape::default(), 0);
    let mut exp = Explorer::new(shell, ShapeEnum::Face, ShapeEnum::Shape)?;
    while exp.more() && numf < 2 {
        face = exp.current().clone();
        numf += 1;
        exp.next()?;
    }
    Ok((face, numf))
}

/// Adds a shell to the compound (its face alone when it has one); returns 1 when
/// something was added.
fn add_shell_or_face(compound: &mut Compound, shell: &Shape) -> Result<i32, Failure> {
    let b = BRepBuilder::default();
    let (face, numf) = first_faces(shell)?;
    if numf == 1 {
        b.add(compound, &face)?;
    } else if numf > 1 {
        b.add(compound, shell)?;
    }
    Ok(i32::from(numf != 0))
}

/// Whether a shell shares an edge with the edges collected so far.
fn shares_edge(shell: &Shape, edges: &MapOfShape) -> Result<bool, Failure> {
    for edge in Explorer::new(shell, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if edges.contains(&edge) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The non-manifold merge of the collected shells: each new shell takes the first
/// shell left and every later one sharing an edge with it; returns the count added.
fn merge_old_shells(
    old_shells: &IndexedMapOfShape,
    compound: &mut Compound,
) -> Result<i32, Failure> {
    let b = BRepBuilder::default();
    let nb_old_shells = old_shells.extent() as i32;
    let mut index_merged: Map<i32> = Map::default();
    let mut numsh = 0;
    while (index_merged.extent() as i32) < nb_old_shells {
        let mut new_shell = Shell::default();
        let mut new_edges = MapOfShape::default();
        for i in 1..=nb_old_shells {
            if index_merged.contains(&i) {
                continue;
            }
            let shell = topods::shell(old_shells.find_key(i as usize)?)?;
            let first = new_shell.is_null();
            if first {
                b.make_shell(&mut new_shell);
                let mut it = ShapeIterator::new(&shell, true, true);
                while let Ok(child) = it.value() {
                    b.add(&mut new_shell, child)?;
                    it.next();
                }
            } else if !shares_edge(&shell, &new_edges)? {
                continue;
            }
            for edge in Explorer::new(&shell, ShapeEnum::Edge, ShapeEnum::Shape)? {
                new_edges.add(edge);
            }
            if !first {
                for face in Explorer::new(&shell, ShapeEnum::Face, ShapeEnum::Shape)? {
                    b.add(&mut new_shell, &face)?;
                }
            }
            index_merged.add(i);
        }
        numsh += add_shell_or_face(compound, &new_shell)?;
    }
    Ok(numsh)
}

impl BRepBuilderAPISewing {
    /// The quilt of the images of the context shape (local sewing) or of the inputs,
    /// whose images are stored.
    fn quilted_shells(&mut self) -> Result<Shape, Failure> {
        let reshape = live_reshape(&self.my_re_shape)?;
        let mut quilt = Quilt::new();
        let is_local = !self.my_shape.is_null();
        if is_local {
            quilt.add(&reshape.apply(&self.my_shape, ShapeEnum::Shape)?)?;
        }
        for i in 1..=self.my_old_shapes.extent() {
            let sh = self.my_old_shapes.find_from_index(i)?.clone();
            if sh.is_null() {
                continue;
            }
            let image = reshape.apply(&sh, ShapeEnum::Shape)?;
            *self.my_old_shapes.change_from_index(i)? = image.clone();
            if !is_local {
                quilt.add(&image)?;
            }
        }
        quilt.shells()
    }

    /// `CreateSewedShape()`: the shells of the quilt (a shell of one face is its face);
    /// in non-manifold mode the shells and faces are merged by shared edges. A single
    /// component is the result itself, otherwise the compound.
    pub(crate) fn create_sewed_shape(&mut self) -> Result<(), Failure> {
        let new_shape = self.quilted_shells()?;
        let b = BRepBuilder::default();
        let mut compound = Compound::default();
        b.make_compound(&mut compound);
        let mut old_shells = IndexedMapOfShape::default();
        let mut numsh = 0;
        let mut it = ShapeIterator::new(&new_shape, false, true);
        while let Ok(child) = it.value() {
            let mut sh = child.clone();
            it.next();
            let kind = sh.shape_type()?;
            if self.my_nonmanifold && kind == ShapeEnum::Shell {
                if !old_shells.contains(&sh) {
                    old_shells.add(sh);
                }
            } else if self.my_nonmanifold && kind == ShapeEnum::Face {
                let mut ss = Shell::default();
                b.make_shell(&mut ss);
                b.add(&mut ss, &sh)?;
                sh = ss.into_shape();
                old_shells.add(sh);
            } else if kind == ShapeEnum::Shell {
                numsh += add_shell_or_face(&mut compound, &sh)?;
            } else {
                b.add(&mut compound, &sh)?;
                numsh += 1;
            }
        }
        if self.my_nonmanifold && old_shells.extent() == 1 {
            numsh += add_shell_or_face(&mut compound, old_shells.find_key(1)?)?;
        } else if self.my_nonmanifold && !old_shells.is_empty() {
            numsh += merge_old_shells(&old_shells, &mut compound)?;
        }
        self.my_sewed_shape = if numsh == 1 {
            ShapeIterator::new(&compound, false, true).value()?.clone()
        } else {
            compound.into_shape()
        };
        Ok(())
    }

    /// The sections of each edge of the result: the cutting section whose image it is,
    /// or its boundary.
    fn edge_sections(&self) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
        let reshape = live_reshape(&self.my_re_shape)?;
        let mut edge_sections = IndexedDataMapOfShapeListOfShape::default();
        for i in 1..=self.my_bound_faces.extent() {
            let bound = self.my_bound_faces.find_key(i)?;
            let lsect: Vec<Shape> = match self.my_bound_sections.find(bound) {
                Ok(sections) => sections.iter().cloned().collect(),
                Err(_) => Vec::new(),
            };
            for edge in Explorer::new(
                &reshape.apply(bound, ShapeEnum::Shape)?,
                ShapeEnum::Edge,
                ShapeEnum::Shape,
            )? {
                let mut sec = bound.clone();
                for section in &lsect {
                    if edge.is_same(&reshape.apply(section, ShapeEnum::Shape)?) {
                        sec = section.clone();
                        break;
                    }
                }
                if let Some(list) = edge_sections.change_seek(&edge) {
                    list.append(sec);
                    continue;
                }
                let mut list = List::default();
                list.append(sec);
                edge_sections.add(edge, list);
            }
        }
        Ok(edge_sections)
    }

    /// `CreateOutputInformations()`: the edges of the result with one section are free
    /// (or degenerated), with two contiguous, with more multiple; the merged sections of
    /// cut boundaries that became contiguous edges are bound to their boundary (the
    /// merged edges are forgotten by `EdgeRegularity` when sewing, so none is then, as
    /// natively).
    pub(crate) fn create_output_informations(&mut self) -> Result<(), Failure> {
        let edge_sections = self.edge_sections()?;
        for i in 1..=edge_sections.extent() {
            let edge = edge_sections.find_key(i)?;
            let list_section = edge_sections.find_from_index(i)?;
            match list_section.extent() {
                1 if BRepTool::degenerated(&topods::edge(edge)?)? => {
                    self.my_degenerated.add(edge.clone());
                }
                1 => {
                    self.my_free_edges.add(edge.clone());
                }
                2 => {
                    self.my_contigous_edges
                        .add(edge.clone(), list_section.clone());
                }
                _ => {
                    self.my_multiple_edges.add(edge.clone());
                }
            }
        }
        self.bind_contiguous_sections()
    }

    /// The merged sections of cut boundaries whose images are contiguous edges, bound to
    /// their boundary.
    fn bind_contiguous_sections(&mut self) -> Result<(), Failure> {
        let reshape_handle = self.my_re_shape.clone();
        let reshape = live_reshape(&reshape_handle)?;
        for i in 1..=self.my_bound_faces.extent() {
            let bound = self.my_bound_faces.find_key(i)?.clone();
            let Ok(sections) = self.my_bound_sections.find(&bound) else {
                continue;
            };
            let sections: Vec<Shape> = sections.iter().cloned().collect();
            for section in sections {
                if !self.my_merged_edges.contains(&section) {
                    continue;
                }
                let nedge = reshape.apply(&section, ShapeEnum::Shape)?;
                if nedge.is_null() {
                    continue;
                }
                if !bound.is_same(&section) && self.my_contigous_edges.contains(&nedge) {
                    self.my_contig_sec_bound.bind(section, bound.clone());
                }
            }
        }
        Ok(())
    }

    /// `SameParameterShape()`: with the SameParameter mode, every edge of the result is
    /// made SameParameter at its tolerance; an exception skips the edge.
    pub(crate) fn same_parameter_shape(&self) -> Result<(), Failure> {
        if !self.same_parameter_mode() {
            return Ok(());
        }
        for sec in Explorer::new(&self.my_sewed_shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = topods::edge(&sec)?;
            // The native catch skips the edge.
            let _ = BRepTool::edge_tolerance(&edge)
                .and_then(|tol| brep_lib::same_parameter_edge(&edge, tol));
        }
        Ok(())
    }
}
