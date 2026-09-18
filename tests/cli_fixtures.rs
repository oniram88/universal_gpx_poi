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
fn cli_outputs_match_stored_suunto_and_garmin_fixtures() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("test_data");
    let source = fixture_dir.join("super-baldo.gpx");
    let temporary_dir = TestDirectory::new();
    let temporary_source = temporary_dir.path().join("super-baldo.gpx");

    fs::copy(&source, &temporary_source)
        .expect("impossibile copiare il GPX sorgente nella cartella temporanea");

    for (answer, vendor) in [("1\n", "suunto"), ("2\n", "garmin")] {
        run_cli(&temporary_source, answer);

        let generated = temporary_dir
            .path()
            .join(format!("super-baldo-{vendor}.gpx"));
        let expected = fixture_dir.join(format!("super-baldo-{vendor}.gpx"));
        assert_files_are_identical(&expected, &generated);
    }
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
    let expected_bytes = fs::read(expected)
        .unwrap_or_else(|error| panic!("impossibile leggere '{}': {error}", expected.display()));
    let generated_bytes = fs::read(generated)
        .unwrap_or_else(|error| panic!("impossibile leggere '{}': {error}", generated.display()));

    if expected_bytes != generated_bytes {
        let first_difference = expected_bytes
            .iter()
            .zip(&generated_bytes)
            .position(|(expected, generated)| expected != generated)
            .unwrap_or(expected_bytes.len().min(generated_bytes.len()));
        panic!(
            "'{}' non coincide con l'output della CLI '{}': prima differenza al byte {first_difference} (attesi {} byte, generati {} byte)",
            expected.display(),
            generated.display(),
            expected_bytes.len(),
            generated_bytes.len(),
        );
    }
}
