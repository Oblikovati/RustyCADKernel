// Rust translation of Draw/TKDraw/DBRep/DBRep_DrawableShape.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DBRep_DrawableShape`: a shape drawn as its face isos, edges and free vertices (or
//! as its hidden-line removal).
mod edges;
mod faces;
mod hidden;
mod normals;
mod persistence;
mod plot;

pub use normals::{MeshNormals, NormalSegment};
pub use plot::{LastShapePick, last_pick};

use super::edge_face::{DBRepEdge, DBRepFace};
use super::hide_data::DBRepHideData;
use super::iso_builder::DBRepIsoBuilder;
use crate::draw::{
    DrawColor, DrawColorKind, DrawConsole, DrawDisplay, DrawInterpretor, DrawOStream, Drawable3D,
    Drawable3DBase, DrawableHandle,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{Explorer, map_shapes_and_ancestors};
use rustycad_tkbrep::topods::{Shape, edge as to_edge, face as to_face};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// The colors of a drawable shape: free edges (one face), connected edges (two faces
/// or more, and vertices), isolated edges (no face), isos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeColors {
    /// `FreeCol`.
    pub free: DrawColor,
    /// `ConnCol`.
    pub connected: DrawColor,
    /// `EdgeCol`.
    pub edge: DrawColor,
    /// `IsosCol`.
    pub isos: DrawColor,
}

impl ShapeColors {
    /// The colors DBRep gives every shape: green free edges, yellow connected edges,
    /// red isolated edges, blue isos.
    pub fn standard() -> Self {
        Self {
            free: DrawColor::new(DrawColorKind::Vert),
            connected: DrawColor::new(DrawColorKind::Jaune),
            edge: DrawColor::new(DrawColorKind::Rouge),
            isos: DrawColor::new(DrawColorKind::Bleu),
        }
    }
}

/// The hidden-line settings of a drawable shape.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HlrSettings {
    /// `withHLR`: draw the hidden-line removal instead of the wireframe.
    pub with_hlr: bool,
    /// `withRg1`: keep the smooth edges.
    pub with_rg1: bool,
    /// `withRgN`: keep the seam-like edges.
    pub with_rgn: bool,
    /// `withHid`: draw the hidden parts too.
    pub with_hid: bool,
    /// `ang`: the discretisation angle.
    pub angle: f64,
}

/// `DBRep_DrawableShape`: a shape with its display settings and the display records
/// (faces with isos, colored edges, per-view hidden lines) built on first drawing.
pub struct DBRepDrawableShape {
    base: Drawable3DBase,
    shape: Shape,
    edges: RefCell<Vec<DBRepEdge>>,
    faces: RefCell<Vec<DBRepFace>>,
    hidden: RefCell<Vec<DBRepHideData>>,
    size: f64,
    discret: Cell<i32>,
    colors: ShapeColors,
    nb_isos: Cell<i32>,
    display_orientation: Cell<bool>,
    triangulations: Cell<bool>,
    polygons: Cell<bool>,
    hlr: Cell<HlrSettings>,
}

impl DBRepDrawableShape {
    /// `DBRep_DrawableShape(C, FreeCol, ConnCol, EdgeCol, IsosCol, size, nbisos,
    /// discret)`: no orientation, triangulation, polygon or hidden-line display (the
    /// angle is 0 until `DisplayHLR`, where C++ leaves it uninitialized).
    ///
    /// ```
    /// use rustycad_tkbrep::topods::Shape;
    /// use rustycad_tkdraw::dbrep::{DBRepDrawableShape, ShapeColors};
    /// let drawable = DBRepDrawableShape::new(Shape::default(), ShapeColors::standard(), 100.0, 2, 30);
    /// assert_eq!((drawable.nb_isos(), drawable.discret()), (2, 30));
    /// ```
    pub fn new(shape: Shape, colors: ShapeColors, size: f64, nb_isos: i32, discret: i32) -> Self {
        Self {
            base: Drawable3DBase::new(),
            shape,
            edges: RefCell::new(Vec::new()),
            faces: RefCell::new(Vec::new()),
            hidden: RefCell::new(Vec::new()),
            size,
            discret: Cell::new(discret),
            colors,
            nb_isos: Cell::new(nb_isos),
            display_orientation: Cell::new(false),
            triangulations: Cell::new(false),
            polygons: Cell::new(false),
            hlr: Cell::new(HlrSettings::default()),
        }
    }

    /// `Shape`.
    pub fn shape(&self) -> Shape {
        self.shape.clone()
    }

    /// `NbIsos`.
    pub fn nb_isos(&self) -> i32 {
        self.nb_isos.get()
    }

    /// `ChangeNbIsos`: rebuilds the face records with `nb_isos` isos per direction;
    /// `out` receives the iso builder reports.
    pub fn change_nb_isos(&self, nb_isos: i32, out: &dyn DrawConsole) -> Result<(), Failure> {
        self.nb_isos.set(nb_isos);
        *self.faces.borrow_mut() = self.face_records(out)?;
        Ok(())
    }

    /// `Discret`.
    pub fn discret(&self) -> i32 {
        self.discret.get()
    }

    /// `ChangeDiscret`.
    pub fn change_discret(&self, discret: i32) {
        self.discret.set(discret);
    }

    /// `DisplayOrientation`: color the edges and vertices by orientation and mark the
    /// edge ends with arrows.
    pub fn display_orientation(&self, display: bool) {
        self.display_orientation.set(display);
    }

    /// `DisplayTriangulation(D)`.
    pub fn set_display_triangulation(&self, display: bool) {
        self.triangulations.set(display);
    }

    /// `DisplayTriangulation()`.
    pub fn display_triangulation(&self) -> bool {
        self.triangulations.get()
    }

    /// `DisplayPolygons(D)`.
    pub fn set_display_polygons(&self, display: bool) {
        self.polygons.set(display);
    }

    /// `DisplayPolygons()`.
    pub fn display_polygons(&self) -> bool {
        self.polygons.get()
    }

    /// `DisplayHLR(withHLR, withRg1, withRgN, withHid, ang)`.
    pub fn display_hlr(&self, settings: HlrSettings) {
        self.hlr.set(settings);
    }

    /// `GetDisplayHLR`.
    pub fn hlr_settings(&self) -> HlrSettings {
        self.hlr.get()
    }

    /// `updateDisplayData`: the face records (isos when `NbIsos` is not 0 and the face
    /// has a surface) and the non-degenerated edges colored by their face count.
    fn update_display_data(&self, out: &dyn DrawConsole) -> Result<(), Failure> {
        self.faces.borrow_mut().clear();
        self.edges.borrow_mut().clear();
        if self.shape.is_null() {
            return Ok(());
        }
        *self.faces.borrow_mut() = self.face_records(out)?;
        *self.edges.borrow_mut() = self.edge_records()?;
        Ok(())
    }

    /// One `DBRep_Face` per face of the shape.
    fn face_records(&self, out: &dyn DrawConsole) -> Result<Vec<DBRepFace>, Failure> {
        let mut records = Vec::new();
        let mut explorer = Explorer::new(&self.shape, ShapeEnum::Face, ShapeEnum::Shape)?;
        while explorer.more() {
            let mut face = to_face(explorer.current())?;
            explorer.next()?;
            if self.nb_isos.get() == 0 || BRepTool::surface_with_location(&face)?.0.is_null() {
                records.push(DBRepFace::new(face, 0, self.colors.edge));
                continue;
            }
            face.set_orientation(Orientation::Forward);
            let builder = DBRepIsoBuilder::new(&face, self.size, self.nb_isos.get())?;
            let mut record = DBRepFace::new(face, builder.nb_domains(), self.colors.isos);
            builder.load_isos(&mut record, out)?;
            records.push(record);
        }
        Ok(records)
    }

    /// One `DBRep_Edge` per non-degenerated edge: isolated, free or connected color.
    fn edge_records(&self) -> Result<Vec<DBRepEdge>, Failure> {
        let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
        map_shapes_and_ancestors(
            &self.shape,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut ancestors,
        )?;
        let mut records = Vec::new();
        for index in 1..=ancestors.extent() {
            let edge = to_edge(ancestors.find_key(index)?)?;
            if BRepTool::degenerated(&edge)? {
                continue;
            }
            let color = match ancestors.find_from_index(index)?.extent() {
                0 => self.colors.edge,
                1 => self.colors.free,
                _ => self.colors.connected,
            };
            records.push(DBRepEdge::new(edge, color));
        }
        Ok(records)
    }

    /// The draw with the iso reports on `out` (`DrawOn` prints them on `std::cout`).
    pub fn draw_with_reports(
        &self,
        display: &mut DrawDisplay<'_>,
        out: &dyn DrawConsole,
    ) -> Result<(), Failure> {
        if self.shape.is_null() {
            display.set_color(self.colors.connected);
            display.draw_string_3d(
                rustycad_tkmath::gp::Point3::new(0.0, 0.0, 0.0),
                "Null Shape",
            );
            return Ok(());
        }
        if self.faces.borrow().is_empty() || self.edges.borrow().is_empty() {
            self.update_display_data(out)?;
        }
        if self.hlr.get().with_hlr {
            return self.display_hidden_lines(display);
        }
        let mut halt = false;
        self.draw_faces(display, &mut halt)?;
        self.draw_edges(display, &mut halt, out)?;
        self.draw_vertices(display, &mut halt)
    }
}

/// `DBRep_ColorOrientation`: red FORWARD, blue REVERSED, pink EXTERNAL, orange
/// INTERNAL.
///
/// ```
/// use rustycad_tkdraw::dbrep::color_orientation;
/// use rustycad_tkdraw::draw::DrawColorKind;
/// use rustycad_tkg3d::top_abs::Orientation;
/// assert_eq!(color_orientation(Orientation::Reversed).id(), DrawColorKind::Bleu);
/// ```
pub fn color_orientation(orientation: Orientation) -> DrawColor {
    DrawColor::new(match orientation {
        Orientation::Forward => DrawColorKind::Rouge,
        Orientation::Reversed => DrawColorKind::Bleu,
        Orientation::External => DrawColorKind::Rose,
        Orientation::Internal => DrawColorKind::Orange,
    })
}

impl Drawable3D for DBRepDrawableShape {
    fn base(&self) -> &Drawable3DBase {
        &self.base
    }

    fn type_name(&self) -> &'static str {
        "DBRep_DrawableShape"
    }

    /// `DrawOn`: "Null Shape" for a null shape; the hidden lines in HLR mode; else
    /// the face isos (and triangulations), the edges (and polygons) and the free
    /// vertices, until a pick hits. The iso builder reports go to the process
    /// standard output, as `std::cout` does.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        self.draw_with_reports(display, &crate::draw::ProcessConsole)
    }

    /// `Copy`: a drawable of the same shape, colors, size, isos and discretisation
    /// (the display flags return to their defaults, as upstream).
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        let copy = Self::new(
            self.shape.clone(),
            self.colors,
            self.size,
            self.nb_isos.get(),
            self.discret.get(),
        );
        Ok(Rc::new(copy))
    }

    /// `Dump`: `BRepTools::Dump`.
    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        stream.write_through(|io| brep_tools::dump(&self.shape, io))
    }

    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        self.save_shape(stream)
    }

    /// `Whatis`: `shape <TYPE> <ORIENTATION>` and the set flags; nothing for a null
    /// shape.
    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        if self.shape.is_null() {
            return Ok(());
        }
        let kind = top_abs::shape_type_to_string(self.shape.shape_type()?);
        let orientation = top_abs::shape_orientation_to_string(self.shape.orientation());
        di.append("shape ")?
            .append(kind)?
            .append(" ")?
            .append(orientation)?;
        let Some(tshape) = self.shape.tshape().get() else {
            return Ok(());
        };
        let flags = [
            (tshape.free(), " Free"),
            (tshape.modified(), " Modified"),
            (tshape.orientable(), " Orientable"),
            (tshape.closed(), " Closed"),
            (tshape.infinite(), " Infinite"),
            (tshape.convex(), " Convex"),
        ];
        for (set, text) in flags {
            if set {
                di.append(text)?;
            }
        }
        Ok(())
    }
}
