// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Generator.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_Generator`: the shape rebuilt with a generated shape glued on it: the faces
//! of the basis left of the generating edges are removed, coplanar faces and coaxial
//! edges are fused, the touched faces rebuilt, and the shell or solid rebuilt on the
//! remaining and generated faces.
mod faces;
mod fusions;
mod statics;

use super::{BuildShape, GeneratedShape};
use crate::errors::{no_such_object, not_done, null_object};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape, Solid};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The shapes and the shapes they are fused with or modified into.
pub(super) type ShapeLists = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `LocOpe_Generator`: the shape with a generated shape glued on it.
///
/// ```
/// use rustycad_tkfeat::loc_ope::Generator;
/// let generator = Generator::new();
/// assert!(!generator.is_done());
/// ```
#[derive(Clone, Default)]
pub struct Generator {
    shape: Shape,
    done: bool,
    res: Shape,
    mod_shapes: ShapeLists,
}

/// The working maps of one Perform.
pub(super) struct Run<'a> {
    pub(super) shape: &'a Shape,
    pub(super) g: &'a mut dyn GeneratedShape,
    pub(super) mod_shapes: ShapeLists,
    /// Faces a gauche.
    pub(super) the_left: MapOfShape,
    /// Edges et vertex generateurs.
    pub(super) g_edg: MapOfShape,
    pub(super) g_vtx: MapOfShape,
    pub(super) the_ff_map: ShapeLists,
    pub(super) the_ee_map: ShapeLists,
    pub(super) to_remove: MapOfShape,
    pub(super) dont_fuse: DataMap<Shape, Shape, ShapeMapHasher>,
}

impl Run<'_> {
    /// The faces of the result: the remaining faces of the basis (or their first
    /// rebuilt descendant) and the generated faces not removed, and the first kept face
    /// of the basis.
    fn result_faces(&mut self) -> Result<(List<Shape>, Shape), Failure> {
        let mut face_ref_ori = Shape::default();
        let mut lfres = List::default();
        for fac in Explorer::new(self.shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if self.the_left.contains(&fac) {
                continue;
            }
            if !self.to_remove.contains(&fac) {
                lfres.append(fac.clone());
                if !self.mod_shapes.is_bound(&fac) {
                    self.mod_shapes.bind(fac.clone(), List::default());
                }
                if let Some(l) = self.mod_shapes.change_seek(&fac) {
                    l.append(fac.clone());
                }
                if face_ref_ori.is_null() {
                    face_ref_ori = fac;
                }
            } else if let Some(l) = self.mod_shapes.seek(&fac) {
                let first = l.first().ok_or_else(|| {
                    no_such_object(
                        "LocOpe_Generator::Perform",
                        "removed face without descendant",
                    )
                })?;
                lfres.append(first.oriented(fac.orientation()));
            }
        }
        let orsolid = self.shape.orientation();
        let oriented: Vec<Shape> = self.g.oriented_faces()?.iter().cloned().collect();
        for fac in oriented {
            if self.to_remove.contains(&fac) {
                continue;
            }
            lfres.append(if orsolid == Orientation::Forward {
                fac.clone()
            } else {
                fac.reversed()
            });
            if !self.mod_shapes.is_bound(&fac) {
                self.mod_shapes.bind(fac.clone(), List::default());
            }
            if let Some(l) = self.mod_shapes.change_seek(&fac) {
                l.append(fac);
            }
        }
        Ok((lfres, face_ref_ori))
    }
}

/// The solid with its single shell reversed when the reference face lost its
/// orientation in the rebuilt solid (Debug temporaire du 06.11.96).
fn reoriented(res: Shape, face_ref_ori: &Shape) -> Result<Shape, Failure> {
    if res.shape_type()? != ShapeEnum::Solid {
        return Ok(res);
    }
    let found =
        Explorer::new(&res, ShapeEnum::Face, ShapeEnum::Shape)?.find(|x| x.is_same(face_ref_ori));
    let Some(found) = found else {
        return Ok(res);
    };
    if found.orientation() == face_ref_ori.orientation() {
        return Ok(res);
    }
    // Si un seul Shell, on change son orientation
    let b = BRepBuilder::default();
    let mut new_sol = Solid::default();
    b.make_solid(&mut new_sol);
    let mut new_sol = new_sol.into_shape();
    let shell = Explorer::new(&res, ShapeEnum::Shell, ShapeEnum::Shape)?
        .current()
        .clone();
    b.add(&mut new_sol, &shell.reversed())?;
    Ok(new_sol)
}

impl Generator {
    /// `LocOpe_Generator()`; `Generator::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_Generator(S)`: the algorithm on `s`; `Generator::with_shape(&s)`.
    pub fn with_shape(s: &Shape) -> Self {
        Self {
            shape: s.clone(),
            ..Self::default()
        }
    }

    /// `Init(S)`; `generator.init(&s)`.
    pub fn init(&mut self, s: &Shape) {
        self.shape = s.clone();
        self.done = false;
    }

    /// `IsDone()`; `generator.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `ResultingShape()`: StdFail_NotDone before Perform; `generator.resulting_shape()?`.
    pub fn resulting_shape(&self) -> Result<&Shape, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Generator::ResultingShape"));
        }
        Ok(&self.res)
    }

    /// `Shape()`: the initial shape; `generator.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// `DescendantFace(F)`: the faces `f` became (of the basis or the generated shape);
    /// Standard_NoSuchObject for a face neither kept nor modified;
    /// `generator.descendant_face(&f)?`.
    pub fn descendant_face(&self, f: &Face) -> Result<&List<Shape>, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Generator::DescendantFace"));
        }
        self.mod_shapes.seek(f).ok_or_else(|| {
            no_such_object(
                "LocOpe_Generator::DescendantFace",
                "the face has no descendant",
            )
        })
    }

    /// `Perform(G)`: the shape rebuilt with the generated shape `g`; Standard_NullObject
    /// without shape; `generator.perform(&mut glued)?`.
    pub fn perform(&mut self, g: &mut dyn GeneratedShape) -> Result<(), Failure> {
        if self.shape.is_null() {
            return Err(null_object("LocOpe_Generator::Perform", "no shape"));
        }
        self.done = false;
        self.res.nullify();
        self.mod_shapes.clear(false);
        let mut run = Run {
            shape: &self.shape,
            g,
            mod_shapes: DataMap::default(),
            the_left: MapOfShape::default(),
            g_edg: MapOfShape::default(),
            g_vtx: MapOfShape::default(),
            the_ff_map: DataMap::default(),
            the_ee_map: DataMap::default(),
            to_remove: MapOfShape::default(),
            dont_fuse: DataMap::default(),
        };
        run.collect_left()?;
        let the_ef_map = run.face_fusions()?;
        run.connexity_edges()?;
        let rebuild_face = run.edge_fusions(&the_ef_map)?;
        for f in rebuild_face {
            run.the_ff_map.bind(f, List::default());
        }
        run.fuse_edges()?;
        run.fuse_faces()?;
        let (lfres, face_ref_ori) = run.result_faces()?;
        let res = BuildShape::with_faces(&lfres)?.shape().clone();
        self.mod_shapes = run.mod_shapes;
        // recodage des regularites qui existaient sur le shape colle
        self.res = reoriented(res, &face_ref_ori)?;
        self.done = true;
        Ok(())
    }
}
