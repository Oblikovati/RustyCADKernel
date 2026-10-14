// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_EXPORT.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_EXPORT`: the free functions reading and completing the interferences
//! of the data structure (FDS_* and FUN_ds_*). The functions natively taking the
//! handled structure lock it once (`change_ds`) and work on the structure; their `_ds`
//! twins take the structure itself.
mod complete;
mod complete_e;
pub(crate) mod complete_se;
pub(crate) mod config;
mod data;
mod purge;
mod same_domain;
mod vertex;

pub use complete::{fun_ds_complete_for_se4, fun_ds_complete_for_se5, fun_ds_complete_for_se6};
pub use complete_e::{fun_ds_complete_for_e7, fun_ds_complete_for_se8, fun_ds_complete_for_se9};
pub use complete_se::{fun_ds_complete_for_se1, fun_ds_complete_for_se2, fun_ds_complete_for_se3};
pub use config::{
    ORIEINF_CLOSEOPPO, ORIEINF_CLOSESAME, ORIEINF_FORREVOPPO, ORIEINF_ONOPPOSHA, ORIEINF_ONSAMESHA,
    ORIEINF_UNKNOWN, fds_config3d, fds_getupperlower, fds_lo_iinfsup, fds_parbefaft,
    fds_state_e_with_f2d, fun_ds_mk_ton_fsdm, fun_ds_one_sd, fun_ds_ori_e_in_f, fun_ds_share_g,
};
pub use data::{
    KindIndices, TransitionData, fds_assign, fds_copy, fds_data, fds_data_iter, fds_has_unk,
    fds_idata, fds_parameter, fds_parameter_checked, fds_set_t, fds_tdata, fun_ds_sam_rk,
    fun_select_traint_interference,
};
pub use purge::{
    fun_ds_add_se_sdm1d, fun_ds_feig_b1_to0, fun_ds_fill_sdm_faces, fun_ds_has_fei, fun_ds_has_i2d,
    fun_ds_purge_for_e9,
};
pub use same_domain::{
    fds_aresamdom, fds_aresamdom_index, fds_edge_is_connex_to_same_domain_faces,
    fds_has_same_domain2d, fds_has_same_domain3d, fds_si_is_gi_of_i_of_sba_of_t_of_i,
    fun_ds_get_vsdm, fun_ds_getoov, fun_ds_getoov_hds, fun_ds_sdm,
};
pub use vertex::{fun_ds_complete1d_for_se_sdm, fun_ds_point_to_vertex, fun_ds_redusamsha};

use super::{TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure, TopOpeBRepDSInterference};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};

/// The interference of a handle; Standard_NullObject for a null handle (natively a
/// null dereference).
pub(crate) fn iref(
    i: &Handle<TopOpeBRepDSInterference>,
) -> Result<&TopOpeBRepDSInterference, Failure> {
    i.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "TopOpeBRepDS_EXPORT: null interference; expected an interference handle",
            "TopOpeBRepDS_Interference",
        )
    })
}

/// The handled structure; Standard_NullObject for a null handle.
pub(crate) fn href(
    hds: &Handle<TopOpeBRepDSHDataStructure>,
) -> Result<&TopOpeBRepDSHDataStructure, Failure> {
    hds.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "TopOpeBRepDS_EXPORT: null data structure; expected a TopOpeBRepDS_HDataStructure",
            "TopOpeBRepDS_HDataStructure",
        )
    })
}

/// `HDS->HasSameDomain(S)` on the structure: a kept shape with same domain shapes.
pub(crate) fn has_same_domain(ds: &TopOpeBRepDSDataStructure, s: &Shape) -> bool {
    ds.has_shape(s, true) && !ds.shape_same_domain(s).is_empty()
}
