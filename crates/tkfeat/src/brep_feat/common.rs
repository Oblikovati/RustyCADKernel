//! Helpers shared by the form features: the barycentre of the sample points of a
//! shape, the Boolean operation of the basis with the tool closing a feature, the
//! check of a sliding face, and the planar fusion test of the faces.
use super::Form;
use crate::errors::construction_error;
use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICut, BRepAlgoAPIFuse};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Plane3, Point3, Xyz};

/// The mean of the sample points of `base` (`LocOpe::SampleEdges`), divided by their
/// count as natively (NaN without point).
pub(crate) fn sample_barycentre(base: &Shape) -> Result<Point3, Failure> {
    let mut spt = Sequence::default();
    crate::loc_ope::sample_edges(base, &mut spt)?;
    let mut bar = Xyz::new(0., 0., 0.);
    for p in spt.iter() {
        bar += p.xyz();
    }
    bar /= spt.len() as f64;
    Ok(Point3::new(bar.x(), bar.y(), bar.z()))
}

/// The plane of the (unlocated, basis) surface of a face moved by its location when
/// not identity, None for another surface.
fn face_plane(f: &Face) -> Result<Option<Plane3>, Failure> {
    let (mut s, loc) = BRepTool::surface_with_location(f)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    if let Some(t) = trimmed.get() {
        s = t.basis_surface();
    }
    let plane = s.cast::<GeomPlane>();
    let Some(plane) = plane.get() else {
        return Ok(None);
    };
    let mut pln = plane.pln();
    // apply locations
    if !loc.is_identity() {
        pln = pln.transformed(loc.transformation());
    }
    Ok(Some(pln))
}

/// The static `ToFuse(F1, F2)` of the features: both faces lie on coplanar planes.
pub(crate) fn to_fuse(f1: &Face, f2: &Face) -> Result<bool, Failure> {
    if f1.is_null() || f2.is_null() {
        return Ok(false);
    }
    let (Some(pl1), Some(pl2)) = (face_plane(f1)?, face_plane(f2)?) else {
        return Ok(false);
    };
    Ok(pl1
        .position()
        .is_coplanar(pl2.position(), precision::confusion(), precision::angular()))
}

/// The face `f` of the basis (Standard_ConstructionError when it is not one).
pub(crate) fn check_basis_face(sbase: &Shape, f: &Face) -> Result<(), Failure> {
    if !Explorer::new(sbase, ShapeEnum::Face, ShapeEnum::Shape)?.any(|x| x.is_same(f)) {
        return Err(construction_error(
            "BRepFeat::Add",
            "the sliding face is not a face of the basis",
        ));
    }
    Ok(())
}

impl Form {
    /// The end of a feature built without the topological reconstruction: the basis fused
    /// with (or cut by) `tool`, or the tool alone for a just feature; the history follows
    /// the Boolean operation.
    pub(crate) fn finish_by_boolean(&mut self, tool: &Shape) -> Result<(), Failure> {
        if self.fuse && !self.just_feat {
            let f = BRepAlgoAPIFuse::from_shapes(&self.sbase, tool)?;
            self.base.my_shape = f.shape().clone();
            let shape = self.base.my_shape.clone();
            self.update_descendants_bop(&f, &shape, false)?;
        } else if !self.fuse {
            let c = BRepAlgoAPICut::from_shapes(&self.sbase, tool)?;
            self.base.my_shape = c.shape().clone();
            let shape = self.base.my_shape.clone();
            self.update_descendants_bop(&c, &shape, false)?;
        } else {
            self.base.my_shape = tool.clone();
        }
        self.base.set_done();
        Ok(())
    }

    /// The basis cut by `tool` when the cut is done (the features made through all or up
    /// to the end without fusion).
    pub(crate) fn cut_when_done(&mut self, tool: &Shape) -> Result<(), Failure> {
        let c = BRepAlgoAPICut::from_shapes(&self.sbase, tool)?;
        let algo: &rustycad_tkbo::brep_algo_api::BRepAlgoAPIAlgo = &c;
        if rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape::is_done(algo) {
            self.base.my_shape = c.shape().clone();
            let shape = self.base.my_shape.clone();
            self.update_descendants_bop(&c, &shape, false)?;
            self.base.set_done();
        }
        Ok(())
    }

    /// The history of the basis faces reset to themselves (the end of `Init`).
    pub(crate) fn reset_history(&mut self) -> Result<(), Failure> {
        self.base.my_shape.nullify();
        self.new_edges.clear();
        self.tgt_edges.clear();
        self.map.clear(false);
        self.f_shape.nullify();
        self.l_shape.nullify();
        for f in Explorer::new(&self.sbase, ShapeEnum::Face, ShapeEnum::Shape)? {
            let mut thelist = rustycad_tkernel::ncollection::List::default();
            thelist.append(f.clone());
            self.map.bind(f, thelist);
        }
        Ok(())
    }

    /// Stops the feature with the error `e` (`NotDone()` then `myStatusError = e`).
    pub(crate) fn fail(&mut self, e: super::StatusError) {
        self.base.set_not_done();
        self.status_error = e;
    }

    /// The fuse and just-feature flags of a feature mode: 0 cut, 1 fuse, 2 just the
    /// feature (other modes keep the flags).
    pub(crate) fn set_mode(&mut self, mode: i32) {
        match mode {
            0 => (self.fuse, self.just_feat) = (false, false),
            1 => (self.fuse, self.just_feat) = (true, false),
            2 => (self.fuse, self.just_feat) = (true, true),
            _ => {}
        }
    }
}
