// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_Check.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_Check`: the integrity checks of the data structure (indices of the
//! interferences, same domain references, vertices on points) with their report.
use super::export::{href, iref};
use super::{
    ListOfInterference, TopOpeBRepDSCheckStatus, TopOpeBRepDSCurvePointInterference,
    TopOpeBRepDSEdgeVertexInterference, TopOpeBRepDSHDataStructure, TopOpeBRepDSInterferenceTool,
    TopOpeBRepDSKind, TopOpeBRepDSPointExplorer,
};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeTool;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, List};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::io::{self, Write};

/// The statuses of the checked elements by index.
type StatusMap = DataMap<i32, TopOpeBRepDSCheckStatus>;

/// The checker of a data structure; `let mut c = TopOpeBRepDSCheck::with_hds(&hds);`.
#[derive(Default)]
pub struct TopOpeBRepDSCheck {
    parent: StandardTransient,
    my_hds: Handle<TopOpeBRepDSHDataStructure>,
    my_map_surface_status: StatusMap,
    my_map_curve_status: StatusMap,
    my_map_point_status: StatusMap,
    my_map_shape_status: StatusMap,
}
impl_transient!(
    TopOpeBRepDSCheck,
    "TopOpeBRepDS_Check",
    StandardTransient,
    parent
);

/// The status of `i` set to NOK (rebound) and false.
fn nok(map: &mut StatusMap, i: i32) -> bool {
    if map.is_bound(&i) {
        map.unbind(&i);
    }
    map.bind(i, TopOpeBRepDSCheckStatus::Nok);
    false
}

/// The status of `i` set to OK unless already set, and true.
fn ok(map: &mut StatusMap, i: i32) -> bool {
    if !map.is_bound(&i) {
        map.bind(i, TopOpeBRepDSCheckStatus::Ok);
    }
    true
}

/// The shape type of a topological kind (COMPOUND for another kind).
fn kind_shape(k: TopOpeBRepDSKind) -> ShapeEnum {
    match k {
        TopOpeBRepDSKind::Solid => ShapeEnum::Solid,
        TopOpeBRepDSKind::Shell => ShapeEnum::Shell,
        TopOpeBRepDSKind::Face => ShapeEnum::Face,
        TopOpeBRepDSKind::Wire => ShapeEnum::Wire,
        TopOpeBRepDSKind::Edge => ShapeEnum::Edge,
        TopOpeBRepDSKind::Vertex => ShapeEnum::Vertex,
        _ => ShapeEnum::Compound,
    }
}

/// `CheckEdgeParameter(myHDS)`: no edge/vertex or curve/point parameter above 1e50
/// (natively truncated to an int first, so never).
fn check_edge_parameter(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<bool, Failure> {
    let ds = href(hds)?.ds();
    let mut is_ok = true;
    for i in 1..=ds.nb_shapes() {
        for i1 in ds.shape_interferences_index(i, true)?.iter() {
            let evi: Handle<TopOpeBRepDSEdgeVertexInterference> = i1.cast();
            if let Some(e) = evi.get() {
                is_ok &= f64::from(e.parameter() as i32) <= 1.0e50;
            }
        }
    }
    for i in 1..=ds.nb_curves() {
        for i1 in ds.curve_interferences(i).iter() {
            let cpi: Handle<TopOpeBRepDSCurvePointInterference> = i1.cast();
            if cpi.is_some() {
                let p = TopOpeBRepDSInterferenceTool::parameter(i1)?;
                is_ok &= f64::from(p as i32) <= 1.0e50;
            }
        }
    }
    Ok(is_ok)
}

impl TopOpeBRepDSCheck {
    /// `TopOpeBRepDS_Check()`: no structure; `TopOpeBRepDSCheck::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepDS_Check(HDS)`; `TopOpeBRepDSCheck::with_hds(&hds)`.
    pub fn with_hds(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Self {
        Self {
            my_hds: hds.clone(),
            ..Self::default()
        }
    }

    /// `ChkIntg()`: the indices of the interferences checked (natively the result is
    /// the check of the last shape list and-ed with those of the geometries);
    /// `c.chk_intg()?`.
    pub fn chk_intg(&mut self) -> Result<bool, Failure> {
        let lists = {
            let ds = href(&self.my_hds)?.ds();
            let mut shapes = Vec::new();
            for i in 1..=ds.nb_shapes() {
                shapes.push(ds.shape_interferences_index(i, true)?.clone());
            }
            let surfaces: Vec<_> = (1..=ds.nb_surfaces())
                .map(|i| ds.surface_interferences(i).clone())
                .collect();
            let curves: Vec<_> = (1..=ds.nb_curves())
                .map(|i| ds.curve_interferences(i).clone())
                .collect();
            let points: Vec<_> = (1..=ds.nb_points())
                .map(|i| ds.point_interferences(i).clone())
                .collect();
            (shapes, surfaces, curves, points)
        };
        let mut b_i = false;
        for li in &lists.0 {
            b_i = self.chk_intg_interf(li)?;
        }
        for li in lists.1.iter().chain(&lists.2).chain(&lists.3) {
            b_i = b_i && self.chk_intg_interf(li)?;
        }
        check_edge_parameter(&self.my_hds)?;
        Ok(b_i)
    }

    /// `ChkIntgInterf(LI)`: the supports and geometries of the interferences checked
    /// (stopping at the first failure); `c.chk_intg_interf(&l)?`.
    pub fn chk_intg_interf(&mut self, li: &ListOfInterference) -> Result<bool, Failure> {
        let mut is_ok = true;
        for i1 in li.iter() {
            let ii = iref(i1)?;
            is_ok = is_ok && self.check_ds(ii.support(), ii.support_type())?;
            is_ok = is_ok && self.check_ds(ii.geometry(), ii.geometry_type())?;
        }
        Ok(is_ok)
    }

    /// `CheckDS(I, K)`: the index in range of its kind (and of the type of a shape),
    /// its status recorded; `c.check_ds(1, TopOpeBRepDSKind::Face)?`.
    pub fn check_ds(&mut self, i: i32, k: TopOpeBRepDSKind) -> Result<bool, Failure> {
        let h = href(&self.my_hds)?;
        let (map, n) = match k {
            TopOpeBRepDSKind::Surface => (&mut self.my_map_surface_status, h.nb_surfaces()),
            TopOpeBRepDSKind::Curve => (&mut self.my_map_curve_status, h.nb_curves()),
            TopOpeBRepDSKind::Point => (&mut self.my_map_point_status, h.nb_points()),
            _ => (&mut self.my_map_shape_status, h.nb_shapes()),
        };
        if n < i {
            return Ok(nok(map, i));
        }
        if matches!(
            k,
            TopOpeBRepDSKind::Surface | TopOpeBRepDSKind::Curve | TopOpeBRepDSKind::Point
        ) {
            return Ok(ok(map, i));
        }
        let s = h.shape(i, true);
        if s.shape_type()? != kind_shape(k) {
            return Ok(nok(map, i));
        }
        Ok(ok(map, i))
    }

    /// Whether the shape of index `i` (or its same domain reference) is consistent
    /// (`ChkIntgSamDom` loop body).
    fn sam_dom_ok(
        &self,
        h: &TopOpeBRepDSHDataStructure,
        i: i32,
        bb: &mut bool,
    ) -> Result<bool, Failure> {
        let ds = h.ds();
        let sind = h.shape(i, true);
        let mut b = self.check_shapes(ds.shape_same_domain(&sind))?;
        let curr = ds.same_domain_ref_index(i);
        let loc = ds.same_domain_ref_index(curr);
        if curr != 0 && curr != loc {
            b = false;
        }
        if curr == 0 {
            return Ok(b);
        }
        let sref = h.shape(curr, true);
        if sind.shape_type()? != sref.shape_type()? {
            b = false;
        }
        if i != curr {
            for sh in ds.shape_same_domain(&sref).iter() {
                if h.shape_index(sh, true) == i {
                    *bb = true;
                    break;
                }
            }
            b = b && *bb;
        }
        Ok(b)
    }

    /// `ChkIntgSamDom()`: the same domain lists and references consistent (natively the
    /// found flag is never reset between shapes); `c.chk_intg_sam_dom()?`.
    pub fn chk_intg_sam_dom(&mut self) -> Result<bool, Failure> {
        let h = href(&self.my_hds)?;
        let (mut b, mut bb) = (true, false);
        for i in 1..=h.nb_shapes() {
            if !self.sam_dom_ok(h, i, &mut bb)? {
                b = false;
            }
        }
        Ok(b)
    }

    /// `CheckShapes(LS)`: each shape of the list is in the structure;
    /// `c.check_shapes(&l)?`.
    pub fn check_shapes(&self, ls: &List<Shape>) -> Result<bool, Failure> {
        let h = href(&self.my_hds)?;
        Ok(ls.iter().all(|s| h.shape_index(s, true) != 0))
    }

    /// The vertices of the structure compared with each other and with the points.
    fn vertices_ok(
        &self,
        h: &TopOpeBRepDSHDataStructure,
        vert: &IndexedMap<i32>,
    ) -> Result<bool, Failure> {
        let mut b = true;
        let nb_ve = vert.extent();
        for i in 1..=nb_ve {
            let s1 = h.shape(*vert.find_key(i)?, true);
            let sdr1 = h.same_domain_reference(&s1);
            let (tol1, p1) = (
                TopOpeBRepToolShapeTool::tolerance(&s1)?,
                TopOpeBRepToolShapeTool::pnt(&s1)?,
            );
            for j in i + 1..=nb_ve {
                let s2 = h.shape(*vert.find_key(j)?, true);
                let sdr2 = h.same_domain_reference(&s2);
                let tol2 = TopOpeBRepToolShapeTool::tolerance(&s2)?;
                let dist = p1.distance(&TopOpeBRepToolShapeTool::pnt(&s2)?);
                if (dist <= tol1 + tol2 && sdr1 != sdr2) || (dist > tol1 + tol2 && sdr1 == sdr2) {
                    b = false;
                }
            }
            let ds = h.ds();
            let mut pe = TopOpeBRepDSPointExplorer::new(&ds, true);
            while pe.more() {
                let ds_pnt = pe.point()?;
                if ds_pnt.point().distance(&p1) <= ds_pnt.tolerance() + tol1 {
                    b = false;
                }
                pe.next();
            }
        }
        Ok(b)
    }

    /// `OneVertexOnPnt()`: the vertices of the structure are same domain exactly when
    /// they are within their tolerances, and on no point; `c.one_vertex_on_pnt()?`.
    pub fn one_vertex_on_pnt(&self) -> Result<bool, Failure> {
        let h = href(&self.my_hds)?;
        let mut vert: IndexedMap<i32> = IndexedMap::default();
        for i in 1..=h.nb_shapes() {
            let s = h.shape(i, true);
            if s.shape_type()? == ShapeEnum::Vertex && h.has_shape(&s, true) {
                vert.add(i);
            }
        }
        // natively the points are then compared with each other without effect
        self.vertices_ok(h, &vert)
    }

    /// The statuses of one kind printed (`PrintMap`).
    fn print_map(map: &StatusMap, eltstr: &str, os: &mut impl Write) -> io::Result<()> {
        if map.extent() == 0 {
            return Ok(());
        }
        let mut b = true;
        write!(os, " {eltstr}\t(/{})\tnumber ", map.extent())?;
        Self::print_elts(map, TopOpeBRepDSCheckStatus::Ok, &mut b, os)?;
        if !b {
            writeln!(os, " = OK")?;
        }
        Self::print_elts(map, TopOpeBRepDSCheckStatus::Nok, &mut b, os)?;
        if !b {
            writeln!(os, " = NOK")?;
        }
        Ok(())
    }

    /// The indices of one status printed (`PrintElts`).
    fn print_elts(
        map: &StatusMap,
        stat: TopOpeBRepDSCheckStatus,
        b: &mut bool,
        os: &mut impl Write,
    ) -> io::Result<()> {
        let mut bb = !*b;
        *b = true;
        for (i, s) in map.items() {
            if *s != stat {
                continue;
            }
            *b = false;
            if bb {
                write!(os, "\t\tnumber ")?;
                bb = false;
            }
            write!(os, "{i} ")?;
        }
        Ok(())
    }

    /// `PrintIntg(S)`: the statuses of the checked elements by kind;
    /// `c.print_intg(&mut out)?`.
    pub fn print_intg(&self, os: &mut impl Write) -> Result<(), Failure> {
        let io = |e: io::Error| Failure::new(&e.to_string());
        write!(os, "\n\n************************************************\n").map_err(io)?;
        write!(os, "state of the DS : (only the tested elements)\n\n").map_err(io)?;
        Self::print_map(&self.my_map_surface_status, "Surface", os).map_err(io)?;
        Self::print_map(&self.my_map_curve_status, "Curve", os).map_err(io)?;
        Self::print_map(&self.my_map_point_status, "Point", os).map_err(io)?;
        let h = href(&self.my_hds)?;
        let mut maps: [StatusMap; 5] = Default::default();
        let types = [
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            ShapeEnum::Wire,
            ShapeEnum::Face,
            ShapeEnum::Solid,
        ];
        for (i, st) in self.my_map_shape_status.items() {
            let t = h.shape(*i, true).shape_type()?;
            if let Some(k) = types.iter().position(|x| *x == t) {
                maps[k].bind(*i, *st);
            }
        }
        for (m, name) in maps.iter().zip(["Vertex", "Edge", "Wire", "Face", "Solid"]) {
            Self::print_map(m, name, os).map_err(io)?;
        }
        write!(os, "\n\n").map_err(io)
    }

    /// `Print(stat, S)`: OK or NOK; `TopOpeBRepDSCheck::print(TopOpeBRepDSCheckStatus::Ok, &mut out)?`.
    pub fn print(stat: TopOpeBRepDSCheckStatus, os: &mut impl Write) -> io::Result<()> {
        match stat {
            TopOpeBRepDSCheckStatus::Ok => write!(os, "OK"),
            TopOpeBRepDSCheckStatus::Nok => write!(os, "NOK"),
        }
    }

    /// `PrintShape(SE, S)`: the padded name of the shape type;
    /// `TopOpeBRepDSCheck::print_shape_type(ShapeEnum::Face, &mut out)?`.
    pub fn print_shape_type(se: ShapeEnum, os: &mut impl Write) -> io::Result<()> {
        let name = match se {
            ShapeEnum::Solid => "Solid ",
            ShapeEnum::Shell => "Shell ",
            ShapeEnum::Face => "Face  ",
            ShapeEnum::Wire => "Wire  ",
            ShapeEnum::Edge => "Edge  ",
            ShapeEnum::Vertex => "Vertex",
            _ => "",
        };
        write!(os, "{name}")
    }

    /// `PrintShape(index, S)`: a mark for an out of range or unchecked shape, else a
    /// blank; `c.print_shape_index(1, &mut out)?`.
    pub fn print_shape_index(&self, index: i32, os: &mut impl Write) -> Result<(), Failure> {
        let io = |e: io::Error| Failure::new(&e.to_string());
        let h = href(&self.my_hds)?;
        if h.nb_shapes() < index {
            return write!(os, "**PB**IN**TopOpeBRepDS_Check::PrintShape** ").map_err(io);
        }
        if !self.my_map_shape_status.is_bound(&index) {
            return writeln!(os, "NO CHECK HAS PROCESSING").map_err(io);
        }
        write!(os, " ").map_err(io)
    }

    /// `HDS()`; `c.hds()`.
    pub fn hds(&self) -> &Handle<TopOpeBRepDSHDataStructure> {
        &self.my_hds
    }

    /// `ChangeHDS()`; `*c.change_hds() = hds.clone()`.
    pub fn change_hds(&mut self) -> &mut Handle<TopOpeBRepDSHDataStructure> {
        &mut self.my_hds
    }
}
