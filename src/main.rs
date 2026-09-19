use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
    let (output_path, mut output_file) = create_output_file(&input_path, target)?;
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

fn timestamped_output_path_for(
    input: &Path,
    target: Vendor,
    timestamp: u128,
    attempt: u32,
) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("converted");
    let collision_suffix = match attempt {
        0 => String::new(),
        attempt => format!("-{attempt}"),
    };
    input.with_file_name(format!(
        "{stem}-{}-{timestamp}{collision_suffix}.gpx",
        target.name()
    ))
}

fn create_output_file(input: &Path, target: Vendor) -> Result<(PathBuf, fs::File), Box<dyn Error>> {
    let default_path = output_path_for(input, target);
    match open_new_file(&default_path) {
        Ok(file) => return Ok((default_path, file)),
        Err(error) if error.kind() != io::ErrorKind::AlreadyExists => return Err(error.into()),
        Err(_) => {}
    }

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    for attempt in 0.. {
        let timestamped_path = timestamped_output_path_for(input, target, timestamp, attempt);
        match open_new_file(&timestamped_path) {
            Ok(file) => return Ok((timestamped_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }

    unreachable!("il contatore dei nomi di output non può esaurirsi")
}

fn open_new_file(path: &Path) -> io::Result<fs::File> {
    // `create_new` combina controllo e creazione in un'unica operazione del
    // filesystem, evitando sovrascritture anche con più processi concorrenti.
    OpenOptions::new().write(true).create_new(true).open(path)
}

fn print_help() {
    println!("universal_gpx_poi - converte i POI di un file GPX per Suunto o Garmin");
    println!();
    println!("Uso:");
    println!("  universal_gpx_poi <FILE.gpx>");
    println!();
    println!("Dopo aver letto il file, il programma chiede il formato di destinazione.");
    println!("Il file originale non viene modificato.");
    println!("Se il file di output esiste già, il nuovo nome include un timestamp.");
}
