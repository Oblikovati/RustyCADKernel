// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_DPrism.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_DPrism`: the drafted prism of a planar face, the evolved solid of the face
//! along a three-edge profile (a drafted side edge between two long caps), its first and
//! last shapes being the faces the caps generate.
mod curves;

use super::BuildShape;
use crate::errors::not_done;
use rustycad_tkbool::brep_fill::BRepFillEvolved;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{self, Axis3, Point3, Vector3, Xyz};
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeEdge, BRepLibMakeVertex, BRepLibMakeWire};

/// The vector of the coordinates `x`.
fn vector_of(x: Xyz) -> Vector3 {
    Vector3::new(x.x(), x.y(), x.z())
}

/// `NECHANT`: the number of sample intervals per spine edge (voir BRepFeat.cxx).
const NECHANT: i32 = 7;

/// `LocOpe_DPrism`: the drafted prism of a face. The native `myCurvs` member is never
/// used and has no counterpart.
#[derive(Clone)]
pub struct DPrism {
    dprism: BRepFillEvolved,
    res: Shape,
    spine: Face,
    profile: Wire,
    profile1: Edge,
    profile2: Edge,
    profile3: Edge,
    height: f64,
    first_shape: Shape,
    last_shape: Shape,
    map: DataMap<Shape, List<Shape>, ShapeMapHasher>,
}

fn vertex_at(x: f64, y: f64, z: f64) -> Result<Shape, Failure> {
    Ok(BRepLibMakeVertex::new(Point3::new(x, y, z))?
        .vertex()?
        .into_shape())
}

fn edge_between(a: &Shape, b: &Shape) -> Result<Edge, Failure> {
    BRepLibMakeEdge::from_vertices(&topods::vertex(a)?, &topods::vertex(b)?)?.edge()
}

/// The length of a cap: twice the larger UV extent of the spine plus `|y|`.
fn cap_length(spine: &Face, y: f64) -> Result<f64, Failure> {
    let (umin, umax, vmin, vmax) = brep_tools::uv_bounds(spine)?;
    Ok(2. * ((umax - umin).max(vmax - vmin) + y.abs()))
}

/// The faces of the compound `c` with a free edge, rebuilt on their wire through that
/// edge only (appended to `lfaces` and `lcomplete`).
fn free_edge_faces(
    c: &Shape,
    lfaces: &mut List<Shape>,
    lcomplete: &mut List<Shape>,
) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    let mut the_map_ef = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(c, ShapeEnum::Edge, ShapeEnum::Face, &mut the_map_ef)?;
    let mut view = MapOfShape::default();
    for i in 1..=the_map_ef.extent() {
        let faces = the_map_ef.find_from_index(i)?;
        if faces.len() != 1 {
            continue;
        }
        let edg = the_map_ef.find_key(i)?;
        let fac = faces.first().cloned().unwrap_or_default();
        if !view.add(fac.clone()) {
            continue;
        }
        let mut new_face = fac.empty_copied()?;
        for w in Explorer::new(
            &fac.oriented(Orientation::Forward),
            ShapeEnum::Wire,
            ShapeEnum::Shape,
        )? {
            let mut edges = Explorer::new(&w, ShapeEnum::Edge, ShapeEnum::Shape)?;
            if edges.any(|e| e.is_same(edg)) {
                b.add(&mut new_face, &w)?;
                lfaces.append(new_face.clone());
                lcomplete.append(new_face.clone());
                break;
            }
        }
    }
    Ok(())
}

/// The plane basis of the (located) surface of a face, a rectangular trimmed surface
/// standing for its basis.
fn basis_surface(f: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
    let s = BRepTool::surface(f)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    Ok(trimmed.get().map_or(s.clone(), |t| t.basis_surface()))
}

impl DPrism {
    /// The evolved solid of `spine` along the profile, not performed yet.
    fn with_profile(spine: &Face, height: f64, edges: (Edge, Edge, Edge)) -> Result<Self, Failure> {
        let (profile1, profile2, profile3) = edges;
        let profile = BRepLibMakeWire::from_three_edges(&profile1, &profile2, &profile3)?.wire()?;
        let mut dprism = BRepFillEvolved::new();
        dprism.perform_face(
            spine,
            &profile,
            &Axis3::from_axis2(gp::xoy()),
            JoinType::Arc,
            false,
        )?;
        Ok(Self {
            dprism,
            res: Shape::default(),
            spine: spine.clone(),
            profile,
            profile1,
            profile2,
            profile3,
            height,
            first_shape: Shape::default(),
            last_shape: Shape::default(),
            map: DataMap::default(),
        })
    }

    /// `LocOpe_DPrism(Spine, Height1, Height2, Angle)`: the drafted prism from `-height2`
    /// to `height1` along the normal, the draft by `angle`;
    /// `DPrism::with_heights(&face, h1, h2, angle)?`.
    pub fn with_heights(
        spine: &Face,
        height1: f64,
        height2: f64,
        angle: f64,
    ) -> Result<Self, Failure> {
        let (y, z) = (height1 * angle.sin(), height1 * angle.cos());
        let vert2 = vertex_at(0., y, z)?;
        let (y1, z1) = (-height2 * angle.sin(), -height2 * angle.cos());
        let vert1 = vertex_at(0., y1, z1)?;
        let profile2 = edge_between(&vert1, &vert2)?;
        let vert3 = vertex_at(0., y + cap_length(spine, y)?, z)?;
        let profile3 = edge_between(&vert2, &vert3)?;
        let vert4 = vertex_at(0., y1 + cap_length(spine, y1)?, z1)?;
        let profile1 = edge_between(&vert4, &vert1)?;
        let mut op = Self::with_profile(spine, height1 + height2, (profile1, profile2, profile3))?;
        if op.dprism.is_done() {
            op.build(true)?;
        }
        Ok(op)
    }

    /// `LocOpe_DPrism(Spine, Height, Angle)`: the drafted prism of height `height` along
    /// the normal, the draft by `angle`; `DPrism::with_height(&face, h, angle)?`.
    pub fn with_height(spine: &Face, height: f64, angle: f64) -> Result<Self, Failure> {
        let (y, z) = (height * angle.sin(), height * angle.cos());
        let vert1 = vertex_at(0., 0., 0.)?;
        let vert2 = vertex_at(0., y, z)?;
        let profile2 = edge_between(&vert1, &vert2)?;
        let deltay = cap_length(spine, y)?;
        let vert3 = vertex_at(0., y + deltay, z)?;
        let profile3 = edge_between(&vert2, &vert3)?;
        let vert4 = vertex_at(0., deltay, 0.)?;
        let profile1 = edge_between(&vert4, &vert1)?;
        let mut op = Self::with_profile(spine, height, (profile1, profile2, profile3))?;
        if op.dprism.is_done() {
            op.build(false)?;
        }
        Ok(op)
    }

    /// The faces generated by the spine edges from the profile edge `prof`, once each,
    /// in a compound.
    fn generated_compound(&self, prof: &Edge) -> Result<Shape, Failure> {
        let b = BRepBuilder::default();
        let mut c = Compound::default();
        b.make_compound(&mut c);
        let mut c = c.into_shape();
        let mut view = MapOfShape::default();
        for es in Explorer::new(&self.spine, ShapeEnum::Edge, ShapeEnum::Shape)? {
            for s in self.dprism.generated_shapes(&es, prof).iter() {
                if view.add(s.clone()) {
                    b.add(&mut c, s)?;
                }
            }
        }
        Ok(c)
    }

    /// The faces generated by the vertices of `es` from the drafted edge.
    fn vertex_faces(
        &self,
        es: &Shape,
        view: &mut MapOfShape,
        lcomplete: &mut List<Shape>,
    ) -> Result<(), Failure> {
        for v in Explorer::new(es, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            for s in self.dprism.generated_shapes(&v, &self.profile2).iter() {
                if view.add(s.clone()) && s.shape_type()? == ShapeEnum::Face {
                    lcomplete.append(s.clone());
                }
            }
        }
        Ok(())
    }

    /// The planar faces generated by `es` merged into one face without the edge
    /// `removed` between them; None when one is not planar.
    fn merged_planar_face(
        &self,
        lffs: &List<Shape>,
        removed: &Shape,
    ) -> Result<Option<Face>, Failure> {
        let b = BRepBuilder::default();
        let mut new_face: Option<Face> = None;
        let mut new_wire = Wire::default();
        b.make_wire(&mut new_wire);
        let mut orref = Orientation::Forward;
        for s in lffs.iter() {
            if s.shape_type()? != ShapeEnum::Face {
                continue;
            }
            let the_wire = Explorer::new(
                &s.oriented(Orientation::Forward),
                ShapeEnum::Wire,
                ShapeEnum::Shape,
            )?
            .current()
            .clone();
            if new_face.is_none() {
                let surf = basis_surface(&topods::face(s)?)?;
                if surf.cast::<GeomPlane>().is_null() {
                    break;
                }
                let mut face = Face::default();
                BRepBuilder::default().make_face_on_surface(
                    &mut face,
                    &surf,
                    BRepTool::face_tolerance(&topods::face(s)?)?,
                )?;
                face.set_orientation(Orientation::Forward);
                new_face = Some(face);
                orref = the_wire.orientation();
                for e in Explorer::new(
                    &the_wire.oriented(Orientation::Forward),
                    ShapeEnum::Edge,
                    ShapeEnum::Shape,
                )? {
                    if !e.is_same(removed) {
                        b.add(&mut new_wire, &e)?;
                    }
                }
            } else {
                for e in Explorer::new(
                    &the_wire.oriented(Orientation::Forward),
                    ShapeEnum::Edge,
                    ShapeEnum::Shape,
                )? {
                    if e.is_same(removed) {
                        continue;
                    }
                    // Les 2 faces planes ont des normales opposees
                    let edge = if the_wire.orientation() != orref {
                        e
                    } else {
                        e.reversed()
                    };
                    b.add(&mut new_wire, &edge)?;
                }
            }
        }
        let Some(mut face) = new_face else {
            return Ok(None);
        };
        b.add(&mut face, &new_wire.oriented(orref))?;
        Ok(Some(face))
    }

    /// The side faces generated by the spine edge `es` (two heights): the planar ones
    /// split by an edge merged into one face mapped to `es`.
    fn side_faces_merged(
        &mut self,
        es: &Shape,
        view: &mut MapOfShape,
        lcomplete: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let lffs = self.dprism.generated_shapes(es, &self.profile2).clone();
        let mut merged = None;
        if let Some(removed) = lffs
            .iter()
            .find(|s| s.shape_type().ok() == Some(ShapeEnum::Edge))
        {
            merged = self.merged_planar_face(&lffs, removed)?;
        }
        match merged {
            Some(face) => {
                lcomplete.append(face.as_ref().clone());
                let mut thelist = List::default();
                thelist.append(face.into_shape());
                self.map.bind(es.clone(), thelist);
            }
            None => {
                for s in lffs.iter() {
                    if view.add(s.clone()) && s.shape_type()? == ShapeEnum::Face {
                        lcomplete.append(s.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// The side faces generated by the spine edge `es` (one height), all kept.
    fn side_faces(&self, es: &Shape, view: &mut MapOfShape, lcomplete: &mut List<Shape>) {
        for s in self.dprism.generated_shapes(es, &self.profile2).iter() {
            if view.add(s.clone()) {
                lcomplete.append(s.clone());
            }
        }
    }

    /// The first and last shapes (the faces generated from the caps) and the drafted
    /// prism of all the faces.
    fn build(&mut self, merge_planar: bool) -> Result<(), Failure> {
        let mut lcomplete = List::default();
        let mut lfaces = List::default();
        free_edge_faces(
            &self.generated_compound(&self.profile1)?,
            &mut lfaces,
            &mut lcomplete,
        )?;
        self.first_shape = BuildShape::with_faces(&lfaces)?.shape().clone();
        lfaces.clear();
        free_edge_faces(
            &self.generated_compound(&self.profile3)?,
            &mut lfaces,
            &mut lcomplete,
        )?;
        self.last_shape = BuildShape::with_faces(&lfaces)?.shape().clone();
        let mut view = MapOfShape::default();
        let spine = self.spine.clone();
        for es in Explorer::new(&spine, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if merge_planar {
                self.side_faces_merged(&es, &mut view, &mut lcomplete)?;
            } else {
                self.side_faces(&es, &mut view, &mut lcomplete);
            }
            self.vertex_faces(&es, &mut view, &mut lcomplete)?;
        }
        self.res = BuildShape::with_faces(&lcomplete)?.shape().clone();
        brep_lib::update_tolerances(&self.res, false)
    }

    /// `IsDone()`: the evolved solid was built; `dprism.is_done()`.
    pub fn is_done(&self) -> bool {
        self.dprism.is_done()
    }

    /// `Shape()`: StdFail_NotDone when the evolved solid failed; `dprism.shape()?`.
    pub fn shape(&self) -> Result<&Shape, Failure> {
        if !self.dprism.is_done() {
            return Err(not_done("LocOpe_DPrism::Shape"));
        }
        Ok(&self.res)
    }

    /// `Spine()`; `dprism.spine()`.
    pub fn spine(&self) -> &Shape {
        &self.spine
    }

    /// `Profile()`; `dprism.profile()`.
    pub fn profile(&self) -> &Shape {
        &self.profile
    }

    /// `FirstShape()`; `dprism.first_shape()`.
    pub fn first_shape(&self) -> &Shape {
        &self.first_shape
    }

    /// `LastShape()`; `dprism.last_shape()`.
    pub fn last_shape(&self) -> &Shape {
        &self.last_shape
    }

    /// `Shapes(S)`: the merged face of the spine edge `s`, else the shapes it generates
    /// from the drafted edge; `dprism.shapes(&edge)?`.
    pub fn shapes(&self, s: &Shape) -> Result<List<Shape>, Failure> {
        if !self.dprism.is_done() {
            return Err(not_done("LocOpe_DPrism::Shapes"));
        }
        if let Some(l) = self.map.seek(s) {
            return Ok(l.clone());
        }
        Ok(self.dprism.generated_shapes(s, &self.profile2).clone())
    }
}
