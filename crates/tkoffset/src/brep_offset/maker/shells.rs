// Rust translation of BRepOffset_MakeOffset::MakeShells, MakeSolid, SelectShells and
// EncodeRegularity of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shells of the split offset faces, the solids, the removal of the open shells and
//! the continuity of the edges of the result.
use super::BRepOffsetMakeOffset;
use super::statics::{self, compound};
use super::steps::update_init_offset;
use crate::brep_offset::{BRepOffsetError, BRepOffsetTool};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::Quilt;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Solid};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::{JoinType, Shape as Continuity, SurfaceType};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_lib;

impl BRepOffsetMakeOffset {
    /// `MakeShells(Range)`: the shells of the last splits of the offset faces (reversed
    /// and with the faces and walls of the shape when thickening); for the complete
    /// intersection of planar solids, the shells of BuildShellsCompleteInter.
    pub(super) fn make_shells(&mut self, the_range: &ProgressRange) -> Result<(), Failure> {
        let a_ps = ProgressScope::with_literal(the_range, "Making shells", 1., false);
        // The splits of the offset faces.
        let mut a_lsf = List::default();
        for r in self.my_image_offset.roots().to_vec() {
            if !a_ps.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
            let mut a_f = r;
            if self.my_thickening {
                // The offset faces change their orientations.
                a_f.reverse();
            }
            for a_f_im in self.my_image_offset.last_image(&a_f) {
                a_lsf.append(a_f_im);
            }
        }
        if self.my_thickening {
            for f in Explorer::new(&self.my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
                a_lsf.append(f);
            }
            for f in self.my_walls.iter() {
                a_lsf.append(f.clone());
            }
        }
        if a_lsf.is_empty() {
            return Ok(());
        }
        let mut b_done = false;
        if self.my_join == JoinType::Intersection
            && self.my_inter
            && !self.my_thickening
            && self.my_faces.is_empty()
            && statics::is_solid(&self.my_shape)?
            && self.my_is_planar
        {
            let mut a_shells = Shape::default();
            b_done = statics::build_shells_complete_inter(
                &a_lsf,
                &mut self.my_image_offset,
                &mut a_shells,
            )?;
            if b_done {
                self.my_offset_shape = a_shells;
            }
        }
        if !b_done {
            let mut glue = Quilt::new();
            for f in a_lsf.iter() {
                glue.add(f)?;
            }
            self.my_offset_shape = glue.shells()?;
        }
        // The right value of the closed flag.
        for a_s in Explorer::new(&self.my_offset_shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
            if !a_s.tshape().closed() && BRepTool::is_closed(&a_s)? {
                a_s.tshape().set_closed(true);
            }
        }
        Ok(())
    }

    /// `MakeSolid(Range)`: the history updated on the result, then the closed shells made
    /// a solid (corrected when invalid with several shells), the others kept in a
    /// compound.
    pub(super) fn make_solid(&mut self, the_range: &ProgressRange) -> Result<(), Failure> {
        if self.my_offset_shape.is_null() {
            return Ok(());
        }
        // The history.
        update_init_offset(
            &mut self.my_init_offset_face,
            &self.my_image_offset,
            &self.my_offset_shape,
            ShapeEnum::Face,
        )?;
        update_init_offset(
            &mut self.my_init_offset_edge,
            &self.my_image_offset,
            &self.my_offset_shape,
            ShapeEnum::Edge,
        )?;
        let builder = BRepBuilder::default();
        let mut nb_shell = 0;
        let mut nc = compound();
        let mut s1 = Shape::default();
        let a_ps = ProgressScope::with_literal(the_range, "Making solid", 1., false);
        let mut sol_s = Solid::default();
        builder.make_solid(&mut sol_s);
        let mut sol = sol_s.into_shape();
        sol.tshape().set_closed(true);
        let a_make_solid = self.my_shape.shape_type()? == ShapeEnum::Solid || self.my_thickening;
        for sh0 in Explorer::new(&self.my_offset_shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
            if !a_ps.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
            let mut sh = topods::shell(&sh0)?.into_shape();
            if self.my_thickening && self.my_offset > 0. {
                sh.reverse();
            }
            nb_shell += 1;
            if sh.tshape().closed() && a_make_solid {
                builder.add(&mut sol, &sh)?;
            } else {
                builder.add(&mut nc, &sh)?;
                if nb_shell == 1 {
                    s1 = sh;
                }
            }
        }
        let nbs = ShapeIterator::new(&sol, true, true).count();
        let mut sol_is_null = nbs == 0;
        // The check of the solid.
        if nbs > 1 {
            let a_check = BRepCheckAnalyzer::new(&sol, false, false, false)?;
            if !a_check.is_valid()? {
                let mut a_sol_list = List::default();
                statics::correct_solid(&mut sol, &mut a_sol_list)?;
                if !a_sol_list.is_empty() {
                    builder.add(&mut nc, &sol)?;
                    for s in a_sol_list.iter() {
                        builder.add(&mut nc, s)?;
                    }
                    sol_is_null = true;
                }
            }
        }
        let nc_is_null = ShapeIterator::new(&nc, true, true).count() == 0;
        if !sol_is_null && !nc_is_null {
            builder.add(&mut nc, &sol)?;
            self.my_offset_shape = nc;
        } else if sol_is_null && !nc_is_null {
            self.my_offset_shape = if nb_shell == 1 { s1 } else { nc };
        } else if !sol_is_null && nc_is_null {
            self.my_offset_shape = sol;
        } else {
            self.my_offset_shape = nc;
        }
        Ok(())
    }

    /// `SelectShells()`: the open shells removed (`Deboucle3D`) unless the shape has
    /// free borders and no caps.
    pub(super) fn select_shells(&mut self) -> Result<(), Failure> {
        let mut free_edges = MapOfShape::default();
        // The edges that may be free borders of the parallel shell: the free borders of
        // the shape.
        for e in Explorer::new(&self.my_face_comp, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if self.my_analyse.ancestors(&e)?.len() < 2 {
                let first = self
                    .my_analyse
                    .type_of(&topods::edge(&e)?)?
                    .first()
                    .ok_or_else(|| {
                        Failure::with_message("Standard_NoSuchObject", "NCollection_List::First")
                    })?;
                if first.get_type()? == TypeOfConcavity::FreeBound {
                    free_edges.add(e);
                }
            }
        }
        // Free borders and no caps: no 3d unwinding.
        if !free_edges.is_empty() && self.my_faces.is_empty() {
            return Ok(());
        }
        self.my_offset_shape = BRepOffsetTool::deboucle_3d(&self.my_offset_shape, &free_edges)?;
        Ok(())
    }

    /// `EncodeRegularity()`: the G1 (or CN) continuity of the edges of the result between
    /// faces whose roots join tangentially (sphere and tube, face and tube, tangent faces,
    /// tangent tubes) and on the seams of analytic offsets.
    pub(super) fn encode_regularity(&mut self) -> Result<(), Failure> {
        if self.my_offset_shape.is_null() {
            return Ok(());
        }
        let builder = BRepBuilder::default();
        let mut ms = MapOfShape::default();
        // The G1 edges of the result.
        for oes in Explorer::new(&self.my_offset_shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let oe = topods::edge(&oes)?;
            let (continuity, max_degree, max_segment) = brep_lib::BUILD_CURVE3D_DEFAULTS;
            brep_lib::build_curve3d(&oe, self.my_tol, continuity, max_degree, max_segment)?;
            if !ms.add(oes.clone()) {
                continue;
            }
            let roe = if self.my_image_offset.is_image(&oes) {
                self.my_image_offset.root(&oes)?
            } else {
                oes.clone()
            };
            let lof_of: Vec<Shape> = self.as_des().ascendant(&roe).collect();
            if lof_of.len() != 2 {
                continue;
            }
            let (f1s, f2s) = (&lof_of[0], &lof_of[1]);
            if f1s.is_null() || f2s.is_null() {
                continue;
            }
            let (f1, f2) = (topods::face(f1s)?, topods::face(f2s)?);
            let root1 = self.my_init_offset_face.root(f1s)?;
            let root2 = self.my_init_offset_face.root(f2s)?;
            let type1 = root1.shape_type()?;
            let type2 = root2.shape_type()?;
            if f1.is_same(&f2) {
                if BRepTool::is_closed_on_face(&oe, &f1)? {
                    // In the intersection mode the edges are not coded in
                    // myInitOffsetEdge: case by case; the hidden parts need CN on the
                    // analytic surfaces.
                    if self.my_join == JoinType::Intersection {
                        let bs = BRepAdaptorSurface::with_face(&f1, false)?;
                        let s_type = bs.get_type()?;
                        if matches!(
                            s_type,
                            SurfaceType::Cylinder
                                | SurfaceType::Cone
                                | SurfaceType::Sphere
                                | SurfaceType::Torus
                        ) {
                            builder.continuity_on_faces(&oe, &f1, &f1, Continuity::CN)?;
                        }
                    } else if self.my_init_offset_edge.is_image(&roe)
                        && type1 == ShapeEnum::Face
                        && type2 == ShapeEnum::Face
                    {
                        let f_root = topods::face(&root1)?;
                        let ei = topods::edge(&self.my_init_offset_edge.image_from(&roe)?)?;
                        let conti = BRepTool::continuity_on_faces(&ei, &f_root, &f_root)?;
                        if conti == Continuity::CN {
                            builder.continuity_on_faces(&oe, &f1, &f1, Continuity::CN)?;
                        } else if conti > Continuity::C0 {
                            builder.continuity_on_faces(&oe, &f1, &f1, Continuity::G1)?;
                        }
                    }
                }
                continue;
            }
            // The G1 regularities between a sphere and a tube (a vertex root on the edge
            // root), a face and a tube (an edge root of the face root), two faces tangent
            // in the initial shape, and two tangent tubes.
            let g1 = match (type1, type2) {
                (ShapeEnum::Edge, ShapeEnum::Vertex) => {
                    let (v1, v2) = topexp::vertices(&topods::edge(&root1)?, false);
                    v1.is_same(&root2) || v2.is_same(&root2)
                }
                (ShapeEnum::Vertex, ShapeEnum::Edge) => {
                    let (v1, v2) = topexp::vertices(&topods::edge(&root2)?, false);
                    v1.is_same(&root1) || v2.is_same(&root1)
                }
                (ShapeEnum::Face, ShapeEnum::Edge) => {
                    Explorer::new(&root1, ShapeEnum::Edge, ShapeEnum::Shape)?
                        .any(|e| e.is_same(&root2))
                }
                (ShapeEnum::Edge, ShapeEnum::Face) => {
                    Explorer::new(&root2, ShapeEnum::Edge, ShapeEnum::Shape)?
                        .any(|e| e.is_same(&root1))
                }
                (ShapeEnum::Face, ShapeEnum::Face) => {
                    let mut le = List::default();
                    BRepOffsetTool::find_common_shapes(&root1, &root2, ShapeEnum::Edge, &mut le)?;
                    let mut tangent = false;
                    if le.len() == 1 {
                        let ed = le.first().cloned().unwrap_or_default();
                        if self.my_analyse.has_ancestor(&ed) {
                            let li = self.my_analyse.type_of(&topods::edge(&ed)?)?;
                            tangent =
                                li.len() == 1 && li[0].get_type()? == TypeOfConcavity::Tangential;
                        }
                    }
                    tangent
                }
                (ShapeEnum::Edge, ShapeEnum::Edge) => {
                    let mut lv = List::default();
                    BRepOffsetTool::find_common_shapes(&root1, &root2, ShapeEnum::Vertex, &mut lv)?;
                    let mut tangent = false;
                    if lv.len() == 1 {
                        let mut led_tg = Vec::new();
                        let v = topods::vertex(lv.first().expect("one vertex"))?;
                        self.my_analyse
                            .tangent_edges(&topods::edge(&root1)?, &v, &mut led_tg)?;
                        tangent = led_tg.iter().any(|e| e.is_same(&root2));
                    }
                    tangent
                }
                _ => false,
            };
            if g1 {
                builder.continuity_on_faces(&oe, &f1, &f2, Continuity::G1)?;
            }
        }
        Ok(())
    }
}
