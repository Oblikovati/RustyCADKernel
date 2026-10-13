use super::*;
const CAP_MODE: &str =
    "cap-section-edges-generated-polygon-thin-fixed-bilinear-fixed-center-edges-disabled";
const LIGHT_MODE: &str = "lit-alpha-opacity-updated";
fn software_identity(version: (u32, u32, u32), info: &str) -> RenderDriverIdentity {
    RenderDriverIdentity {
        vendor_id: 0x10005,
        device_id: 0,
        driver_id: 13,
        driver_version: (version.0 << 22) | (version.1 << 12) | version.2,
        driver_name: "llvmpipe".into(),
        driver_info: info.into(),
        device_name: "llvmpipe (LLVM 20.1.2, 128 bits)".into(),
    }
}
fn mesa25_identity() -> RenderDriverIdentity {
    software_identity((25, 2, 8), "Mesa 25.2.8-0ubuntu0.24.04.4 (LLVM 20.1.2)")
}
fn mesa26_identity() -> RenderDriverIdentity {
    software_identity(
        (26, 1, 6),
        "Mesa 26.1.6-1pop0~1787580452~24.04~a5619ea (LLVM 20.1.2)",
    )
}
fn native_amd_identity() -> RenderDriverIdentity {
    RenderDriverIdentity {
        vendor_id: 0x1002,
        device_id: 0x1586,
        driver_id: 3,
        driver_version: (26 << 22) | (1 << 12) | 6,
        driver_name: "radv".into(),
        driver_info: "Mesa 26.1.6-1pop0~1787580452~24.04~a5619ea".into(),
        device_name: "AMD Radeon 8060S Graphics (RADV STRIX_HALO)".into(),
    }
}
#[test]
fn exact_software_profiles_select_separate_native_cohorts() {
    assert_eq!(
        ffp_native_oracle_profile(&mesa25_identity()).unwrap(),
        FfpNativeOracleProfile::SoftwareMesa25
    );
    assert_eq!(
        ffp_native_oracle_profile(&mesa26_identity()).unwrap(),
        FfpNativeOracleProfile::SoftwareMesa26
    );
}
#[test]
fn newly_proved_assets_have_full_color_and_depth_buffers() {
    for mode in [CAP_MODE, LIGHT_MODE] {
        let (rgba, depth) = mesa25_ffp_asset(mode).unwrap();
        assert_eq!(rgba.len(), 64 * 64 * 4);
        assert_eq!(depth.len(), 64 * 64 * 4);
    }
    assert!(mesa25_ffp_asset("unproved-fixture").is_none());
}
fn disagreeing_identity(field: u32) -> RenderDriverIdentity {
    let mut identity = mesa25_identity();
    match field {
        0 => identity.driver_version += 1,
        1 => identity.driver_id = 3,
        2 => identity.vendor_id = 0x1002,
        3 => identity.device_id = 1,
        4 => identity.driver_info.push('?'),
        5 => identity.driver_name = "radv".into(),
        _ => identity.device_name = "llvmpipe (LLVM 20.1.2, 256 bits)".into(),
    }
    identity
}
#[test]
fn metadata_disagreement_rejects_name_guessing() {
    for field in 0..7 {
        let error = ffp_native_oracle_profile(&disagreeing_identity(field)).unwrap_err();
        assert!(error.contains("received driver identity="));
        assert!(error.contains("expected proved"));
    }
}
#[test]
fn native_amd_path_requires_proved_device_and_driver() {
    let mut identity = native_amd_identity();
    assert_eq!(
        ffp_native_oracle_profile(&identity).unwrap(),
        FfpNativeOracleProfile::NativeAmdMesa26
    );
    identity.device_id = 0;
    assert!(ffp_native_oracle_profile(&identity).is_err());
}
fn selected_asset(
    identity: &RenderDriverIdentity,
    mode: &str,
) -> Option<(&'static [u8], &'static [u8])> {
    profiled_ffp_asset(identity, mode, (b"host", b""), (b"software", b"")).unwrap()
}
#[test]
fn mesa25_selection_uses_the_new_native_bytes() {
    let chosen = selected_asset(&mesa25_identity(), LIGHT_MODE).unwrap();
    assert_eq!(chosen, mesa25_ffp_asset(LIGHT_MODE).unwrap());
}
#[test]
fn mesa26_and_amd_selection_preserve_existing_bytes() {
    assert_eq!(
        selected_asset(&mesa26_identity(), LIGHT_MODE).unwrap().0,
        b"software"
    );
    assert_eq!(
        selected_asset(&native_amd_identity(), LIGHT_MODE)
            .unwrap()
            .0,
        b"host"
    );
}
#[test]
fn ordinary_assets_and_unknown_corrected_profiles_remain_distinct() {
    assert!(selected_asset(&mesa25_identity(), "ordinary-unchanged-mode").is_none());
    assert!(
        profiled_ffp_asset(&disagreeing_identity(0), LIGHT_MODE, (b"", b""), (b"", b"")).is_err()
    );
}

#[test]
fn independently_proved_mesa26_default256_preserves_existing_assets() {
    let mut identity = mesa26_identity();
    identity.device_name = "llvmpipe (LLVM 20.1.2, 256 bits)".into();
    assert_eq!(
        ffp_native_oracle_profile(&identity).unwrap(),
        FfpNativeOracleProfile::SoftwareMesa26
    );
    assert_eq!(
        selected_asset(&identity, LIGHT_MODE).unwrap().0,
        b"software"
    );
    assert_eq!(selected_asset(&identity, CAP_MODE).unwrap().0, b"software");
}

#[test]
fn complete_mesa25_family_retains_lighting_texture_and_cap_filter_assets() {
    for mode in [
        "lit-basic",
        "bilinear",
        "lit-bilinear",
        "cap-oblique-fixed-phong",
        "cap-section-edges-generated-polygon-thin-fixed-trilinear-fixed-minified-edges-disabled",
    ] {
        let (rgba, depth) = mesa25_ffp_asset(mode).unwrap();
        assert_eq!(rgba.len(), 64 * 64 * 4);
        assert_eq!(depth.len(), 64 * 64 * 4);
    }
}
