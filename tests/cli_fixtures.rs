use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("l'orologio di sistema deve essere successivo a UNIX_EPOCH")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "universal-gpx-poi-cli-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("impossibile creare la cartella temporanea del test");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cli_output_matches_stored_suunto_fixture() {
    assert_cli_output_matches_fixture("1\n", "suunto");
}

#[test]
fn cli_output_matches_stored_garmin_fixture() {
    assert_cli_output_matches_fixture("2\n", "garmin");
}

fn assert_cli_output_matches_fixture(answer: &str, vendor: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("test_data");
    let source = fixture_dir.join("super-baldo.gpx");
    let temporary_dir = TestDirectory::new();
    let temporary_source = temporary_dir.path().join("super-baldo.gpx");

    fs::copy(&source, &temporary_source)
        .expect("impossibile copiare il GPX sorgente nella cartella temporanea");

    run_cli(&temporary_source, answer);

    let generated = temporary_dir
        .path()
        .join(format!("super-baldo-{vendor}.gpx"));
    let expected = fixture_dir.join(format!("super-baldo-{vendor}.gpx"));
    assert_files_are_identical(&expected, &generated);
}

fn run_cli(input: &Path, answer: &str) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_universal_gpx_poi"))
        .arg(input)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("impossibile avviare la CLI");

    child
        .stdin
        .take()
        .expect("stdin della CLI non disponibile")
        .write_all(answer.as_bytes())
        .expect("impossibile inviare la scelta del vendor alla CLI");

    let output = child
        .wait_with_output()
        .expect("impossibile attendere la terminazione della CLI");
    assert!(
        output.status.success(),
        "la CLI e' terminata con {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn assert_files_are_identical(expected: &Path, generated: &Path) {
    let expected_bytes = format_xml(expected);
    let generated_bytes = format_xml(generated);

    if expected_bytes != generated_bytes {
        let first_difference = expected_bytes
            .iter()
            .zip(&generated_bytes)
            .position(|(expected, generated)| expected != generated)
            .unwrap_or(expected_bytes.len().min(generated_bytes.len()));
        let formatted_expected = generated.with_file_name("expected-formatted.gpx");
        let formatted_generated = generated.with_file_name("generated-formatted.gpx");
        fs::write(&formatted_expected, &expected_bytes).unwrap_or_else(|error| {
            panic!(
                "impossibile scrivere '{}': {error}",
                formatted_expected.display()
            )
        });
        fs::write(&formatted_generated, &generated_bytes).unwrap_or_else(|error| {
            panic!(
                "impossibile scrivere '{}': {error}",
                formatted_generated.display()
            )
        });
        let diff = unified_diff(
            expected,
            generated,
            &formatted_expected,
            &formatted_generated,
        );
        panic!(
            "'{}' non coincide con l'output della CLI '{}': prima differenza nell'XML formattato al byte {first_difference} (attesi {} byte, generati {} byte)\n\nDiff:\n{diff}",
            expected.display(),
            generated.display(),
            expected_bytes.len(),
            generated_bytes.len(),
        );
    }
}

fn format_xml(path: &Path) -> Vec<u8> {
    let input = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("impossibile leggere '{}': {error}", path.display()));
    let mut reader = Reader::from_str(&input);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        let event = reader
            .read_event()
            .unwrap_or_else(|error| panic!("XML non valido in '{}': {error}", path.display()));
        match event {
            Event::Eof => break,
            Event::Text(text) if text.as_ref().chars().all(char::is_whitespace) => {}
            event => writer.write_event(event).unwrap_or_else(|error| {
                panic!("impossibile formattare '{}': {error}", path.display())
            }),
        }
    }

    writer.into_inner()
}

fn unified_diff(
    expected_label: &Path,
    generated_label: &Path,
    expected: &Path,
    generated: &Path,
) -> String {
    match Command::new("diff")
        .arg("-u")
        .arg("-w")
        .arg("--label")
        .arg(expected_label)
        .arg("--label")
        .arg(generated_label)
        .arg(expected)
        .arg(generated)
        .output()
    {
        Ok(output) if !output.stdout.is_empty() => String::from_utf8_lossy(&output.stdout).into(),
        Ok(output) if !output.stderr.is_empty() => String::from_utf8_lossy(&output.stderr).into(),
        Ok(output) if output.status.success() => {
            "nessuna differenza oltre al whitespace".to_owned()
        }
        Ok(output) => format!("diff terminato con {} senza produrre output", output.status),
        Err(error) => format!("impossibile eseguire diff -u -w: {error}"),
    }
}
