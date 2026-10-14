// SPDX-License-Identifier: GPL-2.0-only
//! Independent integer-mask identities preserve native combinations and key positions.
use martensite_tkmeshvs::flags::*;
#[test]
fn masks_and_unnamed_native_entity_combinations_keep_integer_semantics() {
    assert_eq!(MESHVS_DMF_OCC_MASK,3);
    assert_eq!(MESHVS_DMF_SHRINK,MESHVS_DMF_WIRE_FRAME|MESHVS_DMF_SHADING);
    assert_eq!(MESHVS_DMF_DEFORMED_MASK,384);
    assert_eq!(MESHVS_ET_ELEMENT.0,2|4|8|16);
    assert_eq!(MESHVS_ET_ALL.0,MESHVS_ET_ELEMENT.0|MESHVS_ET_NODE.0);
    assert_eq!(MeshVSEntityType(9).0,MESHVS_ET_NODE.0|MESHVS_ET_FACE.0);
    assert_eq!(MESHVS_DMF_USER,0x800);
}
#[test]
fn appearance_key_positions_preserve_native_unimplemented_renderer_properties() {
    assert_eq!(MeshVSDrawerAttribute::InteriorStyle as i32,0);
    assert_eq!(MeshVSDrawerAttribute::TextExpansionFactor as i32,18);
    assert_eq!(MeshVSDrawerAttribute::TextSpace as i32,19);
    assert_eq!(MeshVSDrawerAttribute::TextDisplayType as i32,21);
    assert_eq!(MeshVSDrawerAttribute::SupressBackFaces as i32,38);
    assert_eq!(MeshVSDrawerAttribute::User as i32,39);
}
