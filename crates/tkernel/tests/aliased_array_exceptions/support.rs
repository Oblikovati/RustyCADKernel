use rustycad_tkernel::standard::Failure;
pub(super) fn record<T>(rows: &mut Vec<String>, name: &str, result: Result<T, Failure>) {
    let suffix = match result {
        Ok(_) => "OK".to_owned(),
        Err(error) => format!(
            "{}|{}|{}",
            error.exception_type(),
            error.message(),
            usize::from(!error.stack_string().is_empty())
        ),
    };
    rows.push(format!("{name}|{suffix}"));
}
