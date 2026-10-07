// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the settings, the model and the reading.

//! The settings of the session (share-out, library, protocol, signature type, model
//! copier), its model, and the reading of a file or a stream.
use super::{IFSelectWorkSession, report_interruption};
use crate::if_select::{
    IFSelectModelCopier, IFSelectReturnStatus, IFSelectShareOut, IFSelectSignature,
    IFSelectWorkLibrary,
};
use crate::interface::{
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceProtocol, InterfaceProtocolBase,
    InterfaceSignType,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, exceptions},
    tcollection::AsciiString,
};
use std::io::Read;

/// The key of the signature type among the names.
const SIGN_TYPE_NAME: &str = "xst-sign-type";

/// The status of a read: 0 done, negative an error, positive a failure.
fn read_status(stat: i32) -> IFSelectReturnStatus {
    match stat {
        0 => IFSelectReturnStatus::RetDone,
        s if s < 0 => IFSelectReturnStatus::RetError,
        _ => IFSelectReturnStatus::RetFail,
    }
}

impl IFSelectWorkSession {
    /// `ShareOut`.
    pub fn share_out(&self) -> Handle<IFSelectShareOut> {
        self.state.read().theshareout.clone()
    }
    /// `SetShareOut(shareout)`: also for the model copier.
    pub fn set_share_out(&self, shareout: &Handle<IFSelectShareOut>) {
        let copier = {
            let mut st = self.state.write();
            st.theshareout = shareout.clone();
            st.thecopier.clone()
        };
        if let Some(c) = copier.get() {
            c.set_share_out(shareout);
        }
    }
    /// `SetModeStat(theMode)`: the graph keeps the statuses and the categories.
    pub fn set_mode_stat(&self, mode: bool) {
        self.state.write().themodelstat = mode;
    }
    /// `GetModeStat`.
    pub fn get_mode_stat(&self) -> bool {
        self.state.read().themodelstat
    }
    /// `SetLibrary(theLib)`.
    pub fn set_library(&self, lib: &Handle<dyn IFSelectWorkLibrary>) {
        self.state.write().thelibrary = lib.clone();
    }
    /// `WorkLibrary`.
    pub fn work_library(&self) -> Handle<dyn IFSelectWorkLibrary> {
        self.state.read().thelibrary.clone()
    }
    /// `SetProtocol(protocol)`: also the active protocol and that of the general tool.
    pub fn set_protocol(&self, protocol: &Handle<dyn InterfaceProtocol>) {
        let gtool = {
            let mut st = self.state.write();
            st.theprotocol = protocol.clone();
            st.thegtool.clone()
        };
        InterfaceProtocolBase::set_active(protocol);
        if let Some(g) = gtool.get() {
            g.set_protocol(protocol, false);
        }
    }
    /// `Protocol`.
    pub fn protocol(&self) -> Handle<dyn InterfaceProtocol> {
        self.state.read().theprotocol.clone()
    }
    /// `SetSignType(signtype)`: the signature type of the general tool, also named
    /// "xst-sign-type".
    pub fn set_sign_type(&self, signtype: &Handle<dyn IFSelectSignature>) {
        let mut st = self.state.write();
        if let Some(g) = st.thegtool.get() {
            g.set_sign_type(&signtype.cast::<dyn InterfaceSignType>());
        }
        let key = AsciiString::new(SIGN_TYPE_NAME);
        if signtype.is_null() {
            st.thenames.unbind(&key);
        } else {
            st.thenames.bind(key, signtype.cast());
        }
    }
    /// `SignType`: the signature type of the general tool (null when not a signature).
    pub fn sign_type(&self) -> Handle<dyn IFSelectSignature> {
        let gtool = self.state.read().thegtool.clone();
        gtool
            .get()
            .map(|g| g.sign_type().cast())
            .unwrap_or_default()
    }
    /// `HasModel`.
    pub fn has_model(&self) -> bool {
        self.state.read().my_model.is_some()
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.state.read().my_model.clone()
    }
    /// `SetModel(model, clearpointed = true)`: the new model with the general tool, its
    /// graph computed (with a protocol) and the checks to be redone; the pointed
    /// selections are cleared with `clearpointed`. `Standard_NullObject` for a null
    /// model (a native null dereference).
    pub fn set_model(
        &self,
        model: &Handle<dyn InterfaceInterfaceModel>,
        clearpointed: bool,
    ) -> Result<(), Failure> {
        let gtool = {
            let mut st = self.state.write();
            if st.my_model != *model {
                st.theloaded = AsciiString::default();
            }
            st.my_model = model.clone();
            st.thegtool.clone()
        };
        if let Some(g) = gtool.get() {
            g.clear_entities(); // smh#14 FRA62479
        }
        let live = model.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::SetModel: null model; expected a model")
        })?;
        live.set_gtool(&gtool);
        self.state.write().thegraph = Handle::default();
        self.compute_graph(false)?; // does something if Protocol present. Otherwise, does nothing
        self.clear_data_virtual(3)?; // Reset CheckList, to be redone
        self.state.write().thecheckrun.clear();
        if clearpointed {
            self.clear_data_virtual(4)?;
        }
        self.clear_data_virtual(0)
    }
    /// `SetLoadedFile(theFileName)`.
    pub fn set_loaded_file(&self, filename: &str) {
        self.state.write().theloaded = AsciiString::new(filename);
    }
    /// `LoadedFile`.
    pub fn loaded_file(&self) -> AsciiString {
        self.state.read().theloaded.clone()
    }
    /// The reading library and protocol, `None` when one is missing.
    fn reading_tools(
        &self,
    ) -> Option<(
        Handle<dyn IFSelectWorkLibrary>,
        Handle<dyn InterfaceProtocol>,
    )> {
        let st = self.state.read();
        (st.thelibrary.is_some() && st.theprotocol.is_some())
            .then(|| (st.thelibrary.clone(), st.theprotocol.clone()))
    }
    /// The model read by `read`, set as the model of the session with `name`.
    fn take_read(
        &self,
        name: &str,
        read: impl FnOnce(&mut Handle<dyn InterfaceInterfaceModel>) -> Result<i32, Failure>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let mut model = Handle::default();
        let status = match read(&mut model) {
            Ok(stat) => read_status(stat),
            Err(failure) => {
                report_interruption(
                    "    ****    ReadFile Interruption by Exception :   ****",
                    &failure,
                );
                IFSelectReturnStatus::RetFail
            }
        };
        if status != IFSelectReturnStatus::RetDone {
            return Ok(status);
        }
        if model.is_null() {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        self.set_model(&model, true)?;
        self.set_loaded_file(name);
        Ok(status)
    }
    /// `ReadFile(filename)`: void without library or protocol; the errors of the
    /// library are reported as a failure.
    pub fn read_file(&self, filename: &str) -> Result<IFSelectReturnStatus, Failure> {
        let Some((lib, protocol)) = self.reading_tools() else {
            return Ok(IFSelectReturnStatus::RetVoid);
        };
        self.take_read(filename, |model| lib.read_file(filename, model, &protocol))
    }
    /// `ReadStream(theName, theIStream)`: as `ReadFile`, from a stream.
    pub fn read_stream(
        &self,
        name: &str,
        istream: &mut dyn Read,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let Some((lib, protocol)) = self.reading_tools() else {
            return Ok(IFSelectReturnStatus::RetVoid);
        };
        self.take_read(name, |model| {
            lib.read_stream(name, istream, model, &protocol)
        })
    }
    /// `ModelCopier`.
    pub fn model_copier(&self) -> Handle<IFSelectModelCopier> {
        self.state.read().thecopier.clone()
    }
    /// `SetModelCopier(copier)`: with the share-out of the session; `Standard_NullObject`
    /// for a null copier (a native null dereference).
    pub fn set_model_copier(&self, copier: &Handle<IFSelectModelCopier>) -> Result<(), Failure> {
        let live = copier.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::SetModelCopier: null copier")
        })?;
        let shareout = {
            let mut st = self.state.write();
            st.thecopier = copier.clone();
            st.theshareout.clone()
        };
        live.set_share_out(&shareout);
        Ok(())
    }
}
