//! HLRBRep_PolyHLRToShape.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{BiPnt2D, BiPntFlags, PolyAlgo, PolyAlgoSegment};
use crate::hlr_algo::{EdgeIterator, EdgeStatus};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge2d;

/// HLRBRep_PolyHLRToShape: the results of a polyhedral hidden line algorithm as
/// compounds of 2D edges: `update` keeps the visible and hidden parts of every
/// segment (hide mode) or, after `show`, the segments are drawn whole.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkhlr::hlr_brep::{PolyAlgo, PolyHLRToShape};
/// let algo = Handle::new(PolyAlgo::new()?);
/// algo.get().unwrap().update()?;
/// let mut to_shape = PolyHLRToShape::new();
/// to_shape.update(&algo)?;
/// assert!(to_shape.v_compound()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct PolyHLRToShape {
    algo: Handle<PolyAlgo>,
    bi_pnt_vis: Vec<BiPnt2D>,
    bi_pnt_hid: Vec<BiPnt2D>,
    // Natively uninitialized until Update; false here.
    hide_mode: bool,
}

fn null_algo() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "HLRBRep_PolyHLRToShape: no algorithm; expected Update",
    )
}

macro_rules! compounds {
    ($($all:ident, $one:ident, $typ:expr, $visible:expr, $name:literal;)*) => {$(
        #[doc = concat!("`", $name, "()`; `to_shape.", stringify!($all), "()?`.")]
        pub fn $all(&self) -> Result<Shape, Failure> {
            self.internal_compound($typ, $visible, &Shape::default())
        }
        #[doc = concat!("`", $name, "(S)`: of the shape `s`; `to_shape.", stringify!($one), "(&s)?`.")]
        pub fn $one(&self, s: &Shape) -> Result<Shape, Failure> {
            self.internal_compound($typ, $visible, s)
        }
    )*};
}

fn flags_of(seg: &PolyAlgoSegment) -> BiPntFlags {
    BiPntFlags {
        reg1: seg.reg1,
        regn: seg.regn,
        outl: seg.outl,
        intl: seg.intl,
    }
}

// Whether a segment of the given flags is drawn for the type (1 outlines, 2 smooth
// edges, 3 sewn edges, 4 sharp edges).
fn to_draw(typ: i32, f: BiPntFlags) -> bool {
    match typ {
        1 => f.intl,
        2 => f.reg1 && !f.regn && !f.outl,
        3 => f.regn && !f.outl,
        _ => !f.intl && (!f.reg1 || f.outl),
    }
}

impl PolyHLRToShape {
    /// HLRBRep_PolyHLRToShape(); `PolyHLRToShape::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update(A): the visible and hidden parts in the view plane of every segment of
    /// the algorithm (hide mode); `to_shape.update(&algo)?`.
    pub fn update(&mut self, a: &Handle<PolyAlgo>) -> Result<(), Failure> {
        self.algo = a.clone();
        self.hide_mode = true;
        self.bi_pnt_vis.clear();
        self.bi_pnt_hid.clear();
        let algo = self.algo.get().ok_or_else(null_algo)?;
        let t = algo.projector().transformation();
        let mut status = EdgeStatus::new();
        algo.init_hide()?;
        while algo.more_hide() {
            let seg = algo.hide(&mut status)?;
            let (mut sta, mut end) = (seg.points.pnt1, seg.points.pnt2);
            t.transforms(&mut sta);
            t.transforms(&mut end);
            let sta2d = Xy::new(sta.x(), sta.y());
            let end2d = Xy::new(end.x(), end.y());
            let d = end2d - sta2d;
            if d.modulus() > 1.0e-10 {
                let flags = flags_of(&seg);
                let mut it = EdgeIterator::new();
                it.init_visible(&status);
                while it.more_visible() {
                    let (a, _, b, _) = it.visible()?;
                    self.bi_pnt_vis.push(BiPnt2D::from_xy(
                        sta2d + a * d,
                        sta2d + b * d,
                        &seg.shape,
                        flags,
                    ));
                    it.next_visible();
                }
                it.init_hidden(&status)?;
                while it.more_hidden() {
                    let (a, _, b, _) = it.hidden();
                    self.bi_pnt_hid.push(BiPnt2D::from_xy(
                        sta2d + a * d,
                        sta2d + b * d,
                        &seg.shape,
                        flags,
                    ));
                    it.next_hidden()?;
                }
            }
            algo.next_hide()?;
        }
        Ok(())
    }

    /// Show(): the segments are drawn whole; `to_shape.show()`.
    pub fn show(&mut self) {
        self.hide_mode = false;
    }

    /// Hide(): the visible or hidden parts are drawn; `to_shape.hide()`.
    pub fn hide(&mut self) {
        self.hide_mode = true;
    }

    compounds! {
        v_compound, v_compound_of, 4, true, "VCompound";
        rg1_line_v_compound, rg1_line_v_compound_of, 2, true, "Rg1LineVCompound";
        rgn_line_v_compound, rgn_line_v_compound_of, 3, true, "RgNLineVCompound";
        out_line_v_compound, out_line_v_compound_of, 1, true, "OutLineVCompound";
        h_compound, h_compound_of, 4, false, "HCompound";
        rg1_line_h_compound, rg1_line_h_compound_of, 2, false, "Rg1LineHCompound";
        rgn_line_h_compound, rgn_line_h_compound_of, 3, false, "RgNLineHCompound";
        out_line_h_compound, out_line_h_compound_of, 1, false, "OutLineHCompound";
    }

    // InternalCompound(typ, visible, S): the 2D edges of the segments of the type (of
    // the faces and edges of `s` when given); a null shape when there is none.
    fn internal_compound(&self, typ: i32, visible: bool, s: &Shape) -> Result<Shape, Failure> {
        let mut map = MapOfShape::default();
        if !s.is_null() {
            for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
                map.add(e);
            }
            for f in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
                map.add(f);
            }
        }
        let mut c = Compound::default();
        let b = BRepBuilder::default();
        b.make_compound(&mut c);
        let mut result = Shape::from(c);
        let mut added = false;
        let mut add = |p1: Point2, p2: Point2| -> Result<(), Failure> {
            b.add(
                &mut result,
                BRepLibMakeEdge2d::from_points(p1, p2)?.shape()?,
            )?;
            added = true;
            Ok(())
        };
        if self.hide_mode {
            let list = if visible {
                &self.bi_pnt_vis
            } else {
                &self.bi_pnt_hid
            };
            for bp in list {
                let f = BiPntFlags {
                    reg1: bp.rg1_line(),
                    regn: bp.rgn_line(),
                    outl: bp.out_line(),
                    intl: bp.int_line(),
                };
                if to_draw(typ, f)
                    && (s.is_null() || map.contains(bp.shape()))
                    && bp.p1().square_distance(bp.p2()) > 1.0e-20
                {
                    add(bp.p1(), bp.p2())?;
                }
            }
        } else {
            let algo = self.algo.get().ok_or_else(null_algo)?;
            let t = algo.projector().transformation();
            algo.init_show()?;
            while algo.more_show() {
                let seg = algo.show()?;
                if to_draw(typ, flags_of(&seg)) && (s.is_null() || map.contains(&seg.shape)) {
                    let (mut sta, mut end) = (seg.points.pnt1, seg.points.pnt2);
                    t.transforms(&mut sta);
                    t.transforms(&mut end);
                    let sta2d = Xy::new(sta.x(), sta.y());
                    let end2d = Xy::new(end.x(), end.y());
                    if (end2d - sta2d).square_modulus() > 1.0e-20 {
                        add(Point2::from(sta2d), Point2::from(end2d))?;
                    }
                }
                algo.next_show()?;
            }
        }
        if !added {
            return Ok(Shape::default());
        }
        Ok(result)
    }
}
