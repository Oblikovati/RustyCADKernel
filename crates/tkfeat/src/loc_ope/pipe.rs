// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Pipe.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_Pipe`: the pipe of a profile along a spine (BRepFill_Pipe) for the pipe
//! feature: the faces swept by the connexity edges of the profile are removed, the
//! coplanar connected faces swept by one profile edge are fused, and the pipe lines
//! through points are chained into B-spline curves.
use super::BuildShape;
use crate::errors::{domain_error, no_such_object, null_object, out_of_range};
use rustycad_tkbool::brep_fill::BRepFillPipe;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{Array1, DataMap, List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCurve, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_fill::GeomFillTrihedron;
use rustycad_tkgeombase::geom_convert;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3, Xyz};

/// `LocOpe_Pipe`: the pipe of a profile along a spine.
pub struct Pipe {
    pipe: BRepFillPipe,
    map: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    res: Shape,
    g_shap: List<Shape>,
    crvs: Sequence<Handle<dyn GeomCurve>>,
}

/// The located surface of a face, a rectangular trimmed surface standing for its basis.
fn basis_surface(f: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
    let s = BRepTool::surface(f)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    Ok(trimmed.get().map_or(s.clone(), |t| t.basis_surface()))
}

/// The plane of a face known to be planar.
fn plane_of(f: &Face) -> Result<(Plane3, Handle<dyn GeomSurface>), Failure> {
    let s = basis_surface(f)?;
    let plane = s.cast::<GeomPlane>();
    let pln = plane
        .get()
        .ok_or_else(|| null_object("LocOpe_Pipe", "the fused face is not planar"))?
        .pln();
    Ok((pln, s))
}

/// The static `Orientation(Sub, S)`: the orientation of `sub` in `s`;
/// Standard_NoSuchObject when it is not there.
fn orientation_in(sub: &Shape, s: &Shape) -> Result<Orientation, Failure> {
    Explorer::new(s, sub.shape_type()?, ShapeEnum::Shape)?
        .find(|x| x.is_same(sub))
        .map(|x| x.orientation())
        .ok_or_else(|| no_such_object("LocOpe_Pipe", "the face is not in the pipe"))
}

/// The direction of the plane of a face oriented `or`: the plane axis, reversed as the
/// native test decides from `direct` (the reference plane decides the second half of the
/// test for the faces fused with it).
fn oriented_axis(pl: &Plane3, direct: bool, reference_direct: bool, or: Orientation) -> Direction3 {
    let mut dir = pl.axis().direction();
    if (direct && or == Orientation::Reversed) || (!reference_direct && or == Orientation::Forward)
    {
        dir.reverse();
    }
    dir
}

/// The faces of `fac_fuse` connected to the reference face: their boundary edges
/// (oriented consistently) collected in `map_ed`, the faces removed from both lists;
/// whether one was fused.
fn absorb_connected(
    fac_fuse: &mut Vec<Shape>,
    map_fac: &mut MapOfShape,
    map_ed: &mut MapOfShape,
    dirref: Direction3,
    plref: &Plane3,
    result: &Shape,
) -> Result<bool, Failure> {
    let mut face_to_fuse = false;
    loop {
        let found = fac_fuse.iter().position(|f| {
            Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)
                .is_ok_and(|mut e| e.any(|x| map_ed.contains(&x)))
        });
        let Some(k) = found else {
            return Ok(face_to_fuse);
        };
        face_to_fuse = true;
        let fac = topods::face(&fac_fuse[k])?;
        let orrelat = orientation_in(&fac, result)?;
        let (pl, _) = plane_of(&fac)?;
        // Native quirk kept: the second half of the reversal test reads the reference plane.
        let dirpl = oriented_axis(&pl, pl.direct()?, plref.direct()?, orrelat);
        let orrelat = if dirpl.dot(dirref) > 0. {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        for e in Explorer::new(&fac.oriented(orrelat), ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !map_ed.add(e.clone()) {
                map_ed.remove(&e);
            }
        }
        map_fac.remove(&fac_fuse[k]);
        fac_fuse.remove(k);
    }
}

/// The face on the plane of `face_ref` bounded by the edges of `map_ed`.
fn fused_face(
    face_ref: &Face,
    map_ed: &MapOfShape,
    surf: &Handle<dyn GeomSurface>,
) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut new_face = Face::default();
    b.make_face_on_surface(&mut new_face, surf, BRepTool::face_tolerance(face_ref)?)?;
    let mut new_wire = Wire::default();
    b.make_wire(&mut new_wire);
    for e in map_ed.iter() {
        b.add(&mut new_wire, e)?;
    }
    let first_wire = Explorer::new(
        &face_ref.oriented(Orientation::Forward),
        ShapeEnum::Wire,
        ShapeEnum::Shape,
    )?
    .current()
    .clone();
    new_wire.set_orientation(first_wire.orientation());
    b.add(&mut new_face, &new_wire)?;
    Ok(new_face.into_shape())
}

impl Pipe {
    /// `LocOpe_Pipe(Spine, Profile)`: the pipe (corrected Frenet), without the faces of
    /// the connexity edges of the profile, the coplanar faces of each profile edge fused;
    /// `Pipe::new(&spine, &profile)?`.
    pub fn new(spine: &Wire, profile: &Shape) -> Result<Self, Failure> {
        let pipe = BRepFillPipe::with_profile(
            spine,
            profile,
            GeomFillTrihedron::IsCorrectedFrenet,
            false,
            false,
        )?;
        let mut op = Self {
            pipe,
            map: DataMap::default(),
            res: Shape::default(),
            g_shap: List::default(),
            crvs: Sequence::default(),
        };
        let result = op.pipe.shape().clone();
        // On enleve les faces generees par les edges de connexite du profile,
        // et on fusionne les plans si possible
        let mut the_ef_map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            profile,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut the_ef_map,
        )?;
        let mut goodfaces = List::default();
        for i in 1..=the_ef_map.extent() {
            let edgpr = topods::edge(the_ef_map.find_key(i)?)?;
            op.map.bind(edgpr.as_ref().clone(), List::default());
            if the_ef_map.find_from_index(i)?.len() >= 2 {
                continue; // on ne prend pas les faces generees
            }
            for face in op.profile_edge_faces(spine, &edgpr, &result)? {
                if let Some(l) = op.map.change_seek(&edgpr) {
                    l.append(face.clone());
                }
                goodfaces.append(face);
            }
        }
        for f in Explorer::new(op.pipe.first_shape(), ShapeEnum::Face, ShapeEnum::Shape)? {
            goodfaces.append(f);
        }
        for f in Explorer::new(op.pipe.last_shape(), ShapeEnum::Face, ShapeEnum::Shape)? {
            goodfaces.append(f);
        }
        op.res = BuildShape::with_faces(&goodfaces)?.shape().clone();
        Ok(op)
    }

    /// The faces swept by the profile edge `edgpr`: the non-planar ones, then the planar
    /// ones fused by connected components of the same plane.
    fn profile_edge_faces(
        &self,
        spine: &Wire,
        edgpr: &Edge,
        result: &Shape,
    ) -> Result<Vec<Shape>, Failure> {
        let mut faces = Vec::new();
        let mut map_fac = MapOfShape::default(); // on mappe les plans generes par cet edge
        for e in Explorer::new(spine, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let resfac = self.pipe.face(&topods::edge(&e)?, edgpr)?;
            if resfac.is_null() {
                continue;
            }
            if basis_surface(&resfac)?.cast::<GeomPlane>().is_null() {
                faces.push(resfac.into_shape());
            } else {
                map_fac.add(resfac.into_shape());
            }
        }
        // Chercher les composantes connexes sur cet ensemble de faces., avec meme
        // support geometrique
        if map_fac.extent() <= 1 {
            // un seul plan. Rien a faire
            faces.extend(map_fac.iter().cloned());
            return Ok(faces);
        }
        while map_fac.extent() >= 2 {
            self.fuse_same_plane(&mut map_fac, result, &mut faces)?;
        }
        Ok(faces)
    }

    /// One pass on the planar faces left: those on the plane of the first are fused by
    /// connected components; a single one left is kept as it is.
    fn fuse_same_plane(
        &self,
        map_fac: &mut MapOfShape,
        result: &Shape,
        faces: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        let all: Vec<Shape> = map_fac.iter().cloned().collect();
        let (plref, _) = plane_of(&topods::face(&all[0])?)?;
        let mut fac_fuse = vec![all[0].clone()];
        for f in &all[1..] {
            let (pl, _) = plane_of(&topods::face(f)?)?;
            if pl.axis().is_parallel(plref.axis(), precision::angular())
                && plref.contains(pl.location(), precision::confusion())
            {
                fac_fuse.push(f.clone());
            }
        }
        // FacFuse contient des faces de meme support. Il faut en faire
        // des composantes connexes
        while fac_fuse.len() >= 2 {
            let face_ref = topods::face(&fac_fuse[0])?;
            // Recuperer l'orientation
            let orref = orientation_in(&face_ref, result)?;
            let (plref, surf) = plane_of(&face_ref)?;
            let direct = plref.direct()?;
            let dirref = oriented_axis(&plref, direct, direct, orref);
            let mut map_ed = MapOfShape::default();
            for e in Explorer::new(
                &face_ref.oriented(Orientation::Forward),
                ShapeEnum::Edge,
                ShapeEnum::Shape,
            )? {
                map_ed.add(e);
            }
            map_fac.remove(&fac_fuse[0]);
            fac_fuse.remove(0); // on enleve FaceRef
            if absorb_connected(&mut fac_fuse, map_fac, &mut map_ed, dirref, &plref, result)? {
                faces.push(fused_face(&face_ref, &map_ed, &surf)?);
            }
        }
        if fac_fuse.len() == 1 {
            map_fac.remove(&fac_fuse[0]);
            faces.push(fac_fuse[0].clone());
        }
        Ok(())
    }

    /// `Spine()`; `pipe.spine()`.
    pub fn spine(&self) -> &Shape {
        self.pipe.spine()
    }

    /// `Profile()`; `pipe.profile()`.
    pub fn profile(&self) -> &Shape {
        self.pipe.profile()
    }

    /// `FirstShape()`; `pipe.first_shape()`.
    pub fn first_shape(&self) -> &Shape {
        self.pipe.first_shape()
    }

    /// `LastShape()`; `pipe.last_shape()`.
    pub fn last_shape(&self) -> &Shape {
        self.pipe.last_shape()
    }

    /// `Shape()`; `pipe.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.res
    }

    /// `Shapes(S)`: the edges swept by a vertex of the profile, or the faces kept for an
    /// edge of it; Standard_DomainError for another type, Standard_NoSuchObject for a
    /// shape outside the profile; `pipe.shapes(&s)?`.
    pub fn shapes(&mut self, s: &Shape) -> Result<&List<Shape>, Failure> {
        let typ_s = s.shape_type()?;
        if typ_s != ShapeEnum::Edge && typ_s != ShapeEnum::Vertex {
            return Err(domain_error(
                "LocOpe_Pipe::Shapes",
                "expected an edge or a vertex of the profile",
            ));
        }
        if !Explorer::new(self.pipe.profile(), typ_s, ShapeEnum::Shape)?.any(|x| x.is_same(s)) {
            return Err(no_such_object(
                "LocOpe_Pipe::Shapes",
                "the shape is not in the profile",
            ));
        }
        self.g_shap.clear();
        if typ_s == ShapeEnum::Vertex {
            let v_profile: Vertex = topods::vertex(s)?;
            for e in Explorer::new(self.pipe.spine(), ShapeEnum::Edge, ShapeEnum::Shape)? {
                let resed = self.pipe.edge(&topods::edge(&e)?, &v_profile)?;
                if !resed.is_null() {
                    self.g_shap.append(resed.into_shape());
                }
            }
            return Ok(&self.g_shap);
        }
        self.map
            .seek(s)
            .ok_or_else(|| no_such_object("LocOpe_Pipe::Shapes", "the profile edge has no faces"))
    }

    /// The B-spline pieces of the pipe line through `p1`, reparametrized end to end on
    /// `[k, k + 1]`, and their highest degree.
    fn pipe_line_pieces(
        &mut self,
        p1: Point3,
    ) -> Result<(Vec<Handle<GeomBSplineCurve>>, i32), Failure> {
        let w = self.pipe.pipe_line(p1)?;
        let mut seq: Vec<Handle<GeomBSplineCurve>> = Vec::new();
        let mut max_deg = 0;
        for e in Explorer::new(&w, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let (c1, p1, p2) = BRepTool::curve(&topods::edge(&e)?)?;
            let c = geom_convert::curve_to_bspline_curve(&c1, ParameterisationType::TgtThetaOver2)?;
            let Some(bsp) = c.get() else {
                continue;
            };
            max_deg = max_deg.max(bsp.degree());
            if p1 != bsp.first_parameter() || p2 != bsp.last_parameter() {
                bsp.segment(p1, p2, precision::p_confusion())?;
            }
            let mut tkn = bsp.knots();
            bsplclib::reparametrize(seq.len() as f64, seq.len() as f64 + 1., &mut tkn)?;
            bsp.set_knots(&tkn)?;
            seq.push(c);
        }
        Ok((seq, max_deg))
    }

    /// The pieces raised to `max_deg` and concatenated into one B-spline curve, the
    /// junction knots counted once; Standard_OutOfRange without piece.
    fn concatenated(
        seq: &[Handle<GeomBSplineCurve>],
        max_deg: i32,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        let first = seq
            .first()
            .and_then(|c| c.get())
            .ok_or_else(|| out_of_range("LocOpe_Pipe::Curves", 1, 0))?;
        let (mut nbp, mut nbkn) = (0, 0);
        for c in seq {
            let bsp = c
                .get()
                .ok_or_else(|| null_object("LocOpe_Pipe::Curves", "null piece"))?;
            bsp.increase_degree(max_deg)?;
            nbp += bsp.nb_poles();
            nbkn += bsp.nb_knots();
        }
        let nbcurv = seq.len() as i32;
        nbp -= nbcurv - 1;
        nbkn -= nbcurv - 1;
        let mut tkn =
            Array1::with_bounds(1, nbkn).map_err(|e| e.into_failure("LocOpe_Pipe::Curves"))?;
        let mut tmu =
            Array1::with_bounds(1, nbkn).map_err(|e| e.into_failure("LocOpe_Pipe::Curves"))?;
        let mut tpol =
            Array1::with_bounds(1, nbp).map_err(|e| e.into_failure("LocOpe_Pipe::Curves"))?;
        let (mut ik, mut ip) = (0, 0);
        for (j, c) in seq.iter().enumerate() {
            let bsp = if j == 0 {
                first
            } else {
                c.get()
                    .ok_or_else(|| null_object("LocOpe_Pipe::Curves", "null piece"))?
            };
            let start = if j == 0 { 1 } else { 2 };
            for k in start..=bsp.nb_poles() {
                ip += 1;
                tpol[ip] = bsp.pole(k)?;
            }
            for k in start..=bsp.nb_knots() {
                ik += 1;
                tkn[ik] = bsp.knot(k)?;
                tmu[ik] = bsp.multiplicity(k)?;
            }
            tmu[ik] -= 1;
        }
        tmu[ik] += 1;
        Ok(Handle::new(GeomBSplineCurve::new(&tpol, &tkn, &tmu, max_deg, false)?).cast())
    }

    /// `Curves(Spt)`: for each point, the pipe line through it as one B-spline curve
    /// (null without piece); `pipe.curves(&points)?`.
    pub fn curves(
        &mut self,
        spt: &Sequence<Point3>,
    ) -> Result<&Sequence<Handle<dyn GeomCurve>>, Failure> {
        self.crvs.clear(None);
        for p in spt.iter() {
            let (seq, max_deg) = self.pipe_line_pieces(*p)?;
            if seq.is_empty() {
                self.crvs.append(Handle::default());
                continue;
            }
            let c = Self::concatenated(&seq, max_deg)?;
            self.crvs.append(c);
        }
        Ok(&self.crvs)
    }

    /// `BarycCurve()`: the pipe line through the barycentre of the sample points of the
    /// first shape, as one B-spline curve; without piece a null curve is appended to the
    /// curves and Standard_OutOfRange raised, as natively; `pipe.baryc_curve()?`.
    pub fn baryc_curve(&mut self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let mut spt = Sequence::default();
        super::sample_edges(self.pipe.first_shape(), &mut spt)?;
        let mut bar = Xyz::new(0., 0., 0.);
        for p in spt.iter() {
            bar += p.xyz();
        }
        bar /= spt.len() as f64;
        let (seq, max_deg) = self.pipe_line_pieces(Point3::new(bar.x(), bar.y(), bar.z()))?;
        if seq.is_empty() {
            self.crvs.append(Handle::default());
        }
        Self::concatenated(&seq, max_deg)
    }
}
