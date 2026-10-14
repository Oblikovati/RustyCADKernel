// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_REGUS.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_REGUS`: the regularization of a shell: its faces split at their
//! internal edges, then the shell split into connected blocks of faces at the edges
//! shared by more than two faces.
mod blocks;

use crate::top_ope_brep_tool::classi::TopOpeBRepToolCLASSI;
use crate::top_ope_brep_tool::reguw::{TopOpeBRepToolREGUW, fun_tool_add};
use crate::top_ope_brep_tool::tool::{ShapeListMap, TOOL_INTERNAL, TopOpeBRepToolTOOL};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The shell regularizer; `let mut r = TopOpeBRepToolREGUS::new(); r.init(&sh);`.
#[derive(Default)]
pub struct TopOpeBRepToolREGUS {
    hasnewsplits: bool,
    my_fsplits: ShapeListMap,
    my_osh_nsh: ShapeListMap,
    my_s: Shape,
    mymape_fsstatic: ShapeListMap,
    mymape_fs: ShapeListMap,
    mymapemult: IndexedMap<Shape, ShapeMapHasher>,
    myn_f: i32,
    myoldn_f: i32,
    myf: Shape,
    myedstoconnect: Map<Shape, ShapeMapHasher>,
    myl_fin_block: List<Shape>,
}

impl TopOpeBRepToolREGUS {
    /// `TopOpeBRepTool_REGUS()`; `TopOpeBRepToolREGUS::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init(S)`: the shell, the face counts and maps cleared; `r.init(&sh)`.
    pub fn init(&mut self, s: &Shape) {
        self.hasnewsplits = false;
        (self.myn_f, self.myoldn_f) = (0, 0);
        self.my_s = s.clone();
        self.mymape_fs.clear(false);
        self.mymape_fsstatic.clear(false);
        self.mymapemult.clear(false);
        self.myedstoconnect.clear(false);
    }

    /// `S()`; `r.s()`.
    pub fn s(&self) -> &Shape {
        &self.my_s
    }

    /// `SetFsplits(Fsplits)`; `r.set_fsplits(&map)`.
    pub fn set_fsplits(&mut self, fsplits: &ShapeListMap) {
        self.my_fsplits = fsplits.clone();
    }

    /// `GetFsplits(Fsplits)`; `r.get_fsplits()`.
    pub fn get_fsplits(&self) -> ShapeListMap {
        self.my_fsplits.clone()
    }

    /// `SetOshNsh(OshNsh)`; `r.set_osh_nsh(&map)`.
    pub fn set_osh_nsh(&mut self, osh_nsh: &ShapeListMap) {
        self.my_osh_nsh = osh_nsh.clone();
    }

    /// `GetOshNsh(OshNsh)`; `r.get_osh_nsh()`.
    pub fn get_osh_nsh(&self) -> ShapeListMap {
        self.my_osh_nsh.clone()
    }

    /// `MapS()`: the faces of each edge of the shell (twice: one map kept static) and
    /// the edges of more than two faces; `r.map_s()?`.
    pub fn map_s(&mut self) -> Result<bool, Failure> {
        self.myoldn_f = 0;
        for f in Explorer::new(&self.my_s, ShapeEnum::Face, ShapeEnum::Shape)? {
            self.myoldn_f += 1;
            for e in Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                fun_tool_add(&mut self.mymape_fs, &e, &f);
                fun_tool_add(&mut self.mymape_fsstatic, &e, &f);
            }
        }
        self.myn_f = self.myoldn_f;
        for (e, lof) in self.mymape_fs.items() {
            if lof.extent() > 2 {
                self.mymapemult.add(e.clone());
            }
        }
        Ok(true)
    }

    /// `WireToFace(Fanc, nWs, nFs)`: the new wires classified on the face and one face
    /// per outer wire; `TopOpeBRepToolREGUS::wire_to_face(&f, &wires, &mut faces)?`.
    pub fn wire_to_face(
        fanc: &Face,
        n_ws: &List<Shape>,
        n_fs: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        n_fs.clear();
        let mut map_wlow = ShapeListMap::default();
        let a_face = topods::face(&fanc.oriented(Orientation::Forward))?;
        let mut classi = TopOpeBRepToolCLASSI::new();
        classi.init2d(&a_face);
        if !classi.classilist(n_ws, &mut map_wlow)? {
            return Ok(false);
        }
        TopOpeBRepToolTOOL::wire_to_face(fanc, &map_wlow, n_fs)
    }

    /// The INTERNAL edge replaced by its FORWARD and REVERSED occurrences in the
    /// connexities of its vertices; false when an update fails.
    fn split_internal_edge(reguw: &mut TopOpeBRepToolREGUW, e_i: &Shape) -> Result<bool, Failure> {
        let e_i_edge = topods::edge(e_i)?;
        let e_r = topods::edge(&e_i.oriented(Orientation::Reversed))?;
        let e_f = topods::edge(&e_i.oriented(Orientation::Forward))?;
        for vs in Explorer::new(e_i, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            let v = topods::vertex(&vs)?;
            if !reguw.remove_old_connexity(&v, TOOL_INTERNAL, &e_i_edge)? {
                return Ok(false);
            }
            let iv_f = TopOpeBRepToolTOOL::oriin_sor(&v, &e_f, false)?;
            if !reguw.add_new_connexity(&v, iv_f, &e_f)? {
                return Ok(false);
            }
            let iv_r = TopOpeBRepToolTOOL::oriin_sor(&v, &e_r, false)?;
            if !reguw.add_new_connexity(&v, iv_r, &e_r)? || !reguw.update_multiple(&v)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The count of the connexity items of a vertex (0 when it is unknown).
    fn connexity_count(reguw: &TopOpeBRepToolREGUW, v: &topods::Vertex) -> Result<i32, Failure> {
        Ok(reguw.connexity(v)?.map_or(0, |c| c.all_items().0))
    }

    /// The INTERNAL edges of a wire whose both vertices hold other edges, split in the
    /// connexities; false when an update fails.
    fn split_internal_edges(reguw: &mut TopOpeBRepToolREGUW, w: &Shape) -> Result<bool, Failure> {
        for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if e.orientation() != Orientation::Internal {
                continue;
            }
            let (vf, vl) =
                topexp::vertices(&topods::edge(&e.oriented(Orientation::Forward))?, false);
            if Self::connexity_count(reguw, &vf)? <= 1 || Self::connexity_count(reguw, &vl)? <= 1 {
                continue;
            }
            if !Self::split_internal_edge(reguw, &e)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `SplitF(Fanc, FSplits)`: the wires of the face regularized with their INTERNAL
    /// edges taken both ways, then the faces of the new wires (with the orientation of
    /// the face); false when no wire is split or the walk fails;
    /// `TopOpeBRepToolREGUS::split_f(&f, &mut splits)?`.
    pub fn split_f(fanc: &Face, f_splits: &mut List<Shape>) -> Result<bool, Failure> {
        let o_anc = fanc.orientation();
        let a_face = topods::face(&fanc.oriented(Orientation::Forward))?;
        f_splits.clear();
        let mut reguw = TopOpeBRepToolREGUW::new(&a_face)?;
        let mut n_ws: List<Shape> = List::default();
        let mut hassp = false;
        for w in Explorer::new(&a_face, ShapeEnum::Wire, ShapeEnum::Shape)? {
            reguw.init(&w)?;
            reguw.map_s()?;
            if !Self::split_internal_edges(&mut reguw, &w)? || !reguw.regu()? {
                return Ok(false);
            }
            let sp_w = reguw.get_splits()?.unwrap_or_default();
            if !sp_w.is_empty() {
                for s in sp_w.iter() {
                    n_ws.append(s.clone());
                }
                hassp = true;
            }
        }
        if !hassp {
            return Ok(false);
        }
        let mut n_fs: List<Shape> = List::default();
        if !Self::wire_to_face(&a_face, &n_ws, &mut n_fs)? {
            return Ok(false);
        }
        for f in n_fs.iter() {
            f_splits.append(f.oriented(o_anc));
        }
        Ok(true)
    }

    /// The split faces replacing the face in the faces of their edges.
    fn record_split(&mut self, f: &Shape, lfsp: &List<Shape>) -> Result<bool, Failure> {
        self.myn_f -= 1;
        for fsp in lfsp.iter() {
            self.myn_f += 1;
            for e in Explorer::new(fsp, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let Some(lof) = self.mymape_fs.change_seek(&e) else {
                    return Ok(false);
                };
                TopOpeBRepToolTOOL::remove(lof, f);
                lof.append(fsp.clone());
                if lof.extent() > 2 {
                    self.mymapemult.add(e);
                }
            }
        }
        Ok(true)
    }

    /// `SplitFaces()`: each face of the shell split (`SplitF`), its splits recorded;
    /// false when an edge of a split is unknown; `r.split_faces()?`.
    pub fn split_faces(&mut self) -> Result<bool, Failure> {
        let s = self.my_s.clone();
        for fs in Explorer::new(&s, ShapeEnum::Face, ShapeEnum::Shape)? {
            let f = topods::face(&fs)?;
            let mut lfsp: List<Shape> = List::default();
            if !Self::split_f(&f, &mut lfsp)? {
                continue;
            }
            self.my_fsplits.bind(fs.clone(), lfsp.clone());
            if !self.record_split(&fs, &lfsp)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
