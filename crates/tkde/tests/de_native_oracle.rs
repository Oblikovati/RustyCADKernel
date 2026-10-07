//! The DE port against the native oracle: `tests/data/tkde_native_oracle.txt` is the output
//! of tools/oracle/tkde_configuration_probe.cxx run against the native TKDE (resource
//! syntax, getters, shape-healing text, registry order, priorities, lookups, copies, content
//! buffers, validation, files, plugin holders); this test prints the same `key|value` lines
//! from the port and compares them line by line.
mod support;

use rustycad_tkde::de::{
    DeConfigurationContext, DeConfigurationNode, DeMultiPluginHolder, DePluginHolder, DeProvider,
    DeShapeFixConfigurationNode, DeWrapper,
};
use rustycad_tkernel::{ncollection::List, standard::Handle, tcollection::AsciiString};
use support::oracle_files::{content_cases, file_cases, validation_cases};
use support::oracle_lines::{Lines, b, esc, list, real, s};
use support::oracle_registry::{order, probe_value, provider, registry, unbind};
use support::probe_node::{PluginVendorA, PluginVendorB, ProbeNode, probe_node};

fn dump_context(lines: &mut Lines, key: &str, context: &DeConfigurationContext) {
    let map = context.get_internal_map();
    lines.line(&format!("{key}.extent"), map.extent());
    for (index, (name, value)) in map.items().enumerate() {
        lines.line(
            &format!("{key}.item{index}"),
            format!("{}={}", esc(name), esc(value)),
        );
    }
}

fn context_case(lines: &mut Lines, key: &str, resource: &str) {
    let context = DeConfigurationContext::new();
    match context.load_str(&s(resource)) {
        Ok(loaded) => lines.line(&format!("{key}.load"), b(loaded)),
        Err(failure) => lines.line(
            &format!("{key}.load"),
            format!("throw {}", failure.exception_type()),
        ),
    }
    dump_context(lines, key, &context);
}

fn context_cases(lines: &mut Lines) {
    context_case(lines, "ctx.simple", "a.b : 1\nc.d :\t two words \n");
    context_case(
        lines,
        "ctx.comment",
        "! comment\na : 1\n  ! not a comment\n",
    );
    context_case(lines, "ctx.nocolon", "novalue\nk : v\n");
    context_case(lines, "ctx.colonfirst", ": v\nk : v\n");
    context_case(lines, "ctx.empty.value", "k :\nl : \t \nm:\n");
    context_case(lines, "ctx.trailing.colon", "a : b:\nc:d:e\n");
    context_case(
        lines,
        "ctx.escape",
        "a : \\ x\nb : \\\\y\nc : \\tz\nd : \\q\n",
    );
    context_case(lines, "ctx.override", "a : 1\na : 2\n");
    context_case(lines, "ctx.crlf", "a : 1\r\nb : 2\r\n");
    context_case(lines, "ctx.blank", "a : 1\n   \nb : 2\n");
    context_case(lines, "ctx.noeol", "a : 1\nb : 2");
    context_case(lines, "ctx.spaces.key", "  key with space  :  v  \n");
    context_case(
        lines,
        "ctx.many",
        "z : 1\ny : 2\nx : 3\nw : 4\nv : 5\nu : 6\nt : 7\ns : 8\n",
    );
}

fn getter_case(lines: &mut Lines, context: &DeConfigurationContext, name: &str) {
    let key = format!("get.{name}");
    let (mut value_real, mut value_int, mut value_bool) = (-1.0, -1, false);
    lines.line(&format!("{key}.set"), b(context.is_param_set(name, "s")));
    let has_real = context.get_real(name, &mut value_real, "s");
    lines.line(
        &format!("{key}.real"),
        format!("{} {}", b(has_real), real(value_real)),
    );
    let has_int = context.get_integer(name, &mut value_int, "s");
    lines.line(&format!("{key}.int"), format!("{} {value_int}", b(has_int)));
    let has_bool = context.get_boolean(name, &mut value_bool, "s");
    lines.line(
        &format!("{key}.bool"),
        format!("{} {}", b(has_bool), b(value_bool)),
    );
    let mut sequence: List<AsciiString> = List::default();
    let has_sequence = context.get_string_seq(name, &mut sequence, "s");
    let items: String = sequence
        .iter()
        .map(|item| format!("[{}]", esc(item)))
        .collect();
    lines.line(
        &format!("{key}.seq"),
        format!("{} {items}", b(has_sequence)),
    );
    let values = format!(
        "{} {} {} {}",
        real(context.real_val(name, 2.5, "s")),
        context.integer_val(name, 5, "s"),
        b(context.boolean_val(name, true, "s")),
        esc(&context.string_val(name, &s("def"), "s"))
    );
    lines.line(&format!("{key}.vals"), values);
}

fn getter_cases(lines: &mut Lines) {
    let context = DeConfigurationContext::new();
    let resource = "s.r1 : 1e-3\ns.r2 : 12\ns.r3 : abc\ns.r4 : 1.5x\ns.i1 : 0x10\n\
s.i2 : 3.5\ns.i3 : -4\ns.b1 : 0\ns.b2 : 2\ns.b3 : yes\n\
s.q1 : a b  c\ns.q2 : OCC\ns.q3 :\nplain : 9\n";
    context.load_str(&s(resource)).expect("a valid resource");
    for name in [
        "r1", "r2", "r3", "r4", "i1", "i2", "i3", "b1", "b2", "b3", "q1", "q2", "q3", "zz",
    ] {
        getter_case(lines, &context, name);
    }
    lines.line("get.noscope", context.integer_val("plain", -1, ""));
}

fn shape_fix_cases(lines: &mut Lines) {
    let node = probe_node("TST", "VND", "tst", true, true, "");
    let object: &ProbeNode = node.get().expect("a node");
    lines.line("fix.default", esc(&object.save()));
    let context = Handle::new(DeConfigurationContext::new());
    let resource = "provider.TST.VND.healing.tolerance3d : 0.25\n\
provider.TST.VND.healing.free.shell : 5\nprovider.TST.VND.healing.vertex.tolerance : 0\n\
provider.TST.VND.healing.tail : x\nprovider.TST.VND.value : 42\n";
    context
        .get()
        .expect("a context")
        .load_str(&s(resource))
        .expect("a valid resource");
    lines.line("fix.load", b(object.load(&context).expect("a context")));
    lines.line("fix.loaded", esc(&object.save()));
    let copy = object.copy().cast::<ProbeNode>();
    lines.line("fix.copy.value", copy.get().expect("a copy").value());
    let extensions: String = [".TsT", "tst", "", ".", "..tst"]
        .iter()
        .map(|text| b(object.check_extension(&s(text))).to_string())
        .collect();
    lines.line("fix.ext", extensions);
    let flags = format!(
        "{}{}{}",
        b(object.is_enabled()),
        b(object.is_stream_supported()),
        b(object.update_load(true, true))
    );
    lines.line("fix.flags", flags);
}

fn wrapper_lookups(lines: &mut Lines, wrapper: &DeWrapper) {
    let mut found: Handle<dyn DeConfigurationNode> = Handle::default();
    let found_acme = wrapper.find(&s("STEP"), &s("ACME"), &mut found);
    let vendor = esc(&found.get().expect("a node").get_vendor());
    let found_none = wrapper.find(&s("STEP"), &s("NONE"), &mut found);
    let found_format = wrapper.find(&s("NONE"), &s("OCC"), &mut found);
    let last = esc(&found.get().expect("a node").get_vendor());
    let text = format!(
        "{} {vendor} {}{} {last}",
        b(found_acme),
        b(found_none),
        b(found_format)
    );
    lines.line("wrap.find", text);
    lines.line(
        "wrap.read.stp",
        provider(wrapper, "/tmp/no/such/file.stp", true),
    );
    lines.line(
        "wrap.write.stp",
        provider(wrapper, "/tmp/out/file.stp", false),
    );
    lines.line(
        "wrap.write.p21",
        provider(wrapper, "/tmp/out/file.p21", false),
    );
    lines.line(
        "wrap.write.ply",
        provider(wrapper, "/tmp/out/model.PLY", false),
    );
    lines.line(
        "wrap.write.obj",
        provider(wrapper, "/tmp/out/model.obj", false),
    );
    lines.line(
        "wrap.write.none",
        provider(wrapper, "/tmp/out/model", false),
    );
}

fn wrapper_priorities(lines: &mut Lines, wrapper: &DeWrapper) {
    let priority = list(&["ZETA", "NONE", "ACME"]);
    wrapper.change_priority(&s("STEP"), &priority, true);
    lines.line("wrap.priority", order(wrapper));
    lines.line(
        "wrap.write.stp2",
        provider(wrapper, "/tmp/out/file.stp", false),
    );
    wrapper.change_priority_all(&priority, false);
    lines.line("wrap.priority.all", order(wrapper));
    wrapper.update_load(true);
    lines.line("wrap.updateload", order(wrapper));
    let first = unbind(wrapper, "STEP", "ZETA");
    let second = unbind(wrapper, "STEP", "ZETA");
    let third = unbind(wrapper, "NONE", "OCC");
    lines.line("wrap.unbind", format!("{first}{second}{third}"));
    lines.line("wrap.after.unbind", order(wrapper));
}

fn wrapper_saves(lines: &mut Lines, wrapper: &DeWrapper) {
    let mut units = wrapper.global_parameters();
    (units.length_unit, units.system_unit) = (25.4, 0.001);
    wrapper.set_global_parameters(units);
    let (formats, vendors, none) = (list(&["STEP", "STL"]), list(&["OCC"]), List::default());
    lines.line("wrap.save.short", esc(&wrapper.save(false, &none, &none)));
    let small = DeWrapper::new();
    small.bind(&probe_node("STL", "OCC", "stl", true, true, "").cast());
    small.bind(&probe_node("BREP", "OCC", "brep", true, true, "").cast());
    lines.line("wrap.save.small", esc(&small.save(true, &none, &none)));
    lines.line(
        "wrap.save.filtered",
        esc(&wrapper.save(true, &formats, &vendors)),
    );
    let mut stl: Handle<dyn DeConfigurationNode> = Handle::default();
    wrapper.find(&s("STL"), &s("OCC"), &mut stl);
    let mut found: Handle<dyn DeProvider> = Handle::default();
    wrapper
        .find_provider(&s("a.stl"), false, &mut found)
        .expect("a lookup");
    let stl_object = stl.get().expect("a node");
    lines.line(
        "wrap.global.copied",
        real(stl_object.global_parameters().length_unit),
    );
    let node = found.get().expect("a provider").get_node();
    lines.line("wrap.provider.node", b(node == stl));
}

fn wrapper_cases(lines: &mut Lines) {
    let handle = registry();
    let wrapper = handle.get().expect("a wrapper");
    lines.line("wrap.order", order(wrapper));
    let rebind = wrapper.bind(&probe_node("STEP", "OCC", "x", false, false, "").cast());
    lines.line("wrap.rebind", b(rebind));
    lines.line("wrap.bindnull", b(wrapper.bind(&Handle::default())));
    wrapper_lookups(lines, wrapper);
    wrapper_priorities(lines, wrapper);
    wrapper_saves(lines, wrapper);
}

fn load_cases(lines: &mut Lines) {
    let handle = registry();
    let wrapper = handle.get().expect("a wrapper");
    let resource = "global.priority.STEP : ACME OCC\nglobal.priority.NONE : OCC\n\
global.general.length.unit : 10\nprovider.STEP.OCC.value : 3\n\
provider.STEP.ACME.healing.tolerance3d : 0.5\nprovider.VRML.OCC.value : 11\n";
    lines.line(
        "load.result",
        b(wrapper.load(&s(resource), true).expect("a valid resource")),
    );
    lines.line("load.order", order(wrapper));
    let step = probe_value(wrapper, "STEP", "OCC").cast::<ProbeNode>();
    lines.line("load.value", step.get().expect("a node").value());
    let acme = probe_value(wrapper, "STEP", "ACME").cast::<dyn DeShapeFixConfigurationNode>();
    let tolerance = acme
        .get()
        .expect("a node")
        .shape_fix_parameters()
        .tolerance3d;
    lines.line("load.tol", real(tolerance));
    lines.line("load.unit", real(wrapper.global_parameters().length_unit));
    let shallow_handle = registry();
    let shallow = shallow_handle.get().expect("a wrapper");
    let text = s("global.priority.STEP : ZETA\nprovider.STEP.OCC.value : 9\n");
    shallow.load(&text, false).expect("a valid resource");
    lines.line("load.shallow", order(shallow));
    let value = probe_value(shallow, "STEP", "OCC").cast::<ProbeNode>();
    lines.line("load.shallow.value", value.get().expect("a node").value());
}

fn copy_cases(lines: &mut Lines) {
    let handle = registry();
    let wrapper = handle.get().expect("a wrapper");
    wrapper.set_keep_updates(true);
    let copy_handle = wrapper.copy();
    let copy = copy_handle.get().expect("a copy");
    lines.line("copy.order", order(copy));
    lines.line(
        "copy.keep",
        format!("{}{}", b(copy.keep_updates()), b(wrapper.keep_updates())),
    );
    let shared = probe_value(wrapper, "STEP", "OCC") == probe_value(copy, "STEP", "OCC");
    lines.line("copy.shared", b(shared));
    let rebound = DeWrapper::from_wrapper(&handle);
    lines.line("copy.handle.order", order(&rebound));
    let keep = format!("{}{}", b(rebound.keep_updates()), b(wrapper.keep_updates()));
    lines.line("copy.handle.keep", keep);
    lines.line(
        "copy.null",
        order(&DeWrapper::from_wrapper(&Handle::default())),
    );
}

fn plugin_cases(lines: &mut Lines) {
    let isolated = Handle::new(DeWrapper::new());
    DeWrapper::set_global_wrapper(&isolated);
    DeWrapper::set_global_wrapper(&Handle::default());
    lines.line(
        "plugin.global.kept",
        b(DeWrapper::global_wrapper() == isolated),
    );
    let global = || order(DeWrapper::global_wrapper().get().expect("a global wrapper"));
    {
        let _multi =
            DeMultiPluginHolder::<(PluginVendorA, PluginVendorB)>::new().expect("a holder");
        lines.line("plugin.multi", global());
        {
            let _single = DePluginHolder::<PluginVendorA>::new().expect("a holder");
            lines.line("plugin.single", global());
        }
        lines.line("plugin.single.released", global());
    }
    lines.line("plugin.multi.released", global());
}

#[test]
fn the_port_prints_the_native_oracle_lines() {
    let mut lines = Lines::default();
    context_cases(&mut lines);
    getter_cases(&mut lines);
    shape_fix_cases(&mut lines);
    wrapper_cases(&mut lines);
    load_cases(&mut lines);
    copy_cases(&mut lines);
    content_cases(&mut lines);
    validation_cases(&mut lines);
    file_cases(&mut lines);
    plugin_cases(&mut lines);
    let expected: Vec<&str> = include_str!("data/tkde_native_oracle.txt")
        .lines()
        .collect();
    for (index, (actual, native)) in lines.0.iter().zip(&expected).enumerate() {
        assert_eq!(actual, native, "line {}", index + 1);
    }
    assert_eq!(lines.0.len(), expected.len());
}
