//! AIS_ExclusionFilter bookkeeping, AIS_TypeFilter/AIS_SignatureFilter on owners and
//! AIS_GraphicTool on a default drawer (the native values; the TKV3d numeric probe
//! v3_ais_attr_filter compares the complete behaviour with native OCCT).
use rustycad_tkernel::quantity::NamedColor;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkv3d::ais::{
    AisAttributeFilter, AisExclusionFilter, AisGraphicTool, AisKindOfInteractive, AisPoint,
    AisSignatureFilter, AisTypeFilter, AisTypeOfAttribute,
};
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::select_mgr::{
    SelectMgrEntityOwner, SelectMgrEntityOwnerSource, SelectMgrFilter,
};

/// The owner of a point; the owner keeps a weak reference, so the point is returned too.
fn point_owner() -> (Handle<AisPoint>, Handle<dyn SelectMgrEntityOwnerSource>) {
    let point = AisPoint::new(&Handle::new(GeomCartesianPoint::new(1., 2., 3.)).cast()).unwrap();
    let owner = SelectMgrEntityOwner::with_selectable(&point.clone().cast(), 0).into_cast();
    (point, owner)
}

#[test]
fn exclusion_filter_stores_kinds_and_signatures() {
    let filter = AisExclusionFilter::with_signature(AisKindOfInteractive::Datum, 1, true);
    assert!(filter.is_stored(AisKindOfInteractive::Datum));
    assert!(!filter.add(AisKindOfInteractive::Datum));
    assert!(filter.add_signature(AisKindOfInteractive::Datum, 4));
    assert_eq!(
        filter.list_of_signature(AisKindOfInteractive::Datum),
        vec![1, 4]
    );
    assert!(filter.add(AisKindOfInteractive::Shape));
    assert_eq!(filter.list_of_stored_types(), vec![1, 2]);
    // A point is a datum of signature 1: excluded.
    let (_point, owner) = point_owner();
    assert!(!filter.is_ok(&owner).unwrap());
    assert!(filter.remove_signature(AisKindOfInteractive::Datum, 1));
    assert!(filter.is_ok(&owner).unwrap());
    filter.clear();
    assert!(filter.list_of_stored_types().is_empty());
}

#[test]
fn type_signature_and_attribute_filters_read_the_object() {
    let (_point, owner) = point_owner();
    assert!(
        AisTypeFilter::new(AisKindOfInteractive::Datum)
            .is_ok(&owner)
            .unwrap()
    );
    assert!(
        !AisTypeFilter::new(AisKindOfInteractive::Shape)
            .is_ok(&owner)
            .unwrap()
    );
    assert!(
        AisSignatureFilter::new(AisKindOfInteractive::Datum, 1)
            .is_ok(&owner)
            .unwrap()
    );
    assert!(
        !AisSignatureFilter::new(AisKindOfInteractive::Datum, 2)
            .is_ok(&owner)
            .unwrap()
    );
    // Without own color, the color criterion does not apply.
    let filter = AisAttributeFilter::with_color(NamedColor::RED);
    assert!(filter.has_color() && !filter.has_width());
    assert!(filter.is_ok(&owner).unwrap());
    filter.unset_color();
    assert!(!filter.has_color());
}

#[test]
fn graphic_tool_reads_the_default_drawer_aspects() {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    let (_, width, kind) =
        AisGraphicTool::get_line_att(&drawer, AisTypeOfAttribute::Hidden).unwrap();
    assert_eq!(width, 1.);
    assert_eq!(kind, AspectTypeOfLine::Dash);
    assert_eq!(
        AisGraphicTool::get_line_width(&drawer, AisTypeOfAttribute::Wire).unwrap(),
        1.
    );
    assert!(AisGraphicTool::get_material(&drawer).is_ok());
    // A drawer without aspects has nothing to read.
    let bare = Handle::new(Prs3dDrawer::new());
    assert!(AisGraphicTool::get_line_width(&bare, AisTypeOfAttribute::Line).is_err());
}
