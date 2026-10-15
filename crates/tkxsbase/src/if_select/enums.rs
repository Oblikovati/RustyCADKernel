// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_ReturnStatus.hxx,
// IFSelect_RemainMode.hxx, IFSelect_PrintCount.hxx, IFSelect_PrintFail.hxx and
// IFSelect_EditValue.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The enumerations of IFSelect.

/// `IFSelect_ReturnStatus`: the status of a command or an action.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum IFSelectReturnStatus {
    /// `IFSelect_RetVoid`: nothing done.
    #[default]
    RetVoid = 0,
    /// `IFSelect_RetDone`.
    RetDone = 1,
    /// `IFSelect_RetError`: an error in the data.
    RetError = 2,
    /// `IFSelect_RetFail`: an execution failure.
    RetFail = 3,
    /// `IFSelect_RetStop`: stop the session.
    RetStop = 4,
}

/// `IFSelect_RemainMode`: what to do with the entities not yet sent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum IFSelectRemainMode {
    /// `IFSelect_RemainForget`.
    #[default]
    RemainForget = 0,
    /// `IFSelect_RemainCompute`.
    RemainCompute = 1,
    /// `IFSelect_RemainDisplay`.
    RemainDisplay = 2,
    /// `IFSelect_RemainUndo`.
    RemainUndo = 3,
}

/// `IFSelect_PrintCount`: the modes of printing a count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum IFSelectPrintCount {
    /// `IFSelect_ItemsByEntity`.
    #[default]
    ItemsByEntity = 0,
    /// `IFSelect_CountByItem`.
    CountByItem = 1,
    /// `IFSelect_ShortByItem`.
    ShortByItem = 2,
    /// `IFSelect_ListByItem`.
    ListByItem = 3,
    /// `IFSelect_EntitiesByItem`.
    EntitiesByItem = 4,
    /// `IFSelect_CountSummary`.
    CountSummary = 5,
    /// `IFSelect_GeneralInfo`.
    GeneralInfo = 6,
    /// `IFSelect_Mapping`.
    Mapping = 7,
    /// `IFSelect_ResultCount`.
    ResultCount = 8,
}

/// `IFSelect_PrintFail`: print the fails only, or the warnings too.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum IFSelectPrintFail {
    /// `IFSelect_FailOnly`.
    #[default]
    FailOnly = 0,
    /// `IFSelect_FailAndWarn`.
    FailAndWarn = 1,
}

/// `IFSelect_EditValue`: the edit mode of a value of an editor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum IFSelectEditValue {
    /// `IFSelect_Optional`.
    #[default]
    Optional = 0,
    /// `IFSelect_Editable`.
    Editable = 1,
    /// `IFSelect_EditProtected`.
    EditProtected = 2,
    /// `IFSelect_EditComputed`.
    EditComputed = 3,
    /// `IFSelect_EditRead`.
    EditRead = 4,
    /// `IFSelect_EditDynamic`.
    EditDynamic = 5,
}

impl IFSelectPrintCount {
    /// The mode of a native integer value (`None` out of range).
    pub fn from_i32(value: i32) -> Option<Self> {
        const ALL: [IFSelectPrintCount; 9] = [
            IFSelectPrintCount::ItemsByEntity,
            IFSelectPrintCount::CountByItem,
            IFSelectPrintCount::ShortByItem,
            IFSelectPrintCount::ListByItem,
            IFSelectPrintCount::EntitiesByItem,
            IFSelectPrintCount::CountSummary,
            IFSelectPrintCount::GeneralInfo,
            IFSelectPrintCount::Mapping,
            IFSelectPrintCount::ResultCount,
        ];
        usize::try_from(value)
            .ok()
            .and_then(|i| ALL.get(i))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants() {
        assert_eq!(IFSelectReturnStatus::RetStop as i32, 4);
        assert_eq!(IFSelectRemainMode::RemainUndo as i32, 3);
        assert_eq!(
            IFSelectPrintCount::from_i32(8),
            Some(IFSelectPrintCount::ResultCount)
        );
        assert_eq!(IFSelectPrintCount::from_i32(9), None);
        assert_eq!(IFSelectPrintFail::FailAndWarn as i32, 1);
        assert_eq!(IFSelectEditValue::EditDynamic as i32, 5);
    }

    /// Every enumerator keeps its native IFSelect_*.hxx order and value.
    #[test]
    fn every_enumerator_keeps_native_value() {
        use IFSelectEditValue as E;
        let edit = [
            E::Optional,
            E::Editable,
            E::EditProtected,
            E::EditComputed,
            E::EditRead,
            E::EditDynamic,
        ];
        assert!(edit.iter().enumerate().all(|(i, v)| *v as i32 == i as i32));
        use IFSelectReturnStatus as R;
        let ret = [R::RetVoid, R::RetDone, R::RetError, R::RetFail, R::RetStop];
        assert!(ret.iter().enumerate().all(|(i, v)| *v as i32 == i as i32));
        use IFSelectRemainMode as M;
        let remain = [
            M::RemainForget,
            M::RemainCompute,
            M::RemainDisplay,
            M::RemainUndo,
        ];
        assert!(
            remain
                .iter()
                .enumerate()
                .all(|(i, v)| *v as i32 == i as i32)
        );
        let fail = [IFSelectPrintFail::FailOnly, IFSelectPrintFail::FailAndWarn];
        assert!(fail.iter().enumerate().all(|(i, v)| *v as i32 == i as i32));
        for value in 0..9 {
            let mode = IFSelectPrintCount::from_i32(value).expect("native PrintCount value");
            assert_eq!(mode as i32, value);
        }
    }
}
