//! `PrsDim_Chamf2dDimension` and `PrsDim_Chamf3dDimension`: the size of a chamfer edge (in
//! a plane) or face, drawn as a leader from the chamfer middle to the label.
use super::dimension_impls::prs_dim_relation_impls;
use super::relation_support::{
    automatic_or_projected, null_object, relation_arrow, text_box_selection,
};
use super::*;
use crate::dsg_prs::{DsgPrsArrowSide, DsgPrsChamf2dPresentation};
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, prs_mgr_presentable_bind};
use crate::select_mgr::SelectMgrSelection;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomLine, GeomPlane};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;

/// The attach point and leader direction of a chamfer dimension (`myPntAttach`, `myDir`).
#[derive(Clone, Copy, Debug, Default)]
pub struct PrsDimChamfFields {
    /// `myPntAttach`.
    pub pnt_attach: Point3,
    /// `myDir`.
    pub dir: Direction3,
}

/// The arrow size of a chamfer: the value quarter clamped to [8, 30] unless defined, applied
/// to the arrow aspect.
fn apply_chamf_arrow_size<R: PrsDimRelationSource + ?Sized>(relation: &R) -> Result<(), Failure> {
    let (defined, size, value) = {
        let f = relation.relation_base().fields.read();
        (f.arrow_size_is_defined, f.arrow_size, f.val)
    };
    if !defined {
        let mut size = size;
        if value / 4.0 < size {
            size = value / 4.0;
        }
        size = size.clamp(8.0, 30.0);
        relation.relation_base().fields.write().arrow_size = size;
    }
    relation_arrow(relation)?
        .get()
        .ok_or_else(null_object)?
        .set_length(relation.arrow_size());
    Ok(())
}

/// The label position, arrow size and leader of a chamfer.
fn draw_chamf<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
    prs: &Handle<dyn Graphic3dStructureSource>,
    fields: PrsDimChamfFields,
) -> Result<(), Failure> {
    let position = automatic_or_projected(relation, &fields.pnt_attach, &fields.dir, 5.0)?;
    apply_chamf_arrow_size(relation)?;
    let f = relation.relation_base().fields.read().clone();
    DsgPrsChamf2dPresentation::add_with_arrow(
        prs,
        &relation.attributes(),
        &fields.pnt_attach,
        &position,
        &f.text,
        f.symbol_prs,
    )
}

/// The chamfer selection: the leader and a small box at the label.
fn chamf_selection<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
    selection: &Handle<SelectMgrSelection>,
    attach: &Point3,
) -> Result<(), Failure> {
    let f = relation.relation_base().fields.read().clone();
    let size = (f.val / 100.0 + 1.0e-6).min(f.arrow_size + 1.0e-6);
    text_box_selection(relation, selection, 7, Some((attach, &f.position)), size)
}

macro_rules! chamf_dimension {
    ($type:ident, $native:literal, $doc:literal) => {
        #[doc = $doc]
        pub struct $type {
            transient: StandardTransient,
            pub(super) base: PrsDimRelationBase,
            /// The attach point and leader direction.
            pub fields: SharedState<PrsDimChamfFields>,
        }
        rustycad_tkernel::impl_transient!($type, $native, dyn PrsDimRelationSource, transient,
            interfaces: [dyn PrsDimRelationSource, dyn crate::ais::AisInteractiveObject, dyn crate::select_mgr::SelectMgrSelectableObject, dyn crate::prs_mgr::PrsMgrPresentableObject]);
        prs_dim_relation_impls!($type);
        impl PrsDimRelationSource for $type {
            fn relation_base(&self) -> &PrsDimRelationBase {
                &self.base
            }
            /// `KindOfDimension`: a length.
            fn kind_of_dimension(&self) -> PrsDimKindOfDimension {
                PrsDimKindOfDimension::Length
            }
            /// `IsMovable`: true.
            fn is_movable(&self) -> bool {
                true
            }
        }
        impl $type {
            fn create(
                shape: &Shape,
                plane: Option<&Handle<GeomPlane>>,
                value: f64,
                text: &ExtendedString,
            ) -> Handle<Self> {
                let relation = Handle::new(Self {
                    transient: StandardTransient::default(),
                    base: PrsDimRelationBase::new(crate::prs_mgr::PrsMgrTypeOfPresentation3d::AllView),
                    fields: SharedState::new(PrsDimChamfFields::default()),
                });
                prs_mgr_presentable_bind(&relation.cast());
                {
                    let mut f = relation.base.fields.write();
                    f.f_shape = shape.clone();
                    if let Some(plane) = plane {
                        f.plane = plane.clone();
                    }
                    f.val = value;
                    f.text = text.clone();
                }
                relation
            }
            fn automatic(
                shape: &Shape,
                plane: Option<&Handle<GeomPlane>>,
                value: f64,
                text: &ExtendedString,
            ) -> Handle<Self> {
                let relation = Self::create(shape, plane, value, text);
                {
                    let mut f = relation.base.fields.write();
                    f.symbol_prs = DsgPrsArrowSide::LastAr;
                    f.automatic_position = true;
                    f.arrow_size = f.val / 100.0;
                }
                relation
            }
            #[allow(clippy::too_many_arguments)]
            fn placed(
                shape: &Shape,
                plane: Option<&Handle<GeomPlane>>,
                value: f64,
                text: &ExtendedString,
                position: &Point3,
                symbol: DsgPrsArrowSide,
                arrow_size: f64,
            ) -> Handle<Self> {
                let relation = Self::create(shape, plane, value, text);
                {
                    let mut f = relation.base.fields.write();
                    f.position = *position;
                    f.symbol_prs = symbol;
                }
                relation.set_arrow_size(arrow_size);
                relation.base.fields.write().automatic_position = false;
                relation
            }
            fn compute_relation_selection(
                &self,
                selection: &Handle<SelectMgrSelection>,
                _: i32,
            ) -> Result<(), Failure> {
                let attach = self.fields.read().pnt_attach;
                chamf_selection(self, selection, &attach)
            }
        }
    };
}

chamf_dimension!(
    PrsDimChamf2dDimension,
    "PrsDim_Chamf2dDimension",
    "`PrsDim_Chamf2dDimension`: a chamfer edge in a plane; `PrsDimChamf2dDimension::new(&edge, &plane, 2.0, &text)`."
);
chamf_dimension!(
    PrsDimChamf3dDimension,
    "PrsDim_Chamf3dDimension",
    "`PrsDim_Chamf3dDimension`: a chamfer face; `PrsDimChamf3dDimension::new(&face, 2.0, &text)`."
);

impl PrsDimChamf2dDimension {
    /// `PrsDim_Chamf2dDimension(shape, plane, value, text)`: an automatic label.
    /// Example: `PrsDimChamf2dDimension::new(&edge, &plane, 2.0, &text)`.
    pub fn new(
        shape: &Shape,
        plane: &Handle<GeomPlane>,
        value: f64,
        text: &ExtendedString,
    ) -> Handle<Self> {
        Self::automatic(shape, Some(plane), value, text)
    }
    /// `PrsDim_Chamf2dDimension(shape, plane, value, text, position, symbol, arrowSize)`.
    /// Example: `PrsDimChamf2dDimension::with_position(&e, &pl, 2.0, &t, &p, side, 1.0)`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_position(
        shape: &Shape,
        plane: &Handle<GeomPlane>,
        value: f64,
        text: &ExtendedString,
        position: &Point3,
        symbol: DsgPrsArrowSide,
        arrow_size: f64,
    ) -> Handle<Self> {
        Self::placed(
            shape,
            Some(plane),
            value,
            text,
            position,
            symbol,
            arrow_size,
        )
    }
    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let edge = topods::edge(&self.first_shape())?;
        let Some((curve, (first, last))) = PrsDim::compute_edge_geometry(&edge)? else {
            return Ok(());
        };
        let line = curve.cast::<GeomLine>();
        let direction = line.get().ok_or_else(null_object)?.position().direction();
        let plane = self.plane();
        let normal = plane
            .get()
            .ok_or_else(null_object)?
            .pln()
            .axis()
            .direction();
        let fields = PrsDimChamfFields {
            pnt_attach: Point3::from((first.xyz() + last.xyz()) / 2.0),
            dir: normal.crossed(direction)?,
        };
        *self.fields.write() = fields;
        draw_chamf(self, prs, fields)
    }
}

impl PrsDimChamf3dDimension {
    /// `PrsDim_Chamf3dDimension(shape, value, text)`: an automatic label.
    /// Example: `PrsDimChamf3dDimension::new(&face, 2.0, &text)`.
    pub fn new(shape: &Shape, value: f64, text: &ExtendedString) -> Handle<Self> {
        Self::automatic(shape, None, value, text)
    }
    /// `PrsDim_Chamf3dDimension(shape, value, text, position, symbol, arrowSize)`.
    /// Example: `PrsDimChamf3dDimension::with_position(&f, 2.0, &t, &p, side, 1.0)`.
    pub fn with_position(
        shape: &Shape,
        value: f64,
        text: &ExtendedString,
        position: &Point3,
        symbol: DsgPrsArrowSide,
        arrow_size: f64,
    ) -> Handle<Self> {
        Self::placed(shape, None, value, text, position, symbol, arrow_size)
    }
    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let face = topods::face(&self.first_shape())?;
        let surface = BRepAdaptorSurface::with_face(&face, true)?;
        let u = (surface.first_u_parameter()? + surface.last_u_parameter()?) / 2.0;
        let v = (surface.first_v_parameter()? + surface.last_v_parameter()?) / 2.0;
        let d1 = surface.d1(u, v)?;
        let fields = PrsDimChamfFields {
            pnt_attach: d1.point,
            dir: Direction3::from_vector(d1.d1u.crossed(d1.d1v))?,
        };
        *self.fields.write() = fields;
        draw_chamf(self, prs, fields)
    }
}
