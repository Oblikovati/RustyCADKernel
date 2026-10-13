//! Complete configured FFP lighting/texture/cap fixtures; never infer Mesa25 pixels from26.
type FfpMesa25Asset = (&'static str, &'static [u8], &'static [u8]);
#[path = "ffp_mesa25_assets_0.rs"]
mod assets_0;
#[path = "ffp_mesa25_assets_1.rs"]
mod assets_1;
#[path = "ffp_mesa25_assets_2.rs"]
mod assets_2;
#[path = "ffp_mesa25_assets_3.rs"]
mod assets_3;
pub(super) fn find_mesa25_asset(mode: &str) -> Option<(&'static [u8], &'static [u8])> {
    assets_0::ASSETS
        .iter()
        .chain(assets_1::ASSETS)
        .chain(assets_2::ASSETS)
        .chain(assets_3::ASSETS)
        .find(|row| row.0 == mode)
        .map(|row| (row.1, row.2))
}
