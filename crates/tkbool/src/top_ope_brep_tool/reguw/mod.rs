// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_REGUW.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871 (the debug traces are left out).

//! `TopOpeBRepTool_REGUW`: the regularization of a wire of a face: its edges split at
//! their internal vertices and the wire split into closed UV loops at its multiple
//! vertices.
mod walk;

use crate::top_ope_brep_tool::connexity::TopOpeBRepToolConnexity;
use crate::top_ope_brep_tool::corriso::TopOpeBRepToolCORRISO;
use crate::top_ope_brep_tool::tool::{
    ShapeListMap, TOOL_CLOSING, TOOL_INTERNAL, TopOpeBRepToolTOOL,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedDataMap, IndexedMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction2, Point2};

/// `FUN_tool_Add(map, key, subitem)`: the sub-item appended to the list of the key
/// (bound first when needed); `fun_tool_add(&mut map, &key, &item)`.
pub fn fun_tool_add(map: &mut ShapeListMap, key: &Shape, subitem: &Shape) {
    match map.change_seek(key) {
        Some(l) => {
            l.append(subitem.clone());
        }
        None => {
            let mut los: List<Shape> = List::default();
            los.append(subitem.clone());
            map.bind(key.clone(), los);
        }
    }
}

/// Standard_Failure of a regularization without shape.
pub(super) fn no_init() -> Failure {
    Failure::with_type(
        "Standard_Failure",
        "TopOpeBRepTool_REGUW : NO INIT",
        "TopOpeBRepTool_REGUW",
    )
}

/// The wire regularizer of a reference face;
/// `let mut r = TopOpeBRepToolREGUW::new(&f)?; r.init(&w)?;`.
pub struct TopOpeBRepToolREGUW {
    my_s: Shape,
    my_corriso: TopOpeBRepToolCORRISO,
    hasnewsplits: bool,
    my_esplits: ShapeListMap,
    my_ow_nw: ShapeListMap,
    mymapv_eds: IndexedDataMap<Shape, TopOpeBRepToolConnexity, ShapeMapHasher>,
    mymapvmultiple: Map<Shape, ShapeMapHasher>,
    my_list_vmultiple: List<Shape>,
    i_step: i32,
    mytol2d: f64,
    isinit0: bool,
    myv0: Vertex,
    myp2d0: Point2,
    myv: Vertex,
    myed: Edge,
    myp2d: Point2,
    mytg2d: Direction2,
}

impl TopOpeBRepToolREGUW {
    /// `TopOpeBRepTool_REGUW(Fref)`; `TopOpeBRepToolREGUW::new(&f)?`.
    pub fn new(fref: &Face) -> Result<Self, Failure> {
        Ok(Self {
            my_s: Shape::default(),
            my_corriso: TopOpeBRepToolCORRISO::with_face(fref)?,
            hasnewsplits: false,
            my_esplits: ShapeListMap::default(),
            my_ow_nw: ShapeListMap::default(),
            mymapv_eds: IndexedDataMap::default(),
            mymapvmultiple: Map::default(),
            my_list_vmultiple: List::default(),
            i_step: 0,
            mytol2d: 0.,
            isinit0: false,
            myv0: Vertex::default(),
            myp2d0: Point2::default(),
            myv: Vertex::default(),
            myed: Edge::default(),
            myp2d: Point2::default(),
            mytg2d: Direction2::X,
        })
    }

    /// `Fref()`; `r.fref()`.
    pub fn fref(&self) -> &Face {
        self.my_corriso.fref()
    }

    /// `SetEsplits(Esplits)`; `r.set_esplits(&map)`.
    pub fn set_esplits(&mut self, esplits: &ShapeListMap) {
        self.my_esplits = esplits.clone();
    }

    /// `GetEsplits(Esplits)`: Standard_Failure before `Init`; `r.get_esplits()?`.
    pub fn get_esplits(&self) -> Result<ShapeListMap, Failure> {
        self.check_init()?;
        Ok(self.my_esplits.clone())
    }

    /// `SetOwNw(OwNw)`; `r.set_ow_nw(&map)`.
    pub fn set_ow_nw(&mut self, ow_nw: &ShapeListMap) {
        self.my_ow_nw = ow_nw.clone();
    }

    /// `GetOwNw(OwNw)`: Standard_Failure before `Init`; `r.get_ow_nw()?`.
    pub fn get_ow_nw(&self) -> Result<ShapeListMap, Failure> {
        self.check_init()?;
        Ok(self.my_ow_nw.clone())
    }

    /// Standard_Failure before `Init`.
    fn check_init(&self) -> Result<(), Failure> {
        if self.has_init() {
            Ok(())
        } else {
            Err(no_init())
        }
    }

    /// The edges with internal vertices (INTERNAL items of the connexities).
    fn edges_with_internal_vertices(&self) -> Result<IndexedMap<Shape, ShapeMapHasher>, Failure> {
        let mut mehas_iv: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        for i in 1..=self.mymapv_eds.extent() {
            let (ni, loe) = self.mymapv_eds.find_from_index(i)?.is_internal()?;
            if ni == 0 {
                continue;
            }
            for e in loe.iter() {
                mehas_iv.add(e.clone());
            }
        }
        Ok(mehas_iv)
    }

    /// The INTERNAL items `e` replaced by its splits in the connexities of their vertices.
    fn replace_by_splits(&mut self, e: &Shape, splits: &List<Shape>) -> Result<(), Failure> {
        for esp in splits.iter() {
            for vv in Explorer::new(esp, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                let co = self.mymapv_eds.change_from_key(&vv)?;
                if !co.remove_item_of(TOOL_INTERNAL, e)? {
                    continue;
                }
                let ivv = TopOpeBRepToolTOOL::oriin_sorclosed(&vv, esp)?;
                co.add_item(ivv, esp)?;
            }
        }
        Ok(())
    }

    /// `SplitEds()`: the edges with internal vertices split (or their given splits
    /// used), their splits replacing them in the connexities; Standard_Failure before
    /// `Init`; `r.split_eds()?`.
    pub fn split_eds(&mut self) -> Result<bool, Failure> {
        self.check_init()?;
        let mehas_iv = self.edges_with_internal_vertices()?;
        for i in 1..=mehas_iv.extent() {
            let e = mehas_iv.find_key(i)?.clone();
            let mut splits: List<Shape> = List::default();
            let mut issp = false;
            let isdone = self.my_esplits.is_bound(&e);
            if isdone {
                splits = self.my_esplits.seek(&e).cloned().unwrap_or_default();
            } else {
                issp = TopOpeBRepToolTOOL::split_e(&topods::edge(&e)?, &mut splits)?;
            }
            if issp {
                self.hasnewsplits = true;
            }
            if !(issp || isdone) {
                continue;
            }
            self.replace_by_splits(&e, &splits)?;
        }
        Ok(true)
    }

    /// `S()`: Standard_Failure before `Init`; `r.s()?`.
    pub fn s(&self) -> Result<&Shape, Failure> {
        self.check_init()?;
        Ok(&self.my_s)
    }

    /// `Init(S)`; `r.init(&w)?`.
    pub fn init(&mut self, s: &Shape) -> Result<(), Failure> {
        self.my_s = s.clone();
        self.init_step(s)
    }

    /// `HasInit()`; `r.has_init()`.
    pub fn has_init(&self) -> bool {
        !self.my_s.is_null()
    }

    /// `InitStep(S)` (private): the UV corrector on the shape, the connexities and the
    /// multiple vertices cleared.
    fn init_step(&mut self, s: &Shape) -> Result<(), Failure> {
        self.check_init()?;
        self.my_corriso.init(&Shape::default())?;
        self.my_corriso.init(s)?;
        self.mymapv_eds.clear(false);
        self.mymapvmultiple.clear(false);
        self.my_list_vmultiple.clear();
        Ok(())
    }

    /// The connexity item of a vertex in an edge (MapS); `true` to stop at this vertex
    /// (a degenerated edge).
    fn map_vertex(
        &mut self,
        v: &Vertex,
        ed: &Edge,
        (isdg_e, isc_e, vcl): (bool, bool, &Option<Vertex>),
    ) -> Result<bool, Failure> {
        if !self.mymapv_eds.contains(v) {
            self.mymapv_eds
                .add(v.clone().into_shape(), TopOpeBRepToolConnexity::with_key(v));
        }
        let co = self.mymapv_eds.change_from_key(v)?;
        if isdg_e {
            co.remove_item_of(TOOL_CLOSING, ed)?;
            co.add_item(TOOL_CLOSING, ed)?;
            return Ok(true);
        }
        if isc_e {
            co.add_item(TOOL_CLOSING, ed)?;
        } else if vcl.as_ref().is_some_and(|c| v.is_same(c)) {
            co.remove_item_of(TOOL_CLOSING, ed)?;
            co.add_item(TOOL_CLOSING, ed)?;
        } else {
            let iov = TopOpeBRepToolTOOL::oriin_sor(v, ed, false)?;
            co.add_item(iov, ed)?;
        }
        Ok(false)
    }

    /// `MapS()`: the connexity of each vertex of the shape (CLOSING for closing,
    /// degenerated and closed edges), then the multiple vertices; false without vertex
    /// or for a faulty vertex; Standard_Failure before `Init`; `r.map_s()?`.
    pub fn map_s(&mut self) -> Result<bool, Failure> {
        self.check_init()?;
        let cs = self.my_corriso.s().clone();
        for es in Explorer::new(&cs, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let ed = topods::edge(&es)?;
            let isdg_e = BRepTool::degenerated(&ed)?;
            let isc_e = TopOpeBRepToolTOOL::is_closing_e_in(&ed, &cs, self.fref())? && !isdg_e;
            let vcl = if isdg_e {
                None
            } else {
                TopOpeBRepToolTOOL::closed_e(&ed)?
            };
            for vs in Explorer::new(&ed, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                let v = topods::vertex(&vs)?;
                if self.map_vertex(&v, &ed, (isdg_e, isc_e, &vcl))? {
                    break;
                }
            }
        }
        if self.mymapv_eds.extent() < 1 {
            return Ok(false);
        }
        for i in 1..=self.mymapv_eds.extent() {
            let v = self.mymapv_eds.find_key(i)?.clone();
            let co = self.mymapv_eds.find_from_index(i)?;
            if co.is_faulty() {
                return Ok(false);
            }
            if co.is_multiple() && self.mymapvmultiple.add(v.clone()) {
                self.my_list_vmultiple.append(v);
            }
        }
        Ok(true)
    }

    /// `GetSplits(Splits)`: the splits stored for the shape; Standard_Failure before
    /// `Init`; `r.get_splits()?`.
    pub fn get_splits(&self) -> Result<Option<List<Shape>>, Failure> {
        self.check_init()?;
        Ok(self.my_ow_nw.seek(&self.my_s).cloned())
    }

    /// `Connexity(v, co)`: a copy of the connexity of the vertex; Standard_Failure before
    /// `Init`; `r.connexity(&v)?`.
    pub fn connexity(&self, v: &Vertex) -> Result<Option<TopOpeBRepToolConnexity>, Failure> {
        self.check_init()?;
        Ok(self.mymapv_eds.seek(v).cloned())
    }

    /// `AddNewConnexity(v, OriKey, e)`: the edge added to the UV corrector and to the
    /// connexity of a known vertex; Standard_Failure before `Init`;
    /// `r.add_new_connexity(&v, 1, &e)?`.
    pub fn add_new_connexity(
        &mut self,
        v: &Vertex,
        ori_key: i32,
        e: &Edge,
    ) -> Result<bool, Failure> {
        self.check_init()?;
        if !self.mymapv_eds.contains(v) {
            return Ok(false);
        }
        if !self.my_corriso.add_new_connexity(v, e)? {
            return Ok(false);
        }
        self.mymapv_eds.change_from_key(v)?.add_item(ori_key, e)?;
        Ok(true)
    }

    /// `RemoveOldConnexity(v, OriKey, e)`: the edge removed from the UV corrector and
    /// from the connexity of a known vertex; Standard_Failure before `Init`;
    /// `r.remove_old_connexity(&v, 1, &e)?`.
    pub fn remove_old_connexity(
        &mut self,
        v: &Vertex,
        ori_key: i32,
        e: &Edge,
    ) -> Result<bool, Failure> {
        self.check_init()?;
        if !self.mymapv_eds.contains(v) {
            return Ok(false);
        }
        self.my_corriso.remove_old_connexity(v, e)?;
        self.mymapv_eds
            .change_from_key(v)?
            .remove_item_of(ori_key, e)
    }

    /// `UpdateMultiple(v)`: a known vertex recorded as multiple when it is;
    /// Standard_Failure before `Init`; `r.update_multiple(&v)?`.
    pub fn update_multiple(&mut self, v: &Vertex) -> Result<bool, Failure> {
        self.check_init()?;
        let Some(co) = self.mymapv_eds.seek(v) else {
            return Ok(false);
        };
        if co.is_multiple() && self.mymapvmultiple.add(v.clone().into_shape()) {
            self.my_list_vmultiple.append(v.clone().into_shape());
        }
        Ok(true)
    }
}
