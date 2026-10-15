//! The interactive-object trait implementations shared by every concrete dimension.

/// Implements `AisInteractiveObject` (a dimension), `PrsMgrPresentableObject` (display mode 0,
/// `Compute` through the inherent `compute_dimension`) and `SelectMgrSelectableObject`
/// (`PrsDim_Dimension::ComputeSelection`) for a struct with a `base: PrsDimDimensionBase`.
macro_rules! prs_dim_dimension_impls {
    ($type:ty) => {
        impl $crate::ais::AisInteractiveObject for $type {
            fn interactive_base(&self) -> &$crate::ais::AisInteractiveObjectBase {
                &self.base.interactive
            }
            /// `Type`: a dimension.
            fn kind_of_interactive(&self) -> $crate::ais::AisKindOfInteractive {
                $crate::ais::AisKindOfInteractive::Dimension
            }
        }
        impl $crate::prs_mgr::PrsMgrPresentableObject for $type {
            fn presentable_base(&self) -> &$crate::prs_mgr::PrsMgrPresentableObjectBase {
                &self.base.interactive.presentable
            }
            fn compute(
                &self,
                _: &dyn rustycad_tkernel::standard::HandleReference<
                    $crate::prs_mgr::PrsMgrPresentationManager,
                >,
                prs: &dyn rustycad_tkernel::standard::HandleReference<
                    dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource,
                >,
                mode: i32,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.compute_dimension(&prs.clone_handle(), mode)
            }
            /// `AcceptDisplayMode`: the whole dimension only.
            fn accept_display_mode(&self, mode: i32) -> bool {
                mode == $crate::prs_dim::PrsDimComputeMode::All as i32
            }
            $crate::ais_presentable_overrides!();
        }
        impl $crate::select_mgr::SelectMgrSelectableObject for $type {
            fn selectable_base(&self) -> &$crate::select_mgr::SelectMgrSelectableObjectBase {
                &self.base.interactive.selectable
            }
            fn compute_selection(
                &self,
                selection: &dyn rustycad_tkernel::standard::HandleReference<
                    $crate::select_mgr::SelectMgrSelection,
                >,
                mode: i32,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                $crate::prs_dim::prs_dim_dimension_compute_selection(
                    self,
                    &selection.clone_handle(),
                    mode,
                )
            }
        }
    };
}
pub(crate) use prs_dim_dimension_impls;

/// Implements `AisInteractiveObject` (a relation), `PrsMgrPresentableObject` (display mode 0,
/// `Compute` through the inherent `compute_relation`, the relation `SetColor`/`UnsetColor`)
/// and `SelectMgrSelectableObject` (through the inherent `compute_relation_selection`) for a
/// struct with a `base: PrsDimRelationBase`.
macro_rules! prs_dim_relation_impls {
    ($type:ty) => {
        impl $crate::ais::AisInteractiveObject for $type {
            fn interactive_base(&self) -> &$crate::ais::AisInteractiveObjectBase {
                &self.base.interactive
            }
            /// `Type`: a relation.
            fn kind_of_interactive(&self) -> $crate::ais::AisKindOfInteractive {
                $crate::ais::AisKindOfInteractive::Relation
            }
        }
        impl $crate::prs_mgr::PrsMgrPresentableObject for $type {
            fn presentable_base(&self) -> &$crate::prs_mgr::PrsMgrPresentableObjectBase {
                &self.base.interactive.presentable
            }
            fn compute(
                &self,
                _: &dyn rustycad_tkernel::standard::HandleReference<
                    $crate::prs_mgr::PrsMgrPresentationManager,
                >,
                prs: &dyn rustycad_tkernel::standard::HandleReference<
                    dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource,
                >,
                mode: i32,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.compute_relation(&prs.clone_handle(), mode)
            }
            /// `AcceptDisplayMode`: mode 0 only.
            fn accept_display_mode(&self, mode: i32) -> bool {
                mode == 0
            }
            fn set_color(
                &self,
                color: rustycad_tkernel::quantity::Color,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                $crate::prs_dim::prs_dim_relation_set_color(self, color)
            }
            fn unset_color(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
                $crate::prs_dim::prs_dim_relation_unset_color(self)
            }
            $crate::ais_presentable_overrides!();
        }
        impl $crate::select_mgr::SelectMgrSelectableObject for $type {
            fn selectable_base(&self) -> &$crate::select_mgr::SelectMgrSelectableObjectBase {
                &self.base.interactive.selectable
            }
            fn compute_selection(
                &self,
                selection: &dyn rustycad_tkernel::standard::HandleReference<
                    $crate::select_mgr::SelectMgrSelection,
                >,
                mode: i32,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.compute_relation_selection(&selection.clone_handle(), mode)
            }
        }
    };
}
pub(crate) use prs_dim_relation_impls;
