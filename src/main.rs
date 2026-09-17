use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use universal_gpx_poi::{Vendor, convert_gpx};

fn main() {
    if let Err(error) = run() {
        eprintln!("Errore: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments
        .iter()
        .any(|argument| argument == "-h" || argument == "--help")
    {
        print_help();
        return Ok(());
    }

    let input_path = arguments
        .first()
        .map(PathBuf::from)
        .ok_or("manca il file GPX di input (usa --help per un esempio)")?;

    if arguments.len() > 1 {
        return Err(
            "sono accettati soltanto il percorso del file GPX e, facoltativamente, --help".into(),
        );
    }
    if input_path
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("gpx")
    {
        return Err("il file di input deve avere estensione .gpx".into());
    }

    let target = ask_vendor()?;
    let input = fs::read_to_string(&input_path)?;
    let (output, report) = convert_gpx(&input, target)?;
    let output_path = output_path_for(&input_path, target);

    // `create_new` combina controllo e creazione in un'unica operazione del
    // filesystem, evitando che un altro processo possa creare e farci
    // sovrascrivere il file fra una chiamata a `exists` e la scrittura.
    let mut output_file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output_path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(format!(
                "il file di output '{}' esiste gia'; rinominalo o rimuovilo prima di riprovare",
                output_path.display()
            )
            .into());
        }
        Err(error) => return Err(error.into()),
    };
    output_file.write_all(output.as_bytes())?;

    println!(
        "Creato {} ({} waypoint, {} tradotti, {} senza tipo esplicito o gia' generici).",
        output_path.display(),
        report.waypoints,
        report.translated,
        report.waypoints - report.translated - report.fallback_waypoints,
    );
    if !report.unknown_values.is_empty() {
        let fallback = match target {
            Vendor::Garmin => "WAYPOINT",
            Vendor::Suunto => "POI",
        };
        eprintln!(
            "Attenzione: valori non presenti nel dizionario, convertiti in {fallback}: {}",
            report
                .unknown_values
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    Ok(())
}

fn ask_vendor() -> Result<Vendor, Box<dyn Error>> {
    println!("In quale formato vuoi tradurre il GPX?");
    println!("  1) Suunto");
    println!("  2) Garmin");
    print!("Scelta: ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    match answer.trim().to_ascii_lowercase().as_str() {
        "1" | "s" | "suunto" => Ok(Vendor::Suunto),
        "2" | "g" | "garmin" => Ok(Vendor::Garmin),
        _ => Err("formato non valido: scegli 1/Suunto oppure 2/Garmin".into()),
    }
}

fn output_path_for(input: &Path, target: Vendor) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("converted");
    input.with_file_name(format!("{stem}-{}.gpx", target.name()))
}

fn print_help() {
    println!("universal_gpx_poi - converte i POI di un file GPX per Suunto o Garmin");
    println!();
    println!("Uso:");
    println!("  universal_gpx_poi <FILE.gpx>");
    println!();
    println!("Dopo aver letto il file, il programma chiede il formato di destinazione.");
    println!("Il file originale non viene modificato.");
}
