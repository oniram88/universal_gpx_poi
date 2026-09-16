mod poi;

pub use poi::Vendor;

use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

use quick_xml::escape::unescape;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConversionReport {
    pub waypoints: usize,
    pub translated: usize,
    pub fallback_waypoints: usize,
    pub unknown_values: BTreeSet<String>,
}

/// Risultato esposto alla pagina web tramite WebAssembly.
///
/// I getter clonano soltanto le due stringhe richieste da JavaScript; il core
/// di conversione resta lo stesso usato dalla CLI.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WebConversion {
    output: String,
    waypoints: usize,
    translated: usize,
    fallback_waypoints: usize,
    unknown_values: String,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebConversion {
    #[wasm_bindgen(getter)]
    pub fn output(&self) -> String {
        self.output.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn waypoints(&self) -> usize {
        self.waypoints
    }

    #[wasm_bindgen(getter)]
    pub fn translated(&self) -> usize {
        self.translated
    }

    #[wasm_bindgen(getter)]
    pub fn fallback_waypoints(&self) -> usize {
        self.fallback_waypoints
    }

    #[wasm_bindgen(getter)]
    pub fn unknown_values(&self) -> String {
        self.unknown_values.clone()
    }
}

/// Binding minimale per il browser. `target` accetta `garmin` o `suunto`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn convert_gpx_for_web(input: &str, target: &str) -> Result<WebConversion, JsValue> {
    let vendor = match target.trim().to_ascii_lowercase().as_str() {
        "garmin" => Vendor::Garmin,
        "suunto" => Vendor::Suunto,
        _ => return Err(JsValue::from_str("Destinazione non valida")),
    };

    let (output, report) =
        convert_gpx(input, vendor).map_err(|error| JsValue::from_str(&error.to_string()))?;

    Ok(WebConversion {
        output,
        waypoints: report.waypoints,
        translated: report.translated,
        fallback_waypoints: report.fallback_waypoints,
        unknown_values: report
            .unknown_values
            .into_iter()
            .collect::<Vec<_>>()
            .join(", "),
    })
}

#[derive(Debug)]
pub enum ConversionError {
    InvalidXml(quick_xml::Error),
    Write(std::io::Error),
    InvalidText(quick_xml::escape::EscapeError),
}

impl Display for ConversionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidXml(error) => write!(formatter, "GPX/XML non valido: {error}"),
            Self::Write(error) => write!(formatter, "impossibile generare il GPX: {error}"),
            Self::InvalidText(error) => write!(formatter, "entita' XML non valida: {error}"),
        }
    }
}

impl std::error::Error for ConversionError {}

/// Converte soltanto i waypoint (`wpt`), lasciando inalterati tracce, route ed
/// estensioni vendor-specific. Il file risultante rimane GPX 1.1 valido.
pub fn convert_gpx(
    input: &str,
    target: Vendor,
) -> Result<(String, ConversionReport), ConversionError> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::with_capacity(input.len() + 256));
    let mut report = ConversionReport::default();
    let mut depth = 0usize;

    loop {
        let event = reader
            .read_event()
            .map_err(ConversionError::InvalidXml)?
            .into_owned();

        match event {
            Event::Start(start) if depth == 1 && is_local_name(&start, "wpt") => {
                let waypoint = read_element(&mut reader, start)?;
                let converted = convert_waypoint(waypoint, target, &mut report)?;
                for event in converted {
                    writer.write_event(event).map_err(ConversionError::Write)?;
                }
            }
            Event::Empty(start) if depth == 1 && is_local_name(&start, "wpt") => {
                let qualified_name = start.name().as_ref().to_owned();
                let waypoint = vec![
                    Event::Start(start),
                    Event::End(BytesEnd::new(&qualified_name).into_owned()),
                ];
                let converted = convert_waypoint(waypoint, target, &mut report)?;
                for event in converted {
                    writer.write_event(event).map_err(ConversionError::Write)?;
                }
            }
            Event::Eof => break,
            Event::Start(start) => {
                depth += 1;
                writer
                    .write_event(Event::Start(start))
                    .map_err(ConversionError::Write)?;
            }
            Event::End(end) => {
                depth = depth.saturating_sub(1);
                writer
                    .write_event(Event::End(end))
                    .map_err(ConversionError::Write)?;
            }
            other => writer.write_event(other).map_err(ConversionError::Write)?,
        }
    }

    // Gli eventi provengono da una &str UTF-8 e quelli aggiunti sono ASCII.
    let output = String::from_utf8(writer.into_inner())
        .expect("quick-xml ha prodotto byte UTF-8 da un input UTF-8");
    Ok((output, report))
}

fn read_element(
    reader: &mut Reader<&[u8]>,
    start: BytesStart<'static>,
) -> Result<Vec<Event<'static>>, ConversionError> {
    let mut events = vec![Event::Start(start)];
    let mut depth = 1usize;

    while depth > 0 {
        let event = reader
            .read_event()
            .map_err(ConversionError::InvalidXml)?
            .into_owned();
        match &event {
            Event::Start(_) => depth += 1,
            Event::End(_) => depth -= 1,
            Event::Eof => {
                return Err(ConversionError::InvalidXml(quick_xml::Error::IllFormed(
                    quick_xml::errors::IllFormedError::MissingEndTag("wpt".into()),
                )));
            }
            _ => {}
        }
        events.push(event);
    }

    Ok(events)
}

fn convert_waypoint(
    mut events: Vec<Event<'static>>,
    target: Vendor,
    report: &mut ConversionReport,
) -> Result<Vec<Event<'static>>, ConversionError> {
    let symbol = direct_child_text(&events, "sym")?;
    let poi_type = direct_child_text(&events, "type")?;
    let translation = poi::translate(symbol.as_deref(), poi_type.as_deref(), target);

    report.waypoints += 1;
    if translation.used_fallback {
        report.fallback_waypoints += 1;
        if let Some(source) = translation.source {
            report.unknown_values.insert(source.to_owned());
        }
    } else if translation.source.is_some() {
        report.translated += 1;
    }

    replace_or_insert_direct_child(&mut events, target.element(), translation.value);
    Ok(events)
}

fn direct_child_text(
    events: &[Event<'static>],
    wanted: &str,
) -> Result<Option<String>, ConversionError> {
    let mut depth = 0usize;
    let mut collecting = false;
    let mut result = String::new();

    for event in events.iter().skip(1) {
        match event {
            Event::Start(start) => {
                if depth == 0 && is_local_name(start, wanted) {
                    collecting = true;
                }
                depth += 1;
            }
            Event::Empty(start) if depth == 0 && is_local_name(start, wanted) => {
                return Ok(Some(String::new()));
            }
            Event::Text(text) if collecting && depth == 1 => {
                result.push_str(&unescape(text.as_ref()).map_err(ConversionError::InvalidText)?);
            }
            Event::CData(text) if collecting && depth == 1 => {
                result.push_str(text.as_ref());
            }
            Event::End(end) => {
                depth = depth.saturating_sub(1);
                if collecting && depth == 0 && end.local_name().as_ref() == wanted {
                    return Ok(Some(result));
                }
            }
            _ => {}
        }
    }

    Ok(None)
}

fn replace_or_insert_direct_child(
    events: &mut Vec<Event<'static>>,
    wanted: &str,
    value: &'static str,
) {
    let mut depth = 0usize;
    let mut index = 1usize;

    while index + 1 < events.len() {
        match &events[index] {
            Event::Start(start) if depth == 0 && is_local_name(start, wanted) => {
                let mut nested = 1usize;
                let mut end_index = index + 1;
                while end_index < events.len() && nested > 0 {
                    match &events[end_index] {
                        Event::Start(_) => nested += 1,
                        Event::End(_) => nested -= 1,
                        _ => {}
                    }
                    end_index += 1;
                }

                let end = events[end_index - 1].clone();
                events.splice(
                    index + 1..end_index,
                    [Event::Text(BytesText::new(value).into_owned()), end],
                );
                return;
            }
            Event::Empty(start) if depth == 0 && is_local_name(start, wanted) => {
                let qualified_name = start.name().as_ref().to_owned();
                events.splice(
                    index..=index,
                    [
                        Event::Start(BytesStart::new(&qualified_name).into_owned()),
                        Event::Text(BytesText::new(value).into_owned()),
                        Event::End(BytesEnd::new(&qualified_name).into_owned()),
                    ],
                );
                return;
            }
            Event::Start(_) => depth += 1,
            Event::End(_) => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    insert_direct_child(events, wanted, value);
}

fn insert_direct_child(events: &mut Vec<Event<'static>>, name: &str, value: &'static str) {
    let closing_index = events.len() - 1;
    let insertion_index = schema_insertion_index(events, name).unwrap_or_else(|| {
        if closing_index > 0 && is_whitespace(&events[closing_index - 1]) {
            closing_index - 1
        } else {
            closing_index
        }
    });

    let trailing_separator = (insertion_index == closing_index.saturating_sub(1))
        .then(|| match &events[insertion_index] {
            Event::Text(text) if text.as_ref().chars().all(char::is_whitespace) => {
                Some(text.as_ref().to_owned())
            }
            _ => None,
        })
        .flatten();
    let between_children_separator = trailing_separator
        .is_none()
        .then(|| {
            insertion_index
                .checked_sub(1)
                .and_then(|index| match &events[index] {
                    Event::Text(text) if text.as_ref().chars().all(char::is_whitespace) => {
                        Some(text.as_ref().to_owned())
                    }
                    _ => None,
                })
        })
        .flatten();

    let qualified_name = match &events[0] {
        Event::Start(start) => start
            .name()
            .as_ref()
            .rsplit_once(':')
            .map(|(prefix, _)| format!("{prefix}:{name}"))
            .unwrap_or_else(|| name.to_owned()),
        _ => name.to_owned(),
    };

    let mut inserted = Vec::new();
    if let Some(separator) = trailing_separator {
        inserted.push(Event::Text(
            BytesText::new(&format!("{separator}  ")).into_owned(),
        ));
    }
    inserted.extend([
        Event::Start(BytesStart::new(&qualified_name).into_owned()),
        Event::Text(BytesText::new(value).into_owned()),
        Event::End(BytesEnd::new(&qualified_name).into_owned()),
    ]);
    if let Some(separator) = between_children_separator {
        inserted.push(Event::Text(BytesText::new(&separator).into_owned()));
    }
    events.splice(insertion_index..insertion_index, inserted);
}

/// Restituisce il primo figlio che, secondo la sequenza di `wptType` in GPX
/// 1.1, deve stare dopo l'elemento da inserire.
fn schema_insertion_index(events: &[Event<'static>], target: &str) -> Option<usize> {
    let target_rank = child_rank(target)?;
    let mut depth = 0usize;

    for (index, event) in events.iter().enumerate().skip(1) {
        match event {
            Event::Start(start) if depth == 0 => {
                if child_rank(start.local_name().as_ref()).is_some_and(|rank| rank > target_rank) {
                    return Some(index);
                }
                depth += 1;
            }
            Event::Empty(start) if depth == 0 => {
                if child_rank(start.local_name().as_ref()).is_some_and(|rank| rank > target_rank) {
                    return Some(index);
                }
            }
            Event::Start(_) => depth += 1,
            Event::End(_) => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn child_rank(name: &str) -> Option<usize> {
    [
        "ele",
        "time",
        "magvar",
        "geoidheight",
        "name",
        "cmt",
        "desc",
        "src",
        "link",
        "sym",
        "type",
        "fix",
        "sat",
        "hdop",
        "vdop",
        "pdop",
        "ageofdgpsdata",
        "dgpsid",
        "extensions",
    ]
    .iter()
    .position(|candidate| *candidate == name)
}

fn is_whitespace(event: &Event<'_>) -> bool {
    matches!(event, Event::Text(text) if text.as_ref().chars().all(char::is_whitespace))
}

fn is_local_name(start: &BytesStart<'_>, wanted: &str) -> bool {
    start.local_name().as_ref() == wanted
}

#[cfg(test)]
mod tests {
    use super::*;

    const GPX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx xmlns="http://www.topografix.com/GPX/1/1" version="1.1" creator="test">
  <wpt lat="1" lon="2">
    <name>Fontana</name>
    <sym>Drinking Water</sym>
  </wpt>
  <wpt lat="3" lon="4">
    <name>Senza tipo</name>
  </wpt>
  <trk><trkseg><trkpt lat="1" lon="2"><type>non modificare</type></trkpt></trkseg></trk>
</gpx>"#;

    #[test]
    fn adds_suunto_type_without_touching_track_points() {
        let (converted, report) = convert_gpx(GPX, Vendor::Suunto).unwrap();
        assert!(converted.contains("<sym>Drinking Water</sym>"));
        assert!(converted.contains("<type>Water</type>"));
        assert!(converted.contains("<type>Waypoint</type>"));
        assert!(converted.contains("<trkpt lat=\"1\" lon=\"2\"><type>non modificare</type>"));
        assert_eq!(report.waypoints, 2);
        assert_eq!(report.translated, 1);
    }

    #[test]
    fn replaces_existing_target_field_and_is_idempotent() {
        let source =
            r#"<gpx><wpt lat="1" lon="2"><type>Peak</type><sym>Flag, Blue</sym></wpt></gpx>"#;
        let (once, report) = convert_gpx(source, Vendor::Garmin).unwrap();
        let (twice, _) = convert_gpx(&once, Vendor::Garmin).unwrap();
        assert!(once.contains("<sym>Summit</sym>"));
        assert_eq!(once, twice);
        assert_eq!(report.translated, 1);
    }

    #[test]
    fn unknown_values_fall_back_and_are_reported() {
        let source = r#"<gpx><wpt lat="1" lon="2"><sym>Alien base</sym></wpt></gpx>"#;
        let (converted, report) = convert_gpx(source, Vendor::Suunto).unwrap();
        assert!(converted.contains("<type>Waypoint</type>"));
        assert!(report.unknown_values.contains("Alien base"));
    }

    #[test]
    fn respects_gpx_child_order_and_namespace_prefix() {
        let source = r#"<g:gpx xmlns:g="http://www.topografix.com/GPX/1/1"><g:wpt lat="1" lon="2"><g:type>Peak</g:type><g:fix>3d</g:fix></g:wpt></g:gpx>"#;
        let (converted, _) = convert_gpx(source, Vendor::Garmin).unwrap();
        assert!(converted.contains("<g:sym>Summit</g:sym><g:type>Peak</g:type><g:fix>3d</g:fix>"));
    }

    #[test]
    fn ignores_wpt_elements_nested_in_extensions() {
        let source = r#"<gpx><extensions><wpt><sym>Summit</sym></wpt></extensions></gpx>"#;
        let (converted, report) = convert_gpx(source, Vendor::Suunto).unwrap();
        assert_eq!(converted, source);
        assert_eq!(report.waypoints, 0);
    }

    #[test]
    fn expands_and_converts_self_closing_waypoints() {
        let source =
            r#"<gpx><wpt lat="1" lon="2"/><g:wpt xmlns:g="urn:gpx" lat="3" lon="4"/></gpx>"#;
        let (converted, report) = convert_gpx(source, Vendor::Suunto).unwrap();
        assert!(converted.contains(r#"<wpt lat="1" lon="2"><type>Waypoint</type></wpt>"#));
        assert!(converted.contains(
            r#"<g:wpt xmlns:g="urn:gpx" lat="3" lon="4"><g:type>Waypoint</g:type></g:wpt>"#
        ));
        assert_eq!(report.waypoints, 2);
        assert_eq!(report.translated, 0);
        assert_eq!(report.fallback_waypoints, 0);
    }
}
