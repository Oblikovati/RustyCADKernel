//! Unit tests of `Interface_InterfaceModel`, `Interface_GTool`, `Interface_ReportEntity`
//! and the general library, on the fake norm.
use super::test_norm::{
    FakeEntity, FakeModule, FakeProtocol, fake_chain, fake_entity, fake_model, fake_protocol,
};
use super::*;
use crate::lib_ctl::LibCtlRegistry;
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardTransient};

#[test]
fn numbering_values_and_types() {
    let (model, ents) = fake_chain();
    assert_eq!(model.nb_entities(), 4);
    assert_eq!(model.number(&ents[2]), 3);
    assert!(model.value(4).unwrap() == ents[3]);
    assert_eq!(
        model.value(5).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert!(model.contains(&ents[0]));
    assert!(!model.contains(&fake_entity("x", &[])));
    assert_eq!(model.type_name(&ents[0], true), "Entity");
    assert_eq!(model.nb_types(&ents[0]), 1);
    assert_eq!(model.entity_type(&ents[0], 1).name(), "Fake_Entity");
    assert!(model.protocol().is_some());
    assert_eq!(InterfaceSignTypeBase::class_name("Fake_Entity"), "Entity");
}

#[test]
fn add_with_refs_takes_the_shared_entities() {
    let (_, ents) = fake_chain();
    let model = fake_model();
    model.add_with_refs(&ents[3], 0, false).unwrap();
    assert_eq!(model.nb_entities(), 4);
    assert!(model.value(1).unwrap() == ents[3]);
    let shallow = fake_model();
    shallow
        .add_with_refs_protocol(&ents[3], &fake_protocol(), 2, false)
        .unwrap();
    assert_eq!(shallow.nb_entities(), 2);
    let bare: Handle<dyn InterfaceInterfaceModel> =
        Handle::new(super::test_norm::FakeModel::default()).cast();
    let error = bare.add_with_refs(&ents[0], 0, false).err().unwrap();
    assert_eq!(error.exception_type(), "Interface_InterfaceMismatch");
}

#[test]
fn reports_and_states() {
    let (model, ents) = fake_chain();
    let check = Handle::new(InterfaceCheck::new());
    check.add_fail_text("load failed", "");
    let report = Handle::new(InterfaceReportEntity::with_check(&check, &ents[1]));
    assert!(model.set_report_entity(2, &report).unwrap());
    assert_eq!(model.entity_state(2), InterfaceDataState::LoadFail);
    assert!(model.is_error_entity(2) && model.is_report_entity(2, false));
    assert!(model.check(2, true) == check);
    assert_eq!(model.check(3, true).nb_fails(), 0);
    let other = Handle::new(InterfaceReportEntity::with_check(
        &Handle::new(InterfaceCheck::new()),
        &ents[0],
    ));
    let mismatch = model.set_report_entity(2, &other).err().unwrap();
    assert_eq!(mismatch.exception_type(), "Interface_InterfaceMismatch");
    let content = fake_entity("new", &[]);
    report.set_content(&content);
    assert_eq!(model.entity_state(2), InterfaceDataState::StateUnloaded);
    assert_eq!(model.redefineds().nb_entities(), 1);
    assert!(model.clear_report_entity(2));
    assert_eq!(model.entity_state(2), InterfaceDataState::StateOK);
}

#[test]
fn semantic_checks_and_global_check() {
    let (model, ents) = fake_chain();
    let mut checks = InterfaceCheckIterator::with_name("semantic");
    checks.set_model(&model);
    let warn = Handle::new(InterfaceCheck::with_entity(&ents[0]));
    warn.add_warning_text("w", "");
    checks.add(&warn, 0);
    let global = Handle::new(InterfaceCheck::new());
    global.add_fail_text("g", "");
    checks.add(&global, 0);
    model.fill_semantic_checks(&checks, true).unwrap();
    assert!(model.has_semantic_checks());
    assert_eq!(model.entity_state(1), InterfaceDataState::DataWarning);
    assert_eq!(model.global_check(false).nb_fails(), 1);
    assert_eq!(model.reports(true).nb_entities(), 1);
    let replacement = Handle::new(InterfaceCheck::new());
    model.set_global_check(&replacement);
    assert!(model.global_check(true) == replacement);
}

#[test]
fn reorders_and_categories() {
    let (model, ents) = fake_chain();
    model.reverse_orders(1).unwrap();
    assert!(model.value(2).unwrap() == ents[3] && model.value(4).unwrap() == ents[1]);
    model.change_order(4, 1, 1).unwrap();
    assert!(model.value(1).unwrap() == ents[1]);
    assert!(model.change_order(1, 3, 3).is_err());
    assert!(model.set_category_number(2, 5).unwrap());
    assert_eq!(model.category_number(2), 5);
    assert_eq!(model.category_number(1), 0);
    assert!(!model.set_category_number(9, 1).unwrap());
    model.replace_entity(1, &fake_entity("z", &[])).unwrap();
    assert_eq!(model.number(&ents[1]), 0);
    let mut out = Vec::new();
    model.print(&ents[2], &mut out, 0).unwrap();
    model.print(&Handle::default(), &mut out, 0).unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        format!("{}:#{}NULL", model.number(&ents[2]), model.number(&ents[2]))
    );
}

#[test]
fn labels_templates_and_clear() {
    let (model, _) = fake_chain();
    assert_eq!(model.next_number_for_label("C", 0, true), 3);
    assert_eq!(model.next_number_for_label("x", 0, true), 0);
    assert_eq!(model.next_number_for_label("4", 0, false), 4);
    model
        .cast::<super::test_norm::FakeModel>()
        .header
        .write()
        .push_str("HEAD");
    assert!(InterfaceModelBase::set_template("fake-template", &model));
    assert!(InterfaceModelBase::has_template("fake-template"));
    let copy = InterfaceModelBase::template("fake-template").unwrap();
    assert_eq!(
        copy.cast::<super::test_norm::FakeModel>()
            .header
            .read()
            .as_str(),
        "HEAD"
    );
    assert!(
        InterfaceModelBase::list_templates()
            .iter()
            .any(|n| n.string().as_bytes() == b"fake-template")
    );
    assert!(InterfaceModelBase::template("none").unwrap().is_null());
    model.clear();
    assert_eq!(model.nb_entities(), 0);
    let entities = model.entities();
    assert_eq!(entities.nb_entities(), 0);
}

#[test]
fn gtool_caches_the_module_by_type() {
    let gtool = InterfaceGTool::with_protocol(&fake_protocol(), 10);
    let entity = fake_entity("a", &[]);
    let (module, cn) = gtool.select(&entity, false).unwrap();
    assert_eq!(cn, 1);
    assert!(module.cast::<FakeModule>().is_some());
    assert!(gtool.select(&entity, true).is_some());
    assert!(
        gtool
            .select(&Handle::new(StandardTransient::default()), false)
            .is_none()
    );
    gtool.clear_entities();
    gtool.reservate(5, true);
    assert_eq!(gtool.sign_name(), "Class Name");
    assert_eq!(gtool.sign_value(&entity, &Handle::default()), "Entity");
    gtool.set_protocol(&fake_protocol(), true);
    assert!(gtool.lib().select(&entity).is_some());
}

#[test]
fn private_registry_libraries() {
    let registry = LibCtlRegistry::<InterfaceGeneralLibKind>::new();
    let protocol: Handle<dyn InterfaceProtocol> = Handle::new(FakeProtocol::default()).cast();
    let module: Handle<dyn InterfaceGeneralModule> = Handle::new(FakeModule::default()).cast();
    let empty = InterfaceGeneralLib::new_in(&registry, &protocol);
    assert!(empty.select(&fake_entity("a", &[])).is_none());
    registry.set_global(&module, &protocol);
    registry.set_global(&module, &protocol);
    let mut lib = InterfaceGeneralLib::new_in(&registry, &protocol);
    let (found, cn) = lib.select(&fake_entity("a", &[])).unwrap();
    assert!(found == module && cn == 1);
    lib.start();
    assert!(lib.more() && lib.module().unwrap() == module && lib.protocol().unwrap() == protocol);
    lib.next();
    assert!(!lib.more());
    assert_eq!(
        lib.module().err().unwrap().exception_type(),
        "Standard_NoSuchObject"
    );
    let mut complete = InterfaceGeneralLib::empty();
    complete.set_complete_in(&registry);
    assert!(complete.select(&fake_entity("b", &[])).is_some());
    complete.clear();
    assert!(complete.select(&fake_entity("b", &[])).is_none());
    assert!(FakeEntity::static_type().name() == "Fake_Entity");
}
