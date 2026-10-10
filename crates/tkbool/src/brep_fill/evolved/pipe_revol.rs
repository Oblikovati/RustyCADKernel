// Rust translation of BRepFill_Evolved::MakePipe, MakeRevol, FindLocation, TransformInitWork
// and the static PutProfilAt of ModelingAlgorithms/TKBool/BRepFill/BRepFill_Evolved.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The tubes (the profile swept along a spine edge) and the revolutions (the profile
//! turned around a vertical axis at a spine vertex) of an elementary volevo, and the
//! placement of the spine in the plane xOy.
use super::statics::{confusion, edge_vertices, side};
use super::{BRepFillEvolved, ProfileGenerated};
use crate::brep_fill::BRepFillPipe;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{
    BRepToolsModification, Modifier, TrsfModification, WireExplorer,
};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomElementarySurface, GeomPlane};
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkgeomalgo::geom_fill::GeomFillTrihedron;
use rustycad_tkmath::gp::{self, Axis1, Axis3, Direction3, Point3, Transform3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkprim::brep_sweep::BRepSweepRevol;
use rustycad_tktopalgo::brep_lib::{BRepLibFindSurface, BRepLibMakeWire};
use std::f64::consts::PI;

/// `PutProfilAt(ProfRef, AxeRef, E, F, AtStart)` (static): the profile moved from
/// `axe_ref` to the frame at the start (or end) of the p-curve of `e` on `f`, X along
/// its tangent in the edge orientation; Standard_ConstructionError without p-curve.
fn put_profil_at(
    prof_ref: &Wire,
    axe_ref: &Axis3,
    e: &Edge,
    f: &Face,
    at_start: bool,
) -> Result<Wire, Failure> {
    let pc = BRepTool::curve_on_face(e, f)?;
    let c2d = pc.pcurve.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_ConstructionError",
            "ConstructionError in PutProfilAt",
            "BRepFill_Evolved",
        )
    })?;
    let reversed = e.as_ref().orientation() == Orientation::Reversed;
    // The start of a REVERSED edge is the end of its p-curve.
    let at_first = at_start != reversed;
    let (p, d1) = c2d.d1(if at_first { pc.first } else { pc.last })?;
    let d1 = if reversed { d1.reversed() } else { d1 };
    let ax = Axis3::new(
        Point3::new(p.x(), p.y(), 0.),
        gp::dz(),
        Direction3::from_vector(rustycad_tkmath::gp::Vector3::new(d1.x(), d1.y(), 0.))?,
    )?;
    let mut trans = Transform3::default();
    trans.set_transformation_between(ax, *axe_ref);
    topods::wire(&prof_ref.as_ref().moved(&Location::new(trans), false)?)
}

/// The profile turned by pi around Oz for a profile on the right of Oz.
fn side_location(profile: &Wire) -> Result<Location, Failure> {
    let mut trsf = Transform3::default();
    if side(profile, confusion())? > 3 {
        trsf.set_rotation(gp::oz(), PI)?;
    }
    Ok(Location::new(trsf))
}

impl BRepFillEvolved {
    /// `MakePipe(SpineEdge, ProfRef)` (private): the tube of the profile (a copy put at
    /// the start of the edge) along the spine edge, recorded in the map of the edge.
    pub(super) fn make_pipe(&mut self, se: &Edge, axe_ref: &Axis3) -> Result<(), Failure> {
        let moved = topods::wire(
            &self
                .my_profile
                .as_ref()
                .moved(&side_location(&self.my_profile)?, false)?,
        )?;
        let dummy_prof = put_profil_at(&moved, axe_ref, se, &self.my_spine, true)?;
        // Copy of the profile to avoid the accumulation of locations on the edges of myProfile.
        let trsf_mod: Handle<dyn BRepToolsModification> =
            Handle::new(TrsfModification::new(Transform3::default())).cast();
        let modif = Modifier::with_modification(dummy_prof.as_ref(), &trsf_mod)?;
        let gen_prof = topods::wire(&modif.modified_shape(dummy_prof.as_ref())?)?;
        let spine = BRepLibMakeWire::from_edge(se)?.wire()?;
        let pipe = BRepFillPipe::with_profile(
            &spine,
            gen_prof.as_ref(),
            GeomFillTrihedron::IsCorrectedFrenet,
            false,
            false,
        )?;
        self.my_map
            .bind(se.as_ref().clone(), ProfileGenerated::default());
        let mut prof_exp = WireExplorer::with_wire(&self.my_profile)?;
        let mut gen_exp = WireExplorer::with_wire(&gen_prof)?;
        let mut first_vertex = true;
        while prof_exp.more() {
            let (vf, vl) = edge_vertices(prof_exp.current());
            let (vfg, vlg) = edge_vertices(gen_exp.current());
            let m = self
                .my_map
                .change_seek(se.as_ref())
                .ok_or_else(|| super::no_such("the tube of a spine edge"))?;
            if first_vertex {
                m.bind(
                    vf.as_ref().clone(),
                    single(pipe.edge(se, &vfg)?.into_shape()),
                );
                first_vertex = false;
            }
            m.bind(
                vl.as_ref().clone(),
                single(pipe.edge(se, &vlg)?.into_shape()),
            );
            m.bind(
                prof_exp.current().as_ref().clone(),
                single(pipe.face(se, gen_exp.current())?.into_shape()),
            );
            prof_exp.next()?;
            gen_exp.next()?;
        }
        Ok(())
    }

    /// `MakeRevol(SpineEdge, SpineVertex, ProfRef)` (private): the revolution of the
    /// profile (put at the end of the edge, turned by 1.5 pi so that the seam avoids
    /// the bisectors) around the vertical axis at the vertex, recorded in its map.
    pub(super) fn make_revol(
        &mut self,
        se: &Edge,
        v_last: &Vertex,
        axe_ref: &Axis3,
    ) -> Result<(), Failure> {
        let moved = topods::wire(
            &self
                .my_profile
                .as_ref()
                .moved(&side_location(&self.my_profile)?, false)?,
        )?;
        let mut gen_prof = put_profil_at(&moved, axe_ref, se, &self.my_spine, false)?.into_shape();
        let axe_rev = Axis1::new(BRepTool::pnt(v_last)?, gp::dz().reversed());
        let mut dummy = Transform3::default();
        dummy.set_rotation(axe_rev, 1.5 * PI)?;
        gen_prof.move_by(&Location::new(dummy), false)?;
        let gen_prof = topods::wire(&gen_prof)?;
        let mut rev = BRepSweepRevol::full(gen_prof.as_ref(), axe_rev, true)?;
        self.my_map
            .bind(v_last.as_ref().clone(), ProfileGenerated::default());
        let mut prof_exp = WireExplorer::with_wire(&self.my_profile)?;
        let mut gen_exp = WireExplorer::with_wire(&gen_prof)?;
        let mut first_vertex = true;
        while prof_exp.more() {
            let (vf, vl) = edge_vertices(prof_exp.current());
            let (vfg, vlg) = edge_vertices(gen_exp.current());
            let or = gen_exp.current().as_ref().orientation();
            let mut generated: Vec<(Shape, Shape)> = Vec::new();
            if first_vertex {
                let rv = rev.shape_of(vfg.as_ref())?;
                generated.push((vf.as_ref().clone(), rv.oriented(rv.orientation())));
                first_vertex = false;
            }
            let rf = rev.shape_of(gen_exp.current().as_ref())?;
            generated.push((
                prof_exp.current().as_ref().clone(),
                rf.oriented(top_abs::compose(rf.orientation(), or)),
            ));
            let rv = rev.shape_of(vlg.as_ref())?;
            generated.push((vl.as_ref().clone(), rv.oriented(rv.orientation())));
            let m = self
                .my_map
                .change_seek(v_last.as_ref())
                .ok_or_else(|| super::no_such("the revolution of a spine vertex"))?;
            for (key, s) in generated {
                m.bind(key, single(s));
            }
            prof_exp.next()?;
            gen_exp.next()?;
        }
        Ok(())
    }

    /// `FindLocation(Face)` (private): the location taking the plane of the face (found
    /// when not a plane; Standard_NoSuchObject when not planar) onto xOy.
    pub(super) fn find_location(&self, face: &Face) -> Result<Location, Failure> {
        let (mut s, mut l) = BRepTool::surface_with_location(face)?;
        if !s.is_kind_name("Geom_Plane") {
            let fs = BRepLibFindSurface::new(face.as_ref(), -1., true, false)?;
            if !fs.found() {
                return Err(Failure::with_type(
                    "Standard_NoSuchObject",
                    "BRepFill_Evolved : The Face is not planar",
                    "BRepFill_Evolved",
                ));
            }
            (s, l) = (fs.surface(), fs.location());
        }
        if !l.is_identity() {
            s = s
                .get()
                .ok_or_else(|| Failure::new("FindLocation: null surface"))?
                .transformed(l.transformation())?
                .cast();
        }
        let p: Handle<GeomPlane> = s.cast();
        let axis = p
            .get()
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_NullObject",
                    "BRepFill_Evolved FindLocation: the surface is not a plane",
                    "BRepFill_Evolved",
                )
            })?
            .position();
        let axe_ref = Axis3::new(Point3::new(0., 0., 0.), gp::dz(), gp::dx())?;
        let mut t = Transform3::default();
        t.set_transformation_between(axe_ref, axis);
        Ok(Location::new(t))
    }

    /// `TransformInitWork(LS, LP)` (private): the spine moved by `ls`, the profile by `lp`.
    pub(super) fn transform_init_work(
        &mut self,
        ls: &Location,
        lp: &Location,
    ) -> Result<(), Failure> {
        let mut spine = self.my_spine.as_ref().clone();
        spine.move_by(ls, false)?;
        self.my_spine = topods::face(&spine)?;
        let mut profile = self.my_profile.as_ref().clone();
        profile.move_by(lp, false)?;
        self.my_profile = topods::wire(&profile)?;
        Ok(())
    }
}

/// A list of one shape.
fn single(s: Shape) -> List<Shape> {
    let mut l = List::default();
    l.append(s);
    l
}
