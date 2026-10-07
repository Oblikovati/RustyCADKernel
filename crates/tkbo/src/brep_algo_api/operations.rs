//! `BRepAlgoAPI_Cut`, `BRepAlgoAPI_Fuse` and `BRepAlgoAPI_Common`: the Boolean operations
//! with their operation fixed at construction.
use super::boolean_operation::BRepAlgoAPIBooleanOperation;
use crate::bop_algo::{BOPAlgoPaveFiller, Operation};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use std::ops::{Deref, DerefMut};

/// Defines one Boolean operation API over [`BRepAlgoAPIBooleanOperation`].
macro_rules! boolean_api {
    ($(#[$doc:meta])* $api:ident, $native:literal, $operation:expr) => {
        $(#[$doc])*
        pub struct $api {
            operation: BRepAlgoAPIBooleanOperation,
        }

        impl Default for $api {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Deref for $api {
            type Target = BRepAlgoAPIBooleanOperation;
            fn deref(&self) -> &BRepAlgoAPIBooleanOperation {
                &self.operation
            }
        }

        impl DerefMut for $api {
            fn deref_mut(&mut self) -> &mut BRepAlgoAPIBooleanOperation {
                &mut self.operation
            }
        }

        impl $api {
            #[doc = concat!("An empty operation (`", $native, "()`); `", stringify!($api), "::new()`.")]
            pub fn new() -> Self {
                let mut operation = BRepAlgoAPIBooleanOperation::new();
                operation.set_operation($operation);
                Self { operation }
            }

            #[doc = concat!("An operation on the results of a performed filler (`", $native, "(PF)`).")]
            pub fn with_filler(filler: BOPAlgoPaveFiller) -> Self {
                let mut operation = BRepAlgoAPIBooleanOperation::with_filler(filler);
                operation.set_operation($operation);
                Self { operation }
            }

            #[doc = concat!("The operation of `object` and `tool`, performed (`", $native, "(S1, S2)`).")]
            pub fn from_shapes(object: &Shape, tool: &Shape) -> Result<Self, Failure> {
                Self::performed(object, tool, None, $operation)
            }

            fn performed(
                object: &Shape,
                tool: &Shape,
                filler: Option<BOPAlgoPaveFiller>,
                operation: Operation,
            ) -> Result<Self, Failure> {
                let operation =
                    BRepAlgoAPIBooleanOperation::with_shapes(object, tool, filler, operation);
                let mut api = Self { operation };
                api.operation.build()?;
                Ok(api)
            }
        }
    };
}

boolean_api!(
    /// The part of the objects outside the tools.
    ///
    /// ```
    /// use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
    /// use rustycad_tkbo::bop_algo::Operation;
    /// assert_eq!(BRepAlgoAPICut::new().operation(), Operation::Cut);
    /// ```
    BRepAlgoAPICut,
    "BRepAlgoAPI_Cut",
    Operation::Cut
);

boolean_api!(
    /// The union of the objects and the tools.
    ///
    /// ```
    /// use rustycad_tkbo::brep_algo_api::BRepAlgoAPIFuse;
    /// use rustycad_tkbo::bop_algo::Operation;
    /// assert_eq!(BRepAlgoAPIFuse::new().operation(), Operation::Fuse);
    /// ```
    BRepAlgoAPIFuse,
    "BRepAlgoAPI_Fuse",
    Operation::Fuse
);

boolean_api!(
    /// The part common to the objects and the tools.
    ///
    /// ```
    /// use rustycad_tkbo::brep_algo_api::BRepAlgoAPICommon;
    /// use rustycad_tkbo::bop_algo::Operation;
    /// assert_eq!(BRepAlgoAPICommon::new().operation(), Operation::Common);
    /// ```
    BRepAlgoAPICommon,
    "BRepAlgoAPI_Common",
    Operation::Common
);

impl BRepAlgoAPICut {
    /// The cut of `object` by `tool` (`forward`) or of `tool` by `object` on the results of
    /// a performed filler (`BRepAlgoAPI_Cut(S1, S2, PF, bFWD)`).
    /// `BRepAlgoAPICut::from_shapes_with_filler(&a, &b, filler, true)?`.
    pub fn from_shapes_with_filler(
        object: &Shape,
        tool: &Shape,
        filler: BOPAlgoPaveFiller,
        forward: bool,
    ) -> Result<Self, Failure> {
        let operation = if forward {
            Operation::Cut
        } else {
            Operation::Cut21
        };
        Self::performed(object, tool, Some(filler), operation)
    }
}

impl BRepAlgoAPIFuse {
    /// The fuse of `object` and `tool` on the results of a performed filler
    /// (`BRepAlgoAPI_Fuse(S1, S2, PF)`); `BRepAlgoAPIFuse::from_shapes_with_filler(&a, &b, pf)?`.
    pub fn from_shapes_with_filler(
        object: &Shape,
        tool: &Shape,
        filler: BOPAlgoPaveFiller,
    ) -> Result<Self, Failure> {
        Self::performed(object, tool, Some(filler), Operation::Fuse)
    }
}

impl BRepAlgoAPICommon {
    /// The common of `object` and `tool` on the results of a performed filler
    /// (`BRepAlgoAPI_Common(S1, S2, PF)`); `BRepAlgoAPICommon::from_shapes_with_filler(&a, &b, pf)?`.
    pub fn from_shapes_with_filler(
        object: &Shape,
        tool: &Shape,
        filler: BOPAlgoPaveFiller,
    ) -> Result<Self, Failure> {
        Self::performed(object, tool, Some(filler), Operation::Common)
    }
}
