//! The faces of the general builder (TopOpeBRepBuild_makefaces.cxx): the faces of a
//! wire edge set with their unclosed wires corrected and closing edges purged, and the
//! faces of a face builder with their INTERNAL/EXTERNAL edges reoriented.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::{
    ShapeToShapeMap, TopOpeBRepBuildFaceBuilder, TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_tool::{
    correct_on_uviso, fun_tool_closed_w, make_faces as tool_make_faces, purge_closing_edges,
};
use crate::top_ope_brep_tool::{
    fc2d_curve_on_surface, fc2d_has_curve_on_surface, fun_tool_closed_s_flag,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;

/// `FUN_purgeFon1nonoriE(newFace)` (static): a face of one edge, INTERNAL, EXTERNAL or
/// degenerated.
fn fun_purge_fon1_nonori_e(new_face: &Shape) -> Result<bool, Failure> {
    let edges: Vec<Shape> = Explorer::new(new_face, ShapeEnum::Edge, ShapeEnum::Shape)?.collect();
    if edges.len() != 1 {
        return Ok(false);
    }
    let ed = &edges[0];
    if !matches!(
        ed.orientation(),
        Orientation::Forward | Orientation::Reversed
    ) {
        return Ok(true);
    }
    BRepTool::degenerated(&topods::edge(ed)?)
}

/// `FUN_ReOrientIntExtEdge(FRE, OFRE, INE)` (static): the orientation of an INTERNAL or
/// EXTERNAL edge connected to the oriented edge `fre` (its own when not connected).
fn fun_reorient_int_ext_edge(fre: &Edge, ofre: Orientation, ine: &Edge) -> Orientation {
    let mut result = ine.orientation();
    let (vf1, vl1) = topexp::vertices(fre, false);
    let (vf2, vl2) = topexp::vertices(ine, false);
    let (fwd, rev) = match ofre {
        Orientation::Forward => (Orientation::Forward, Orientation::Reversed),
        Orientation::Reversed => (Orientation::Reversed, Orientation::Forward),
        _ => return result,
    };
    if vl1.is_same(vf2.as_ref()) {
        result = fwd;
    }
    if vl1.is_same(vl2.as_ref()) {
        result = rev;
    }
    if vf1.is_same(vf2.as_ref()) {
        result = rev;
    }
    if vf1.is_same(vl2.as_ref()) {
        result = fwd;
    }
    result
}

/// `FUN_CheckORI(O1, O2)` (static): 0 both INTERNAL/EXTERNAL, 1 the first only, 2 the
/// second only, 4 none.
fn fun_check_ori(o1: Orientation, o2: Orientation) -> i32 {
    let ie = |o: Orientation| o == Orientation::Internal || o == Orientation::External;
    match (ie(o1), ie(o2)) {
        (true, true) => 0,
        (true, false) => 1,
        (false, true) => 2,
        _ => 4,
    }
}

/// The counts of a new wire: its edges, INTERNAL ones and EXTERNAL ones.
#[derive(Default, Clone, Copy)]
struct WireCounts {
    ne: i32,
    nint: i32,
    next: i32,
}

impl WireCounts {
    /// The count of an INTERNAL or EXTERNAL edge reoriented.
    fn reoriented(&mut self, o: Orientation) {
        if o == Orientation::Internal {
            self.nint -= 1;
        } else {
            self.next -= 1;
        }
    }
}

impl TopOpeBRepBuildBuilder {
    /// `GWESMakeFaces(FF, WES, LOF)` (virtual): the faces of the wire edge set (classes
    /// forced) with their unclosed wires corrected through the same domain vertices,
    /// their closing edges purged, corrected on the UV isos of a closed face and
    /// regularized; `b.gwes_make_faces(&f, &mut wes, &mut lof)?`.
    pub fn gwes_make_faces(
        &mut self,
        ff: &Shape,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
        lof: &mut List<Shape>,
    ) -> Result<(), Failure> {
        if self.b1.is_some() {
            return self.b1_gwes_make_faces(ff, wes, lof);
        }
        self.gwes_make_faces_base(ff, wes, lof)
    }

    /// The unclosed wires of the face builder corrected through the same domain
    /// vertices of the vertices of the same geometry (made same domain first).
    fn correct_unclosed_wires(&self, fabu: &mut TopOpeBRepBuildFaceBuilder) -> Result<(), Failure> {
        let mut map_pie: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        fabu.detect_pseudo_internal_edge(&mut map_pie)?;
        let (mut map_vv_same_g, mut map_von1_edge, mut map_vv_ref) = (
            ShapeToShapeMap::default(),
            ShapeToShapeMap::default(),
            ShapeToShapeMap::default(),
        );
        fabu.detect_unclosed_wire(&mut map_vv_same_g, &mut map_von1_edge)?;
        if map_vv_same_g.is_empty() {
            return Ok(());
        }
        let hds = self.hds()?;
        for i in 1..=map_vv_same_g.len() {
            let v = map_vv_same_g.find_key(i)?.clone();
            let mut hsdm = hds.has_same_domain(&v, true);
            if !hsdm {
                let vsame_g = map_vv_same_g.find_from_index(i)?.clone();
                let mut bds = hds.change_ds();
                let (rank_v, rank_vsame_g) = (bds.ancestor_rank(&v), bds.ancestor_rank(&vsame_g));
                if rank_vsame_g != 0 && rank_v != 1 {
                    bds.fill_shapes_same_domain(&vsame_g, &v, true)?;
                } else {
                    bds.fill_shapes_same_domain(&v, &vsame_g, true)?;
                }
                drop(bds);
                hsdm = hds.has_same_domain(&v, true);
            }
            if hsdm {
                map_vv_ref.add(v.clone(), hds.shape(hds.same_domain_reference(&v), true));
            }
        }
        fabu.correct_gclosed_wire(&map_vv_ref, &map_von1_edge)?;
        fabu.detect_unclosed_wire(&mut map_vv_same_g, &mut map_von1_edge)
    }

    /// `TopOpeBRepBuild_Builder::GWESMakeFaces`, the base version.
    pub(crate) fn gwes_make_faces_base(
        &mut self,
        ff: &Shape,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
        lof: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut fabu = TopOpeBRepBuildFaceBuilder::new();
        fabu.init_face_builder(wes, ff, true)?;
        self.correct_unclosed_wires(&mut fabu)?;
        let mut mwisold = DataMap::default();
        let mut msh_nok: IndexedMap<Shape> = IndexedMap::default();
        self.gfabu_make_faces(ff, &mut fabu, lof, &mut mwisold)?;
        let fa = topods::face(ff)?;
        if !purge_closing_edges(&fa, lof, &mwisold, &mut msh_nok)? {
            return Err(Failure::with_type(
                "Standard_Failure",
                "TopOpeBRepBuild::GWESMakeFaces",
                "TopOpeBRepBuild_Builder",
            ));
        }
        if !msh_nok.is_empty() {
            let mut loff = List::default();
            if !tool_make_faces(&fa, lof, &msh_nok, &mut loff)? {
                return Err(Failure::with_type(
                    "Standard_Failure",
                    "TopOpeBRepBuild::GWESMakeFaces",
                    "TopOpeBRepBuild_Builder",
                ));
            }
            *lof = loff;
        }
        if fun_tool_closed_s_flag(ff)? {
            let mut new_lof = List::default();
            for f in lof.iter() {
                let mut fa2 = topods::face(f)?;
                correct_on_uviso(&fa, &mut fa2)?;
                new_lof.append(fa2.into_shape());
            }
            *lof = new_lof;
        }
        let mut new_lof = List::default();
        self.regularize_faces(ff, lof, &mut new_lof)?;
        *lof = new_lof;
        Ok(())
    }

    /// The edge of the face builder given a p-curve on the new face when it has none.
    fn ensure_pcurve(new_edge: &Edge, new_face: &Face) -> Result<(), Failure> {
        if fc2d_has_curve_on_surface(new_edge, new_face)? {
            return Ok(());
        }
        let tol_e = BRepTool::edge_tolerance(new_edge)?;
        let (c2d, _, _, tolpc) = fc2d_curve_on_surface(new_edge, new_face, true)?;
        if c2d.is_null() {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "TopOpeBRepBuild_Builder::GFABUMakeFaces null PC",
                "TopOpeBRepBuild_Builder",
            ));
        }
        BRepBuilder::default().update_edge_pcurve_on_face(
            new_edge,
            &c2d,
            new_face,
            tol_e.max(tolpc),
        )
    }

    /// The edges of the current wire of the face builder in a compound, counted.
    fn wire_edges_compound(
        fabu: &mut TopOpeBRepBuildFaceBuilder,
        new_face: &Face,
    ) -> Result<(Shape, WireCounts), Failure> {
        let mut cmp = Compound::default();
        BRepBuilder::default().make_compound(&mut cmp);
        let mut cmp = cmp.into_shape();
        let mut c = WireCounts::default();
        fabu.init_edge()?;
        while fabu.more_edge() {
            let new_edge = topods::edge(fabu.edge()?)?;
            c.ne += fabu.add_edge_wire(new_edge.as_ref(), &mut cmp)?;
            match new_edge.orientation() {
                Orientation::Internal => c.nint += 1,
                Orientation::External => c.next += 1,
                _ => {}
            }
            Self::ensure_pcurve(&new_edge, new_face)?;
            fabu.next_edge()?;
        }
        Ok((cmp, c))
    }

    /// The edges of the compound added to the wire with their INTERNAL/EXTERNAL edges
    /// reoriented along their neighbours (each vertex of two edges), the counts updated.
    fn add_reoriented_edges(
        fabu: &TopOpeBRepBuildFaceBuilder,
        map_voe: &IndexedDataMapOfShapeListOfShape,
        tdf: &Face,
        new_wire: &mut Shape,
        c: &mut WireCounts,
    ) -> Result<(), Failure> {
        let (mut lof_add_e, mut naddsame): (List<Shape>, i32) = (List::default(), 0);
        while c.ne > lof_add_e.len() as i32 + naddsame {
            for step in 1..=map_voe.len() {
                let lofe = map_voe.find_from_index(step)?;
                let mut it = lofe.iter();
                let mut e1 = topods::edge(
                    it.next()
                        .ok_or_else(|| Failure::new("GFABUMakeFaces: vertex without edge"))?,
                )?;
                let mut e2 = topods::edge(
                    it.next()
                        .ok_or_else(|| Failure::new("GFABUMakeFaces: vertex of one edge"))?,
                )?;
                let (mut o1, mut o2) = (e1.orientation(), e2.orientation());
                let (same1, same2) = (
                    BRepTool::is_closed_on_face(&e1, tdf)?,
                    BRepTool::is_closed_on_face(&e2, tdf)?,
                );
                let (mut add1, mut add2) = (true, true);
                for le in lof_add_e.iter() {
                    let ole = le.orientation();
                    if e1.as_ref().is_same(le) && !same1 {
                        (add1, o1) = (false, ole);
                        e1 = topods::edge(&e1.oriented(ole))?;
                    }
                    if e2.as_ref().is_same(le) && !same2 {
                        (add2, o2) = (false, ole);
                        e2 = topods::edge(&e2.oriented(ole))?;
                    }
                }
                match fun_check_ori(o1, o2) {
                    0 => (add1, add2) = (false, false),
                    1 => {
                        let ori = fun_reorient_int_ext_edge(&e2, o2, &e1);
                        if matches!(ori, Orientation::Forward | Orientation::Reversed) {
                            e1 = topods::edge(&e1.oriented(ori))?;
                            c.reoriented(o1);
                        }
                    }
                    2 => {
                        let ori = fun_reorient_int_ext_edge(&e1, o1, &e2);
                        if matches!(ori, Orientation::Forward | Orientation::Reversed) {
                            e2 = topods::edge(&e2.oriented(ori))?;
                            c.reoriented(o2);
                        }
                    }
                    _ => {}
                }
                for (add, e, same) in [(add1, &e1, same1), (add2, &e2, same2)] {
                    if add {
                        fabu.add_edge_wire(e.as_ref(), new_wire)?;
                        if same {
                            naddsame += 1;
                        } else {
                            lof_add_e.append(e.clone().into_shape());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// The new wire of the current wire of the face builder (None when the build stops:
    /// two copies of one edge in the only wire).
    fn gfabu_new_wire(
        &self,
        fabu: &mut TopOpeBRepBuildFaceBuilder,
        new_face: &Face,
        tdf: &Face,
        nbw: i32,
    ) -> Result<Option<(Shape, WireCounts)>, Failure> {
        let mut new_wire = Wire::default().into_shape();
        self.my_build_tool.make_wire(&mut new_wire)?;
        let (cmp, mut c) = Self::wire_edges_compound(fabu, new_face)?;
        let edges: Vec<Shape> = Explorer::new(&cmp, ShapeEnum::Edge, ShapeEnum::Shape)?.collect();
        if (c.nint == 0 && c.next == 0) || c.ne == c.nint || c.ne == c.next {
            if nbw == 1 && c.ne == 2 && edges.len() >= 2 && edges[0].is_same(&edges[1]) {
                return Ok(None);
            }
            for e in &edges {
                fabu.add_edge_wire(e, &mut new_wire)?;
            }
            return Ok(Some((new_wire, c)));
        }
        let mut map_voe = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&cmp, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map_voe)?;
        let closed =
            (1..=map_voe.len()).all(|i| map_voe.find_from_index(i).is_ok_and(|l| l.len() == 2));
        if !closed {
            for e in &edges {
                fabu.add_edge_wire(e, &mut new_wire)?;
            }
        } else {
            Self::add_reoriented_edges(fabu, &map_voe, tdf, &mut new_wire, &mut c)?;
        }
        Ok(Some((new_wire, c)))
    }

    /// The trimmed surface of the face widened to the UV bounds of the new face.
    fn widen_trim(surf: &Handle<dyn GeomSurface>, new_face: &Face) -> Result<(), Failure> {
        let hrts: Handle<GeomRectangularTrimmedSurface> = surf.clone().cast();
        let Some(rts) = hrts
            .get()
            .filter(|_| surf.is_instance_name("Geom_RectangularTrimmedSurface"))
        else {
            return Ok(());
        };
        let (mut oumin, mut oumax, mut ovmin, mut ovmax) = rts.bounds();
        let (umin, umax, vmin, vmax) = uv_bounds(new_face)?;
        oumin = oumin.min(umin);
        oumax = oumax.max(umax);
        ovmin = ovmin.min(vmin);
        ovmax = ovmax.max(vmax);
        rts.set_trim(oumin, oumax, ovmin, ovmax, true, true)
    }

    /// `GFABUMakeFaces(FF, FABU, LOF, MWisOld)`: per face of the builder, a face on the
    /// surface of `ff` with its old wires and its new wires (INTERNAL/EXTERNAL edges
    /// reoriented, closed when so), but faces of one non-oriented edge; the wires
    /// remembered old (1) or new (0); `b.gfabu_make_faces(&f, &mut fb, &mut lof, &mut m)?`.
    pub fn gfabu_make_faces(
        &mut self,
        ff: &Shape,
        fabu: &mut TopOpeBRepBuildFaceBuilder,
        lof: &mut List<Shape>,
        mwisold: &mut DataMap<Shape, i32, ShapeMapHasher>,
    ) -> Result<(), Failure> {
        let tdf = topods::face(ff)?;
        let (surf, loc): (Handle<dyn GeomSurface>, Location) =
            BRepTool::surface_with_location(&tdf)?;
        let tol_ff = BRepTool::face_tolerance(&tdf)?;
        let bb = BRepBuilder::default();
        let mut lnew_face = List::default();
        fabu.init_face();
        while fabu.more_face() {
            let (mut nbnew, mut nbold) = (0, 0);
            let mut new_face = Face::default();
            bb.make_face_on_located_surface(&mut new_face, &surf, &loc, tol_ff)?;
            let nbw = fabu.init_wire();
            while fabu.more_wire() {
                let isold = fabu.is_old_wire()?;
                let (mut new_wire, c) = if isold {
                    nbold += 1;
                    (fabu.old_wire()?, WireCounts::default())
                } else {
                    match self.gfabu_new_wire(fabu, &new_face, &tdf, nbw)? {
                        Some(r) => r,
                        None => return Ok(()),
                    }
                };
                if c.ne != 0 {
                    mwisold.bind(new_wire.clone(), i32::from(isold));
                }
                if !isold {
                    if c.ne == 0 || (nbw == 1 && c.ne == c.nint + c.next) {
                        fabu.next_wire();
                        continue;
                    }
                    nbnew += 1;
                    let closed = fun_tool_closed_w(&topods::wire(&new_wire)?)?;
                    self.my_build_tool.closed(&mut new_wire, closed)?;
                }
                let mut nf = new_face.clone().into_shape();
                self.my_build_tool.add_face_wire(&mut nf, &new_wire)?;
                fabu.next_wire();
            }
            if (nbnew != 0 || nbold != 0) && !fun_purge_fon1_nonori_e(new_face.as_ref())? {
                Self::widen_trim(&surf, &new_face)?;
                lnew_face.append(new_face.into_shape());
            }
            fabu.next_face();
        }
        lof.append_list(&mut lnew_face);
        Ok(())
    }
}
