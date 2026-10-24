//! Native Windows comparison of truncating and staged document publication.
use martensite_tkernel::osd::{create_private_document_directory, preserve_document_access};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const ORIGINAL: &[u8] = b"original document bytes";
const REPLACEMENT: &[u8] = b"completed replacement document";
const ZONE: &[u8] = b"[ZoneTransfer]\r\nZoneId=3\r\nHostUrl=https://example.invalid/model\r\n";

fn stream_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(":Zone.Identifier");
    PathBuf::from(name)
}

fn seed(directory: &Path, name: &str) -> io::Result<PathBuf> {
    let destination = directory.join(name);
    fs::write(&destination, ORIGINAL)?;
    fs::write(stream_path(&destination), ZONE)?;
    assert_eq!(fs::read(stream_path(&destination))?, ZONE);
    Ok(destination)
}

fn capture(path: &Path) -> (bool, bool, bool) {
    let contents = fs::read(path).unwrap();
    let stream = fs::read(stream_path(path));
    (
        contents == ORIGINAL,
        contents == REPLACEMENT,
        matches!(stream, Ok(ref bytes) if bytes == ZONE),
    )
}

fn original_truncating_save(directory: &Path) -> io::Result<bool> {
    let destination = seed(directory, "truncate.bin")?;
    let mut output = File::create(&destination)?;
    output.write_all(REPLACEMENT)?;
    output.sync_all()?;
    drop(output);
    let (_, new_bytes, stream) = capture(&destination);
    println!(
        "{{\"operation\":\"File::create\",\"new_bytes\":{new_bytes},\"stream_preserved\":{stream}}}"
    );
    assert!(
        new_bytes && stream,
        "native truncation control must preserve the seeded stream"
    );
    Ok(stream)
}

fn bare_rename_save(directory: &Path) -> io::Result<()> {
    let destination = seed(directory, "rename.bin")?;
    let staged = directory.join("rename-stage.bin");
    fs::write(&staged, REPLACEMENT)?;
    fs::rename(&staged, &destination)?;
    let (_, new_bytes, stream) = capture(&destination);
    println!(
        "{{\"operation\":\"fs::rename\",\"new_bytes\":{new_bytes},\"stream_preserved\":{stream}}}"
    );
    assert!(new_bytes, "rename control must publish replacement bytes");
    Ok(())
}

fn candidate_save(directory: &Path) -> io::Result<bool> {
    let destination = seed(directory, "candidate.bin")?;
    let private = directory.join("candidate-stage");
    create_private_document_directory(&private)?;
    let staged = private.join("candidate.bin");
    fs::write(&staged, REPLACEMENT)?;
    let outcome = publish_candidate(&staged, &destination);
    let (old_bytes, new_bytes, stream) = capture(&destination);
    print_outcome(&outcome, old_bytes, new_bytes, stream);
    Ok(stream && ((outcome.is_ok() && new_bytes) || (outcome.is_err() && old_bytes)))
}

fn publish_candidate(staged: &Path, destination: &Path) -> io::Result<()> {
    // This is the candidate's exact CDF commit sequence after its driver completes.
    let completed = OpenOptions::new().write(true).open(staged)?;
    preserve_document_access(destination, staged)?;
    completed.sync_all()?;
    fs::rename(staged, destination)
}

fn print_outcome(outcome: &io::Result<()>, old_bytes: bool, new_bytes: bool, stream: bool) {
    match outcome {
        Ok(()) => println!(
            "{{\"operation\":\"candidate_facade_then_rename\",\"published\":true,\"old_bytes\":{old_bytes},\"new_bytes\":{new_bytes},\"stream_preserved\":{stream}}}"
        ),
        Err(error) => println!(
            "{{\"operation\":\"candidate_facade_then_rename\",\"published\":false,\"old_bytes\":{old_bytes},\"new_bytes\":{new_bytes},\"stream_preserved\":{stream},\"error_kind\":\"{:?}\",\"error_os_code\":{},\"error\":{:?}}}",
            error.kind(),
            error
                .raw_os_error()
                .map_or_else(|| "null".to_owned(), |code| code.to_string()),
            error.to_string()
        ),
    }
}

fn run(directory: &Path) -> io::Result<bool> {
    fs::create_dir_all(directory)?;
    original_truncating_save(directory)?;
    bare_rename_save(directory)?;
    candidate_save(directory)
}

fn main() -> io::Result<()> {
    assert!(cfg!(windows), "native Windows execution is required");
    let parent = std::env::args_os().nth(1).map(PathBuf::from).unwrap();
    let first = run(&parent.join("first"))?;
    let second = run(&parent.join("second"))?;
    println!(
        "{{\"candidate_preserves_stream_or_refuses_without_modification\":{},\"repetitions\":2}}",
        first && second
    );
    assert!(
        first && second,
        "candidate publication lost a destination Zone.Identifier stream"
    );
    Ok(())
}
