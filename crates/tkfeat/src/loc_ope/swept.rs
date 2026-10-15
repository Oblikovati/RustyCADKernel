// Rust translation of the IntPerf shared by LocOpe_Prism, LocOpe_Revol, LocOpe_LinearForm
// and LocOpe_RevolutionForm (ModelingAlgorithms/TKFeat/LocOpe) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871: the four natives carry the same text with
// their own sweep.

//! The swept shape of a base (moved first by a transformation when given): a face is
//! swept as it is; any other base keeps the swept faces of its free edges only, closed
//! by its first and last shapes, when one of its edges is shared by two faces. Each
//! edge of the base maps to the face it sweeps.
use super::BuildShape;
use crate::errors::{no_such_object, not_done};
use rustycad_tkbrep::brep_tools::{Modifier, TrsfModification};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{DataMap, List, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point3, Transform3, Xyz};
use rustycad_tkprim::brep_sweep::{BRepSweepPrism, BRepSweepRevol};

/// The sweeps of BRepSweep the local operations use.
pub(crate) trait SweepTool {
    /// `FirstShape()`.
    fn first(&mut self) -> Result<Shape, Failure>;
    /// `LastShape()`.
    fn last(&mut self) -> Result<Shape, Failure>;
    /// `Shape()`.
    fn result(&mut self) -> Result<Shape, Failure>;
    /// `Shape(S)`: the shape swept by `s` (null when none).
    fn swept(&mut self, s: &Shape) -> Result<Shape, Failure>;
}

impl SweepTool for BRepSweepPrism {
    fn first(&mut self) -> Result<Shape, Failure> {
        self.first_shape()
    }
    fn last(&mut self) -> Result<Shape, Failure> {
        self.last_shape()
    }
    fn result(&mut self) -> Result<Shape, Failure> {
        self.shape()
    }
    fn swept(&mut self, s: &Shape) -> Result<Shape, Failure> {
        self.shape_of(s)
    }
}

impl SweepTool for BRepSweepRevol {
    fn first(&mut self) -> Result<Shape, Failure> {
        self.first_shape()
    }
    fn last(&mut self) -> Result<Shape, Failure> {
        self.last_shape()
    }
    fn result(&mut self) -> Result<Shape, Failure> {
        self.shape()
    }
    fn swept(&mut self, s: &Shape) -> Result<Shape, Failure> {
        self.shape_of(s)
    }
}

/// The shapes and the faces they sweep (`myMap`).
pub(crate) type SweptMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The state shared by the swept local operations.
#[derive(Clone, Default)]
pub(crate) struct Swept {
    pub(crate) base: Shape,
    pub(crate) done: bool,
    pub(crate) res: Shape,
    pub(crate) first_shape: Shape,
    pub(crate) last_shape: Shape,
    pub(crate) map: SweptMap,
}

/// The base moved by `trsf`, with the modifier that moved it.
fn moved_base(base: &Shape, trsf: Transform3) -> Result<(Shape, Modifier), Failure> {
    let mut modif = Modifier::new(false);
    let modbase: Handle<TrsfModification> = Handle::new(TrsfModification::new(trsf));
    modif.init(base)?;
    modif.perform(&modbase.cast(), &ProgressRange::default())?;
    let moved = modif.modified_shape(base)?;
    Ok((moved, modif))
}

/// Binds `edg` to the face it sweeps, unless bound already.
fn bind_swept(map: &mut SweptMap, edg: &Shape, sweep: &mut dyn SweepTool) -> Result<(), Failure> {
    if map.is_bound(edg) {
        return Ok(());
    }
    let mut thelist = List::default();
    let desc = sweep.swept(edg)?;
    if !desc.is_null() {
        thelist.append(desc);
    }
    map.bind(edg.clone(), thelist);
    Ok(())
}

impl Swept {
    /// Clears the results before a new Perform (`myMap.Clear()` ... `myRes.Nullify()`).
    pub(crate) fn reset(&mut self, base: &Shape) {
        self.map.clear(false);
        self.first_shape.nullify();
        self.last_shape.nullify();
        self.res.nullify();
        self.base = base.clone();
    }

    /// The faces swept by the free edges of a base other than a face, with the first
    /// and last shapes: rebuilt into a shape when an edge has two faces (Cas base !=
    /// FACE); None when every edge is free.
    fn non_face_result(
        &mut self,
        the_base: &Shape,
        sweep: &mut dyn SweepTool,
    ) -> Result<Option<Shape>, Failure> {
        let mut the_ef_map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            the_base,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut the_ef_map,
        )?;
        let mut lfaces = List::default();
        let mut toremove = false;
        for i in 1..=the_ef_map.extent() {
            let edg = the_ef_map.find_key(i)?.clone();
            let mut thelist = List::default();
            let desc = sweep.swept(&edg)?;
            if !desc.is_null() {
                if the_ef_map.find_from_index(i)?.len() >= 2 {
                    toremove = true;
                } else {
                    thelist.append(desc.clone());
                    lfaces.append(desc);
                }
            }
            self.map.bind(edg, thelist);
        }
        if !toremove {
            return Ok(None);
        }
        // Rajouter les faces de FirstShape et LastShape
        for f in Explorer::new(&self.first_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            lfaces.append(f);
        }
        for f in Explorer::new(&self.last_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            lfaces.append(f);
        }
        Ok(Some(BuildShape::with_faces(&lfaces)?.shape().clone()))
    }

    /// `IntPerf()`: sweeps the base (moved by `trsf` when given) with the sweep made by
    /// `make`, and maps the edges of the base to their swept faces.
    pub(crate) fn int_perf<S: SweepTool>(
        &mut self,
        trsf: Option<Transform3>,
        make: impl FnOnce(&Shape) -> Result<S, Failure>,
    ) -> Result<(), Failure> {
        let (the_base, modif) = match trsf {
            Some(t) => {
                let (moved, modif) = moved_base(&self.base, t)?;
                (moved, Some(modif))
            }
            None => (self.base.clone(), None),
        };
        let mut sweep = make(&the_base)?;
        self.first_shape = sweep.first()?;
        self.last_shape = sweep.last()?;
        let rebuilt = if the_base.shape_type()? == ShapeEnum::Face {
            None
        } else {
            self.non_face_result(&the_base, &mut sweep)?
        };
        self.res = match rebuilt {
            Some(res) => res,
            None => {
                for edg in Explorer::new(&the_base, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    bind_swept(&mut self.map, &edg, &mut sweep)?;
                }
                sweep.result()?
            }
        };
        if let Some(modif) = modif {
            self.rebind_moved_edges(&modif)?;
        }
        self.done = true;
        Ok(())
    }

    /// m-a-j des descendants: the descendants of the moved edges go to the edges of the
    /// base.
    fn rebind_moved_edges(&mut self, modif: &Modifier) -> Result<(), Failure> {
        for edg in Explorer::new(&self.base, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edgbis = modif.modified_shape(&edg)?;
            if edgbis.is_same(&edg) {
                continue;
            }
            if let Some(desc) = self.map.seek(&edgbis).cloned() {
                self.map.bind(edg, desc);
                self.map.unbind(&edgbis);
            }
        }
        Ok(())
    }

    /// `Shape()`: StdFail_NotDone before Perform.
    pub(crate) fn shape(&self, member: &str) -> Result<&Shape, Failure> {
        if !self.done {
            return Err(not_done(member));
        }
        Ok(&self.res)
    }

    /// `Shapes(S)`: the faces swept by `s`; Standard_NoSuchObject when `s` is not an
    /// edge of the base.
    pub(crate) fn shapes(&self, member: &str, s: &Shape) -> Result<&List<Shape>, Failure> {
        self.map
            .seek(s)
            .ok_or_else(|| no_such_object(member, "the shape is not an edge of the base"))
    }

    /// The sample points of the first shape and their barycentre.
    pub(crate) fn first_shape_samples(&self) -> Result<(Sequence<Point3>, Point3), Failure> {
        let mut spt = Sequence::default();
        super::sample_edges(&self.first_shape, &mut spt)?;
        let mut bar = Xyz::new(0., 0., 0.);
        for p in spt.iter() {
            bar += p.xyz();
        }
        bar /= spt.len() as f64;
        Ok((spt, Point3::new(bar.x(), bar.y(), bar.z())))
    }
}
