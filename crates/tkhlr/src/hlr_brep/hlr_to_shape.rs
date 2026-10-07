//! HLRBRep_HLRToShape.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{
    Algo, Data, EdgeData, EdgeDataArray, FaceIterator, TypeOfResultingEdge, make_edge, make_edge_3d,
};
use crate::hlr_algo::EdgeIterator;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// HLRBRep_HLRToShape: the results of a hidden line algorithm as compounds of 2D
/// edges in the view plane (or of 3D edges): the visible (`V`) or hidden (`H`)
/// parts of the sharp edges, smooth edges (`Rg1Line`), sewn edges (`RgNLine`),
/// outlines and isolines, of every shape or of one.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkhlr::hlr_brep::{Algo, HLRToShape};
/// let to_shape = HLRToShape::new(&Handle::new(Algo::new()?));
/// assert!(to_shape.v_compound()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct HLRToShape {
    algo: Handle<Algo>,
}

// The compound being built and whether an edge was added (`added`).
struct Result3 {
    result: Shape,
    added: bool,
}

macro_rules! compounds {
    ($($all:ident, $one:ident, $typ:expr, $visible:expr, $name:literal;)*) => {$(
        #[doc = concat!("`", $name, "()`; `to_shape.", stringify!($all), "()?`.")]
        pub fn $all(&self) -> Result<Shape, Failure> {
            self.internal_compound($typ, $visible, &Shape::default(), false)
        }
        #[doc = concat!("`", $name, "(S)`: of the shape `s`; `to_shape.", stringify!($one), "(&s)?`.")]
        pub fn $one(&self, s: &Shape) -> Result<Shape, Failure> {
            self.internal_compound($typ, $visible, s, false)
        }
    )*};
}

impl HLRToShape {
    /// HLRBRep_HLRToShape(A); `HLRToShape::new(&algo)`.
    pub fn new(a: &Handle<Algo>) -> Self {
        Self { algo: a.clone() }
    }

    compounds! {
        v_compound, v_compound_of, 5, true, "VCompound";
        rg1_line_v_compound, rg1_line_v_compound_of, 3, true, "Rg1LineVCompound";
        rgn_line_v_compound, rgn_line_v_compound_of, 4, true, "RgNLineVCompound";
        out_line_v_compound, out_line_v_compound_of, 2, true, "OutLineVCompound";
        iso_line_v_compound, iso_line_v_compound_of, 1, true, "IsoLineVCompound";
        h_compound, h_compound_of, 5, false, "HCompound";
        rg1_line_h_compound, rg1_line_h_compound_of, 3, false, "Rg1LineHCompound";
        rgn_line_h_compound, rgn_line_h_compound_of, 4, false, "RgNLineHCompound";
        out_line_h_compound, out_line_h_compound_of, 2, false, "OutLineHCompound";
        iso_line_h_compound, iso_line_h_compound_of, 1, false, "IsoLineHCompound";
    }

    /// OutLineVCompound3d(): the visible outlines as 3D edges;
    /// `to_shape.out_line_v_compound_3d()?`.
    pub fn out_line_v_compound_3d(&self) -> Result<Shape, Failure> {
        self.internal_compound(2, true, &Shape::default(), true)
    }

    /// CompoundOfEdges(type, visible, In3d); `to_shape.compound_of_edges(TypeOfResultingEdge::Sharp, true, false)?`.
    pub fn compound_of_edges(
        &self,
        ty: TypeOfResultingEdge,
        visible: bool,
        in_3d: bool,
    ) -> Result<Shape, Failure> {
        self.internal_compound(ty as i32, visible, &Shape::default(), in_3d)
    }

    /// CompoundOfEdges(S, type, visible, In3d): of the shape `s`;
    /// `to_shape.compound_of_edges_of(&s, TypeOfResultingEdge::Sharp, true, false)?`.
    pub fn compound_of_edges_of(
        &self,
        s: &Shape,
        ty: TypeOfResultingEdge,
        visible: bool,
        in_3d: bool,
    ) -> Result<Shape, Failure> {
        self.internal_compound(ty as i32, visible, s, in_3d)
    }

    // InternalCompound(typ, visible, S, In3d): with the projector scaled, the edges of
    // the type drawn face by face then (types 3 and over) the free ones, of every
    // shape, of the shape `s`, or of the faces and edges of `s` when it is not one of
    // the shapes; a null shape when nothing is drawn.
    fn internal_compound(
        &self,
        typ: i32,
        visible: bool,
        s: &Shape,
        in_3d: bool,
    ) -> Result<Shape, Failure> {
        let algo = self.algo.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "HLRBRep_HLRToShape: no algorithm")
        })?;
        let ds = algo.data_structure();
        let mut out = Result3 {
            result: Shape::default(),
            added: false,
        };
        if let Some(data) = ds.get() {
            data.change_projector(|p| p.scaled(true))?;
            // A failure leaves the projector scaled, as the native exception does.
            self.draw_all(algo, data, (typ, visible, in_3d), s, &mut out)?;
            data.change_projector(|p| p.scaled(false))?;
        }
        if !out.added {
            return Ok(Shape::default());
        }
        Ok(out.result)
    }

    fn draw_all(
        &self,
        algo: &Algo,
        data: &Data,
        mode: (i32, bool, bool),
        s: &Shape,
        out: &mut Result3,
    ) -> Result<(), Failure> {
        let (mut e1, mut e2, mut f1, mut f2) = (1, data.nb_edges(), 1, data.nb_faces());
        let mut explor = false;
        if !s.is_null() {
            let index = algo.index(s);
            if index == 0 {
                explor = true;
            } else {
                (_, _, e1, e2, f1, f2) = algo.shape_bounds(index)?.bounds();
            }
        }
        let mut c = Compound::default();
        BRepBuilder::default().make_compound(&mut c);
        out.result = Shape::from(c);
        mark_used(data, e1, e2)?;
        if explor {
            return draw_explored(data, mode, s, out);
        }
        for iface in f1..=f2 {
            draw_face(data, mode, iface, out)?;
        }
        if mode.0 >= 3 {
            let mut ed = data.e_data_array();
            for ie in e1..=e2 {
                draw_free_edge(&mut ed, ie, mode, out)?;
            }
        }
        Ok(())
    }
}

fn edge_mut(ed: &mut EdgeDataArray, i: i32) -> Result<&mut EdgeData, Failure> {
    ed.change_value(i)
        .map_err(|x| x.into_failure("HLRBRep_HLRToShape"))
}

// The selected edges that are not vertical are to draw (unused, hide count 0).
fn mark_used(data: &Data, e1: i32, e2: i32) -> Result<(), Failure> {
    let mut ed = data.e_data_array();
    for ie in e1..=e2 {
        let e = edge_mut(&mut ed, ie)?;
        if e.selected() && !e.vertical() {
            e.set_used(false);
            e.set_hide_count(0);
        } else {
            e.set_used(true);
        }
    }
    Ok(())
}

// The faces of `s`, then (types 3 and over) its edges outside faces, that are in
// the data structure.
fn draw_explored(
    data: &Data,
    mode: (i32, bool, bool),
    s: &Shape,
    out: &mut Result3,
) -> Result<(), Failure> {
    let faces: Vec<i32> = {
        let fm = data.face_map();
        Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)?
            .map(|f| fm.find_index(&f) as i32)
            .collect()
    };
    for iface in faces.into_iter().filter(|i| *i != 0) {
        draw_face(data, mode, iface, out)?;
    }
    if mode.0 < 3 {
        return Ok(());
    }
    let edges: Vec<i32> = {
        let em = data.edge_map();
        Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Face)?
            .map(|e| em.find_index(&e) as i32)
            .collect()
    };
    let mut ed = data.e_data_array();
    for ie in edges.into_iter().filter(|i| *i != 0) {
        draw_free_edge(&mut ed, ie, mode, out)?;
    }
    Ok(())
}

fn draw_free_edge(
    ed: &mut EdgeDataArray,
    ie: i32,
    (typ, visible, in_3d): (i32, bool, bool),
    out: &mut Result3,
) -> Result<(), Failure> {
    let e = edge_mut(ed, ie)?;
    if !e.used() {
        draw_edge(visible, false, typ, e, out, in_3d)?;
        e.set_used(true);
    }
    Ok(())
}

// Whether the face edge is drawn for the type (an outline in 3D, a smooth edge that
// is not an outline...).
fn face_edge_to_draw(
    typ: i32,
    in_3d: bool,
    itf: &FaceIterator,
    edf: &EdgeData,
) -> Result<bool, Failure> {
    Ok(match typ {
        1 => itf.iso_line()?,
        2 if in_3d => itf.internal()? || itf.out_line()?,
        2 => itf.internal()?,
        3 => edf.rg1_line() && !edf.rgn_line() && !itf.out_line()?,
        4 => edf.rgn_line() && !itf.out_line()?,
        _ => !itf.iso_line()? && !itf.internal()? && (!edf.rg1_line() || itf.out_line()?),
    })
}

// DrawFace(visible, typ, iface, DS, Result, added, In3d): the unused edges of the
// face of the type; a smooth edge refused for sharp edges or outlines is tried
// with one more face before it is used.
fn draw_face(
    data: &Data,
    (typ, visible, in_3d): (i32, bool, bool),
    iface: i32,
    out: &mut Result3,
) -> Result<(), Failure> {
    let mut itf = FaceIterator::new();
    {
        let fd = data.f_data_array();
        itf.init_edge(
            fd.value(iface)
                .map_err(|x| x.into_failure("HLRBRep_HLRToShape::DrawFace"))?,
        )?;
    }
    let mut ed = data.e_data_array();
    while itf.more_edge() {
        let edf = edge_mut(&mut ed, itf.edge()?)?;
        if !edf.used() {
            if face_edge_to_draw(typ, in_3d, &itf, edf)? {
                draw_edge(visible, true, typ, edf, out, in_3d)?;
                edf.set_used(true);
            } else if (typ > 4 || typ == 2) && edf.rg1_line() && !itf.out_line()? {
                let hc = edf.hide_count();
                if hc > 0 {
                    edf.set_used(true);
                } else {
                    edf.set_hide_count(hc + 1); // to try with another face
                }
            } else {
                edf.set_used(true);
            }
        }
        itf.next_edge()?;
    }
    Ok(())
}

// DrawEdge(visible, inFace, typ, ed, Result, added, In3d): the visible or hidden
// parts of the edge, as 2D or 3D edges.
fn draw_edge(
    visible: bool,
    in_face: bool,
    typ: i32,
    ed: &EdgeData,
    out: &mut Result3,
    in_3d: bool,
) -> Result<(), Failure> {
    let todraw = if in_face {
        true
    } else if typ == 3 {
        ed.rg1_line() && !ed.rgn_line()
    } else if typ == 4 {
        ed.rgn_line()
    } else {
        !ed.rg1_line()
    };
    if !todraw {
        return Ok(());
    }
    let mut parts = Vec::new();
    let mut it = EdgeIterator::new();
    if visible {
        it.init_visible(ed.status());
        while it.more_visible() {
            parts.push(it.visible()?);
            it.next_visible();
        }
    } else {
        it.init_hidden(ed.status())?;
        while it.more_hidden() {
            parts.push(it.hidden());
            it.next_hidden()?;
        }
    }
    let b = BRepBuilder::default();
    for (sta, _, end, _) in parts {
        let e = if in_3d {
            make_edge_3d(ed.geometry(), sta, end)?
        } else {
            make_edge(ed.geometry(), sta, end)?
        };
        if !e.is_null() {
            b.add(&mut out.result, &Shape::from(e))?;
            out.added = true;
        }
    }
    Ok(())
}
