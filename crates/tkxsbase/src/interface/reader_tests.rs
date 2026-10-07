//! Unit tests of the file reading classes (`Interface_FileReaderData`,
//! `Interface_ReaderModule`, `Interface_ReaderLib`, `Interface_FileReaderTool`) on a fake
//! file of the fake norm.
use super::test_norm::{FakeEntity, fake_model, fake_protocol};
use super::*;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient},
};
use std::sync::OnceLock;

/// A fake file: one record per entity, its parameters added by the test.
struct FakeReaderData {
    transient: StandardTransient,
    base: InterfaceFileReaderDataBase,
    nbrec: i32,
}
impl_transient!(FakeReaderData, "Fake_ReaderData", dyn InterfaceFileReaderData, transient, interfaces: [dyn InterfaceFileReaderData]);

impl InterfaceFileReaderData for FakeReaderData {
    fn reader_data_base(&self) -> &InterfaceFileReaderDataBase {
        &self.base
    }
    fn find_next_record(&self, num: i32) -> i32 {
        if num < self.nbrec { num + 1 } else { 0 }
    }
}

/// The reader module of the fake norm: a record whose first parameter is "?" is unknown.
#[derive(Default)]
struct FakeReaderModule {
    transient: StandardTransient,
}
impl_transient!(FakeReaderModule, "Fake_ReaderModule", dyn InterfaceReaderModule, transient, interfaces: [dyn InterfaceReaderModule]);

impl InterfaceReaderModule for FakeReaderModule {
    fn case_num(&self, data: &Handle<dyn InterfaceFileReaderData>, num: i32) -> i32 {
        let first = data.param_c_value(num, 1).unwrap_or_default();
        i32::from(first != b"?")
    }
    fn read(
        &self,
        _cn: i32,
        _data: &Handle<dyn InterfaceFileReaderData>,
        _num: i32,
        _ach: &mut Handle<InterfaceCheck>,
        _ent: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        Ok(())
    }
}

/// The reader tool of the fake norm: names and shared entities from the parameters.
#[derive(Default)]
struct FakeReaderTool {
    base: InterfaceFileReaderToolBase,
}

impl InterfaceFileReaderTool for FakeReaderTool {
    fn reader_tool_base(&self) -> &InterfaceFileReaderToolBase {
        &self.base
    }
    fn recognize(
        &self,
        num: i32,
        ach: &mut Handle<InterfaceCheck>,
        ent: &mut Handle<StandardTransient>,
    ) -> Result<bool, Failure> {
        let (mut glib, mut rlib) = (
            InterfaceGeneralLib::new(&self.protocol()),
            InterfaceReaderLib::new(&self.protocol()),
        );
        self.recognize_by_lib(num, &mut glib, &mut rlib, ach, ent)
    }
    fn begin_read(&self, _amodel: &Handle<dyn InterfaceInterfaceModel>) -> Result<(), Failure> {
        Ok(())
    }
    fn analyse_record(
        &self,
        num: i32,
        anent: &Handle<StandardTransient>,
        acheck: &mut Handle<InterfaceCheck>,
    ) -> Result<bool, Failure> {
        let data = self.data();
        let name = data.param_c_value(num, 1)?;
        let entity = anent.cast::<FakeEntity>();
        let Some(live) = entity.get() else {
            return Ok(true);
        };
        // The unknown entity a failed record gets is analysed again: it does not fail.
        if name == b"crash" && *live.name.read() != "unknown" {
            return Err(Failure::new("analysis failed"));
        }
        if name == b"bad" {
            acheck.add_fail_text("bad record", "");
        }
        *live.name.write() = String::from_utf8_lossy(&name).into_owned();
        for nump in 2..=data.nb_params(num)? {
            live.shared.write().push(data.param_entity(num, nump)?);
        }
        Ok(true)
    }
}

fn register_reader() {
    static DONE: OnceLock<()> = OnceLock::new();
    DONE.get_or_init(|| {
        let module: Handle<dyn InterfaceReaderModule> =
            Handle::new(FakeReaderModule::default()).cast();
        InterfaceReaderLib::set_global(&module, &fake_protocol());
    });
}

/// A fake file of records (name, referenced records...).
fn fake_file(records: &[(&str, &[i32])]) -> Handle<dyn InterfaceFileReaderData> {
    let data = FakeReaderData {
        transient: StandardTransient::default(),
        base: InterfaceFileReaderDataBase::new(records.len() as i32, 20),
        nbrec: records.len() as i32,
    };
    for (index, (name, refs)) in records.iter().enumerate() {
        let num = index as i32 + 1;
        data.add_param(num, name.as_bytes(), InterfaceParamType::ParamText, 0)
            .unwrap();
        for target in refs.iter() {
            data.add_param(
                num,
                format!("#{target}").as_bytes(),
                InterfaceParamType::ParamIdent,
                *target,
            )
            .unwrap();
        }
        data.init_params(num).unwrap();
    }
    Handle::new(data).cast()
}

fn read(records: &[(&str, &[i32])]) -> (FakeReaderTool, Handle<dyn InterfaceInterfaceModel>) {
    register_reader();
    let tool = FakeReaderTool::default();
    tool.set_data(&fake_file(records), &fake_protocol());
    tool.set_entities().unwrap();
    let model = fake_model();
    tool.load_model(&model).unwrap();
    (tool, model)
}

#[test]
fn reader_data_parameters() {
    let data = fake_file(&[("a", &[]), ("b", &[1, 1])]);
    assert_eq!((data.nb_records(), data.nb_entities()), (2, 2));
    assert_eq!(data.nb_params(2).unwrap(), 3);
    assert_eq!(data.nb_params(0).unwrap(), 4);
    assert_eq!(data.params(2).unwrap().length(), 3);
    assert_eq!(
        data.param_type(2, 2).unwrap(),
        InterfaceParamType::ParamIdent
    );
    assert_eq!(data.param_number(2, 3).unwrap(), 1);
    assert!(data.is_param_defined(1, 1).unwrap());
    assert_eq!(data.param_first_rank(1).unwrap(), 1);
    assert_eq!(data.param_position(3), (2, 0));
    data.change_param(1, 1, |p| p.set_entity_number(7)).unwrap();
    assert_eq!(data.param(1, 1).unwrap().entity_number(), 7);
    data.set_error_load(false);
    assert!(data.is_error_load() && !data.reset_error_load() && !data.is_error_load());
    assert!(data.bound_entity(9).is_null());
    assert!(data.bind_entity(9, &Handle::default()).is_err());
    assert_eq!(InterfaceFileReaderDataBase::fastof(b"1e2"), 100.0);
}

#[test]
fn reading_a_file_into_a_model() {
    let (tool, model) = read(&[("a", &[]), ("b", &[1]), ("?", &[]), ("bad", &[2])]);
    assert_eq!(model.nb_entities(), 4);
    let b = model.value(2).unwrap().cast::<FakeEntity>();
    assert_eq!(b.name.read().as_str(), "b");
    assert!(b.shared.read()[0] == model.value(1).unwrap());
    // The unrecognised record got the unknown entity and a report.
    assert!(model.is_report_entity(3, false));
    assert_eq!(model.entity_state(4), InterfaceDataState::StateUnloaded);
    assert!(model.is_error_entity(4));
    assert!(tool.model() == model);
    tool.clear();
    assert!(tool.data().is_null());
}

#[test]
fn analysis_failures_become_reports() {
    let (_, model) = read(&[("a", &[]), ("crash", &[]), ("c", &[1])]);
    assert_eq!(model.nb_entities(), 3);
    assert!(model.is_error_entity(2));
    assert!(model.report_entity(2, false).has_new_content());
}

#[test]
fn accessors() {
    let tool = FakeReaderTool::default();
    assert!(tool.error_handle() && tool.trace_level() == 0 && tool.messenger().is_some());
    tool.set_trace_level(2);
    tool.set_error_handle(false);
    tool.set_messenger(None);
    assert!(!tool.error_handle() && tool.trace_level() == 2);
    assert_eq!(
        tool.unknown_entity().err().unwrap().exception_type(),
        "Standard_NullObject"
    );
    tool.set_data(&fake_file(&[]), &fake_protocol());
    assert!(tool.new_model().unwrap().is_some());
    let model = fake_model();
    tool.set_model(&model);
    assert!(tool.model() == model);
}
