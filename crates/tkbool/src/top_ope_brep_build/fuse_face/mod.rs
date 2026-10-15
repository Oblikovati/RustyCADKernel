// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_FuseFace
// (.hxx/.lxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fusion of the faces of a result sharing edges into larger faces
//! (`TopOpeBRepBuild_FuseFace`).
mod group;

use group::{ShapeLists, group_edge, group_shape, make_edge, same_support};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeFace, BRepLibMakeWire};

/// The fusion of faces (`TopOpeBRepBuild_FuseFace`);
/// `let mut ff = TopOpeBRepBuildFuseFace::with_faces(&lif, &lrf, 1);`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildFuseFace {
    my_lie: List<Shape>,
    my_lee: List<Shape>,
    my_lme: List<Shape>,
    my_liv: List<Shape>,
    my_lev: List<Shape>,
    my_lmv: List<Shape>,
    my_lif: List<Shape>,
    my_lrf: List<Shape>,
    my_lff: List<Shape>,
    my_internal: bool,
    my_modified: bool,
    my_done: bool,
}

/// The result of a step: Some(()) to go on, None when the faces are kept unfused.
type Step<T> = Result<Option<T>, Failure>;

impl TopOpeBRepBuildFuseFace {
    /// `TopOpeBRepBuild_FuseFace()`; `TopOpeBRepBuildFuseFace::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_FuseFace(LIF, LRF, CXM)`; `TopOpeBRepBuildFuseFace::with_faces(&lif, &lrf, 1)`.
    pub fn with_faces(lif: &List<Shape>, lrf: &List<Shape>, cxm: i32) -> Self {
        let mut ff = Self::new();
        ff.init(lif, lrf, cxm);
        ff
    }

    /// `Init(LIF, LRF, CXM)`: the faces to fuse `lrf`, the internal edges kept for CXM 2
    /// (not for 1), all results cleared; `ff.init(&lif, &lrf, 1)`.
    pub fn init(&mut self, lif: &List<Shape>, lrf: &List<Shape>, cxm: i32) {
        self.my_lif = lif.clone();
        self.my_lrf = lrf.clone();
        match cxm {
            1 => self.my_internal = false,
            2 => self.my_internal = true,
            _ => {}
        }
        for l in [
            &mut self.my_lff,
            &mut self.my_lie,
            &mut self.my_lee,
            &mut self.my_lme,
            &mut self.my_liv,
            &mut self.my_lev,
            &mut self.my_lmv,
        ] {
            l.clear();
        }
        self.my_modified = false;
        self.my_done = false;
    }

    /// The faces kept unfused: done, not modified, the result the faces to fuse.
    fn keep_unfused(&mut self) {
        self.my_modified = false;
        self.my_done = true;
        self.my_lff = self.my_lrf.clone();
    }

    /// The forward faces to fuse when all valid and of one orientation, and whether
    /// they are reversed.
    fn forward_faces(&self) -> Step<(List<Shape>, bool)> {
        let (mut fwd, mut rev, mut mylist) = (false, false, List::default());
        for fac in self.my_lrf.iter() {
            match fac.orientation() {
                Orientation::Forward => fwd = true,
                Orientation::Reversed => rev = true,
                _ => {}
            }
            if !BRepCheckAnalyzer::new(fac, true, false, false)?.is_valid_shape(fac)? {
                return Ok(None);
            }
            mylist.append(fac.oriented(Orientation::Forward));
        }
        Ok((!(fwd && rev)).then_some((mylist, rev)))
    }

    /// `PerformFace()`: the faces sharing edges grouped and each group fused into one
    /// face on the basis surface of its first face, with the wires of the fused edges
    /// and the INTERNAL/EXTERNAL (and, when kept, merged) edges as wires; the faces
    /// kept when anything fails; `ff.perform_face()?`.
    pub fn perform_face(&mut self) -> Result<(), Failure> {
        self.my_modified = false;
        self.my_lff.clear();
        if self.my_lrf.len() <= 1 {
            self.keep_unfused();
            return Ok(());
        }
        let Some((mylist, ori3d_reversed)) = self.forward_faces()? else {
            self.keep_unfused();
            return Ok(());
        };
        let mut map_fac = ShapeLists::default();
        group_shape(&mylist, false, &mut map_fac)?;
        if map_fac.is_empty() || self.my_lrf.len() == map_fac.len() {
            self.keep_unfused();
            return Ok(());
        }
        for (fac, lfac) in map_fac.items() {
            if lfac.len() == 1 {
                self.my_lff.append(fac.clone());
                continue;
            }
            match self.fused_face(fac, lfac, ori3d_reversed)? {
                Some(f) => {
                    self.my_lff.append(f);
                }
                None => {
                    self.keep_unfused();
                    return Ok(());
                }
            }
        }
        if self.my_lff.is_empty() {
            self.keep_unfused();
            return Ok(());
        }
        self.my_modified = true;
        self.my_done = true;
        Ok(())
    }

    /// The face of a group of faces (None to keep the faces unfused).
    fn fused_face(&mut self, fac: &Shape, lfac: &List<Shape>, ori3d_reversed: bool) -> Step<Shape> {
        let mut lwir = List::default();
        for fac1 in lfac.iter() {
            for w in Explorer::new(fac1, ShapeEnum::Wire, ShapeEnum::Shape)? {
                lwir.append(w);
            }
        }
        let mut map_wir = ShapeLists::default();
        group_shape(&lwir, false, &mut map_wir)?;
        if map_wir.is_empty() {
            return Ok(None);
        }
        let (mut face_lie, mut face_lee, mut face_lme, mut face_lw): (
            List<Shape>,
            List<Shape>,
            List<Shape>,
            List<Shape>,
        ) = Default::default();
        for (wir, lwir1) in map_wir.items() {
            if lwir1.len() == 1 {
                face_lw.append(wir.clone());
                continue;
            }
            if !self.fuse_wires(
                lwir1,
                (&mut face_lie, &mut face_lee, &mut face_lme, &mut face_lw),
            )? {
                return Ok(None);
            }
        }
        self.face_of_wires(
            fac,
            (&face_lie, &face_lee, &face_lme, &face_lw),
            ori3d_reversed,
        )
    }

    /// The edges of a group of wires sorted by orientation: the oriented ones into
    /// wires, the others recorded (false to keep the faces unfused).
    fn fuse_wires(
        &mut self,
        lwir1: &List<Shape>,
        lists: (
            &mut List<Shape>,
            &mut List<Shape>,
            &mut List<Shape>,
            &mut List<Shape>,
        ),
    ) -> Result<bool, Failure> {
        let (face_lie, face_lee, face_lme, face_lw) = lists;
        let mut ledg = List::default();
        for wir1 in lwir1.iter() {
            for e in Explorer::new(wir1, ShapeEnum::Edge, ShapeEnum::Shape)? {
                ledg.append(e);
            }
        }
        let mut map_edg = ShapeLists::default();
        group_shape(&ledg, true, &mut map_edg)?;
        if map_edg.is_empty() {
            return Ok(false);
        }
        let mut wire_le = List::default();
        for (edg, ledg1) in map_edg.items() {
            let (mut r, mut f, mut i, mut x) = (false, false, false, false);
            for edg1 in ledg1.iter() {
                match edg1.orientation() {
                    Orientation::Reversed if r => return Ok(false),
                    Orientation::Reversed => r = true,
                    Orientation::Forward if f => return Ok(false),
                    Orientation::Forward => f = true,
                    Orientation::Internal => i = true,
                    Orientation::External => x = true,
                }
            }
            if r && f {
                let e = edg.oriented(Orientation::Internal);
                self.my_lme.append(e.clone());
                face_lme.append(e);
            } else if r || f {
                wire_le.append(edg.oriented(if r {
                    Orientation::Reversed
                } else {
                    Orientation::Forward
                }));
            } else if i {
                let e = edg.oriented(Orientation::Internal);
                self.my_lie.append(e.clone());
                face_lie.append(e);
            } else if x {
                let e = edg.oriented(Orientation::External);
                self.my_lee.append(e.clone());
                face_lee.append(e);
            }
        }
        wires_of_edges(wire_le, face_lw)
    }

    /// The face on the basis surface of `fac` with the wires and the INTERNAL/EXTERNAL
    /// (and merged when internal) edges as wires (None when it fails).
    fn face_of_wires(
        &self,
        fac: &Shape,
        lists: (&List<Shape>, &List<Shape>, &List<Shape>, &List<Shape>),
        ori3d_reversed: bool,
    ) -> Step<Shape> {
        let (face_lie, face_lee, face_lme, face_lw) = lists;
        let mut s: Handle<dyn GeomSurface> = BRepTool::surface(&topods::face(fac)?)?;
        if s.is_instance_name("Geom_RectangularTrimmedSurface") {
            let t: Handle<GeomRectangularTrimmedSurface> = s.clone().cast();
            s = t
                .get()
                .ok_or_else(|| Failure::new("FuseFace: null surface"))?
                .basis_surface();
        }
        let mut mf = BRepLibMakeFace::from_surface(&s, precision::confusion())?;
        for w in face_lw.iter() {
            mf.add(&topods::wire(w)?)?;
        }
        let mut single = |l: &List<Shape>| -> Result<(), Failure> {
            for e in l.iter() {
                let w = BRepLibMakeWire::from_edge(&topods::edge(e)?)?.wire()?;
                mf.add(&w)?;
            }
            Ok(())
        };
        single(face_lie)?;
        single(face_lee)?;
        if self.my_internal {
            single(face_lme)?;
        }
        if !mf.is_done() {
            return Ok(None);
        }
        let mut f = mf.face()?.into_shape();
        if ori3d_reversed {
            f.reverse();
        }
        Ok(Some(f))
    }

    /// `PerformEdge()`: the edges of the fused faces meeting two by two at a vertex on
    /// the same support grouped and made into edges (discarded, as natively);
    /// `ff.perform_edge()?`.
    pub fn perform_edge(&mut self) -> Result<(), Failure> {
        let mut map_ver = ShapeLists::default();
        for fac in self.my_lff.iter() {
            for wir in Explorer::new(fac, ShapeEnum::Wire, ShapeEnum::Shape)? {
                for edg in Explorer::new(&wir, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    for ver in Explorer::new(&edg, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                        if !map_ver.is_bound(&ver) {
                            map_ver.bind(ver.clone(), List::default());
                        }
                        map_ver.change_find(&ver)?.append(edg.clone());
                    }
                }
            }
        }
        let mut kept = ShapeLists::default();
        for (ver, ledg) in map_ver.items() {
            if ledg.len() != 2 {
                continue;
            }
            let mut it = ledg.iter();
            let (e1, e2) = (
                topods::edge(it.next().unwrap_or(ver))?,
                topods::edge(it.next().unwrap_or(ver))?,
            );
            if same_support(&e1, &e2)? {
                kept.bind(ver.clone(), ledg.clone());
            }
        }
        let mut map_edg = ShapeLists::default();
        group_edge(&kept, &mut map_edg)?;
        make_edge(&mut map_edg)?;
        self.my_modified = true;
        self.my_done = true;
        Ok(())
    }

    /// The face of the wires of one fused face (None when a wire or the face fails).
    fn clear_face(&mut self, fac: &Shape) -> Step<Shape> {
        let mut face_lw = List::default();
        for wir in Explorer::new(fac, ShapeEnum::Wire, ShapeEnum::Shape)? {
            let mut wire_le = List::default();
            for edg in Explorer::new(&wir, ShapeEnum::Edge, ShapeEnum::Shape)? {
                match edg.orientation() {
                    Orientation::Internal => self.my_lie.append(edg),
                    Orientation::External => self.my_lee.append(edg),
                    _ => wire_le.append(edg),
                };
            }
            if !wire_le.is_empty() {
                let mut mw = BRepLibMakeWire::new();
                mw.add_list(&wire_le)?;
                if !mw.is_done() {
                    return Ok(None);
                }
                face_lw.append(
                    BRepLibMakeWire::from_wire(&mw.wire()?)?
                        .wire()?
                        .into_shape(),
                );
            }
        }
        let mut it = face_lw.iter();
        let Some(first) = it.next() else {
            return Ok(None);
        };
        let mut mf = BRepLibMakeFace::from_wire(&topods::wire(first)?, false)?;
        for w in it {
            mf.add(&topods::wire(w)?)?;
        }
        if !mf.is_done() {
            return Ok(None);
        }
        Ok(Some(mf.face()?.into_shape()))
    }

    /// `ClearEdge()`: the fused faces rebuilt from their oriented edges only (the
    /// INTERNAL and EXTERNAL ones recorded); the faces kept when anything fails;
    /// `ff.clear_edge()?`.
    pub fn clear_edge(&mut self) -> Result<(), Failure> {
        let mut new_lff = List::default();
        for fac in self.my_lff.clone().iter() {
            match self.clear_face(fac)? {
                Some(f) => {
                    new_lff.append(f);
                }
                None => {
                    self.keep_unfused();
                    return Ok(());
                }
            }
        }
        if new_lff.is_empty() {
            self.keep_unfused();
            return Ok(());
        }
        self.my_lff = new_lff;
        self.my_modified = true;
        self.my_done = true;
        Ok(())
    }

    /// `ClearVertex()`: nothing, as natively; `ff.clear_vertex()`.
    pub fn clear_vertex(&mut self) {}

    /// `IsDone()`; `ff.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }

    /// `IsModified()`; `ff.is_modified()`.
    pub fn is_modified(&self) -> bool {
        self.my_modified
    }

    /// `LFuseFace()`; `ff.l_fuse_face()`.
    pub fn l_fuse_face(&self) -> &List<Shape> {
        &self.my_lff
    }

    /// `LInternEdge()`; `ff.l_intern_edge()`.
    pub fn l_intern_edge(&self) -> &List<Shape> {
        &self.my_lie
    }

    /// `LExternEdge()`; `ff.l_extern_edge()`.
    pub fn l_extern_edge(&self) -> &List<Shape> {
        &self.my_lee
    }

    /// `LModifEdge()`; `ff.l_modif_edge()`.
    pub fn l_modif_edge(&self) -> &List<Shape> {
        &self.my_lme
    }

    /// `LInternVertex()`; `ff.l_intern_vertex()`.
    pub fn l_intern_vertex(&self) -> &List<Shape> {
        &self.my_liv
    }

    /// `LExternVertex()`; `ff.l_extern_vertex()`.
    pub fn l_extern_vertex(&self) -> &List<Shape> {
        &self.my_lev
    }

    /// `LModifVertex()`; `ff.l_modif_vertex()`.
    pub fn l_modif_vertex(&self) -> &List<Shape> {
        &self.my_lmv
    }
}

/// The wires of the oriented edges: one wire per connected chain (made again from
/// itself), until every edge is in a wire (false when a wire fails).
fn wires_of_edges(mut wire_le: List<Shape>, face_lw: &mut List<Shape>) -> Result<bool, Failure> {
    let mut number1 = wire_le.len();
    while number1 > 0 {
        let mut mw = BRepLibMakeWire::new();
        mw.add_list(&wire_le)?;
        if !mw.is_done() {
            return Ok(false);
        }
        let w: Wire = BRepLibMakeWire::from_wire(&mw.wire()?)?.wire()?;
        face_lw.append(w.clone().into_shape());
        let mut m: Map<Shape, ShapeMapHasher> = Map::default();
        let mut nb = 0;
        for e in Explorer::new(w.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
            m.add(e);
            nb += 1;
        }
        if nb == number1 {
            number1 = 0;
        } else {
            let mut rest = List::default();
            for e in wire_le.iter() {
                if m.add(e.clone()) {
                    rest.append(e.clone());
                }
            }
            wire_le = rest;
            number1 = wire_le.len();
        }
    }
    Ok(true)
}
