//! Message output operations (twin of `tools/oracle/tkernel_message_output_probe.cxx`).
use super::report::MsgProbePrinter;
use super::{bit, bytes, ext, hex, rest, units};
use rustycad_tkernel::message::{
    Algorithm, ConsoleColor, ExecStatus, Gravity, Messenger, Msg, MsgFile, Printer, PrinterHandle,
    PrinterOStream, PrinterToReport, PrinterType, Status,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeType, RuntimeTyped};
use rustycad_tkernel::tcollection::AsciiString;
use std::str::SplitWhitespace;
use std::sync::Arc;

/// The output of an output operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "algo_run" => algo_run(input),
        "messenger_ops" => messenger_ops(input),
        "printer_file" => printer_file(input),
        "printer_badfile" => {
            let path = String::from_utf8(bytes(input.next().expect("path"))).expect("UTF-8 path");
            let printer = PrinterOStream::from_file_name(&path, false, Gravity::Info);
            bit(printer.to_colorize()).to_string()
        }
        "printer_console" => {
            let color: usize = input.next().expect("color").parse().expect("int");
            let intense = input.next().expect("intense") != "0";
            let mut stream: Vec<u8> = Vec::new();
            PrinterOStream::set_console_text_color(
                Some(&mut stream),
                CONSOLE_COLORS[color],
                intense,
            );
            hex(&stream)
        }
        _ => return None,
    })
}

const CONSOLE_COLORS: [ConsoleColor; 9] = [
    ConsoleColor::Default,
    ConsoleColor::Black,
    ConsoleColor::White,
    ConsoleColor::Red,
    ConsoleColor::Blue,
    ConsoleColor::Green,
    ConsoleColor::Yellow,
    ConsoleColor::Cyan,
    ConsoleColor::Magenta,
];

/// The two algorithms of `algo_run` sharing one collecting messenger.
struct Algorithms {
    main: Algorithm,
    other: Algorithm,
    printer: Arc<MsgProbePrinter>,
}
impl Algorithms {
    fn new() -> Self {
        let printer = Arc::new(MsgProbePrinter::default());
        printer.set_trace_level(Gravity::Trace);
        let messenger = Arc::new(Messenger::with_printer(printer.clone()));
        let mut main = Algorithm::new();
        let mut other = Algorithm::new();
        main.set_messenger(Some(messenger.clone()));
        other.set_messenger(Some(messenger));
        Self {
            main,
            other,
            printer,
        }
    }
}

fn status(field: &str) -> Status {
    Status(field.parse().expect("status"))
}

/// Status-setting steps shared by both algorithms; false for other steps.
fn algo_set(algorithm: &mut Algorithm, step: &[&str]) -> bool {
    let no_repetitions = step.get(3) == Some(&"1");
    match step[0] {
        "st" => algorithm.set_status(status(step[1])),
        "si" => algorithm.set_status_with_integer(status(step[1]), step[2].parse().expect("int")),
        "ss" => algorithm.set_status_with_c_str(status(step[1]), &c_text(step[2]), no_repetitions),
        "sa" => algorithm.set_status_with_ascii(
            status(step[1]),
            &AsciiString::from_bytes(bytes(step[2])),
            no_repetitions,
        ),
        "se" => algorithm.set_status_with_extended(status(step[1]), &ext(step[2]), no_repetitions),
        "sn" => algorithm.set_status_with_h_extended(status(step[1]), &Handle::default(), true),
        "sm" => {
            let mut message = Msg::new();
            message.set(&ext(step[2]));
            algorithm.set_status_with_msg(status(step[1]), &message);
        }
        "clear" => algorithm.clear_status(),
        _ => return false,
    }
    true
}

/// `const char*` text: the bytes up to the first NUL.
fn c_text(token: &str) -> Vec<u8> {
    let text = bytes(token);
    text.split(|&byte| byte == 0)
        .next()
        .unwrap_or_default()
        .to_vec()
}

fn algo_query(algorithm: &Algorithm, step: &[&str]) -> String {
    let (status, max) = (status(step[1]), step[2].parse().expect("max"));
    if step[0] == "nums" {
        let report = algorithm
            .get_message_numbers(status)
            .map(|numbers| Algorithm::prepare_report_numbers(numbers, max));
        return format!(
            " N{}",
            report.map_or_else(|| "-".to_owned(), |text| units(&text))
        );
    }
    let report = algorithm
        .get_message_strings(status)
        .map(|strings| Algorithm::prepare_report_strings(strings, max));
    format!(
        " S{}",
        report.map_or_else(|| "-".to_owned(), |text| units(&text))
    )
}

fn algo_step(algorithms: &mut Algorithms, step: &[&str]) -> Result<String, Failure> {
    if algo_set(&mut algorithms.main, step) {
        return Ok(String::new());
    }
    let gravity = |field: &str| Gravity::ALL[field.parse::<usize>().expect("gravity")];
    match step[0] {
        other if other.starts_with('o') => {
            let mut renamed = step.to_vec();
            renamed[0] = &other[1..];
            algo_set(&mut algorithms.other, &renamed);
        }
        "cat" => {
            MsgFile::global().add_msg(&AsciiString::from_bytes(bytes(step[1])), &ext(step[2]));
        }
        "send" => algorithms
            .main
            .send_messages(gravity(step[1]), step[2].parse().expect("max"))?,
        "sendst" => {
            let filter = ExecStatus::from_status(status(step[1]));
            algorithms.main.send_status_messages(
                &filter,
                gravity(step[2]),
                step[3].parse().expect("max"),
            )?;
        }
        "add" => algorithms.main.add_status(&algorithms.other),
        "addf" => algorithms
            .main
            .add_status_filtered(&ExecStatus::from_status(status(step[1])), &algorithms.other),
        _ => return Ok(algo_query(&algorithms.main, step)),
    }
    Ok(String::new())
}

fn algo_run(input: &mut SplitWhitespace<'_>) -> String {
    let mut algorithms = Algorithms::new();
    let mut out = String::new();
    for step in rest(input) {
        let fields: Vec<&str> = step.split(':').collect();
        match algo_step(&mut algorithms, &fields) {
            Ok(text) => out.push_str(&text),
            Err(failure) => out.push_str(&format!(" E{}", failure.exception_type())),
        }
    }
    let log = algorithms.printer.log.lock().unwrap().clone();
    out.push_str(&format!(" L:{} ", hex(log.as_bytes())));
    for index in ExecStatus::FIRST_STATUS..ExecStatus::LAST_STATUS {
        out.push_str(
            &bit(algorithms
                .main
                .get_status()
                .is_set(ExecStatus::status_by_index(index)))
            .to_string(),
        );
    }
    out
}

fn printer_type(kind: u8) -> &'static RuntimeType {
    match kind {
        b'R' => PrinterToReport::static_type(),
        b'O' => PrinterOStream::static_type(),
        _ => PrinterType::static_type(),
    }
}

/// State of one `messenger_ops`.
struct MessengerOps {
    messenger: Messenger,
    collectors: Vec<Arc<MsgProbePrinter>>,
    last: Option<PrinterHandle>,
}

fn last_printer(ops: &MessengerOps) -> PrinterHandle {
    ops.last.clone().expect("a printer was added before")
}

fn add_printer(ops: &mut MessengerOps, printer: PrinterHandle) -> String {
    ops.last = Some(printer.clone());
    format!(" {}", bit(ops.messenger.add_printer(printer)))
}

fn messenger_step(ops: &mut MessengerOps, step: &[&str]) -> String {
    match step[0] {
        "p" => {
            let collector = Arc::new(MsgProbePrinter::default());
            ops.collectors.push(collector.clone());
            add_printer(ops, collector)
        }
        "r" => add_printer(ops, Arc::new(PrinterToReport::new())),
        "o" => {
            let path = String::from_utf8(bytes(step[1])).expect("UTF-8 path");
            add_printer(
                ops,
                Arc::new(PrinterOStream::from_file_name(&path, false, Gravity::Info)),
            )
        }
        "dup" => format!(" {}", bit(ops.messenger.add_printer(last_printer(ops)))),
        "rm" => format!(" {}", bit(ops.messenger.remove_printer(&last_printer(ops)))),
        "rmt" => format!(
            " {}",
            ops.messenger
                .remove_printers(printer_type(step[1].as_bytes()[0]))
        ),
        "send" => {
            ops.messenger.send_c_str(
                &bytes(step[2]),
                Gravity::ALL[step[1].parse::<usize>().expect("gravity")],
            );
            String::new()
        }
        "trace" => {
            let level = Gravity::ALL[step[1].parse::<usize>().expect("gravity")];
            ops.collectors
                .iter()
                .for_each(|collector| collector.set_trace_level(level));
            String::new()
        }
        "json" => {
            let mut json = DumpStream::default();
            ops.messenger.dump_json(&mut json, -1);
            format!(" J{}", hex(json.bytes()))
        }
        _ => String::new(),
    }
}

fn messenger_ops(input: &mut SplitWhitespace<'_>) -> String {
    let messenger = Messenger::new();
    messenger.printers_mut().clear();
    let mut ops = MessengerOps {
        messenger,
        collectors: Vec::new(),
        last: None,
    };
    let mut out = String::new();
    for step in rest(input) {
        let fields: Vec<&str> = step.split(':').collect();
        out.push_str(&messenger_step(&mut ops, &fields));
    }
    out.push_str(&format!(" n{}", ops.messenger.printers().len()));
    for collector in &ops.collectors {
        out.push_str(&format!(
            " {}",
            hex(collector.log.lock().unwrap().as_bytes())
        ));
    }
    out
}

fn printer_file(input: &mut SplitWhitespace<'_>) -> String {
    let path = String::from_utf8(bytes(input.next().expect("path"))).expect("UTF-8 path");
    let prefill = bytes(input.next().expect("prefill"));
    let append = input.next().expect("append") != "0";
    let trace = Gravity::ALL[input
        .next()
        .expect("trace")
        .parse::<usize>()
        .expect("gravity")];
    let colorize = input.next().expect("colorize") != "0";
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent).expect("probe directory");
    }
    // Files outlive a probe run: start from known content so appending is reproducible.
    std::fs::write(&path, prefill).expect("probe file");
    let mut out = String::new();
    {
        let printer = PrinterOStream::from_file_name(&path, append, trace);
        out.push_str(&bit(printer.to_colorize()).to_string());
        printer.set_to_colorize(colorize);
        for step in rest(input) {
            let (gravity, text) = step.split_once(':').expect("gravity:text");
            let gravity = Gravity::ALL[gravity.parse::<usize>().expect("gravity")];
            printer.send_ascii(&AsciiString::from_bytes(bytes(text)), gravity);
        }
        printer.close();
        printer.send_ascii(&AsciiString::new("after close"), Gravity::Fail);
    }
    let content = std::fs::read(&path).unwrap_or_default();
    out.push_str(&format!(" {}", hex(&content)));
    out
}
