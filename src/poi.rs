//! Dizionario di traduzione dei POI.
//!
//! Per aggiungere un tipo basta inserire una riga in `POI_DICTIONARY` e i
//! relativi alias. Il parser confronta i valori senza distinguere maiuscole,
//! minuscole, trattini, underscore o spazi multipli.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
    Garmin,
    Suunto,
}

impl Vendor {
    pub fn name(self) -> &'static str {
        match self {
            Self::Garmin => "garmin",
            Self::Suunto => "suunto",
        }
    }

    pub(crate) fn element(self) -> &'static str {
        match self {
            Self::Garmin => "sym",
            Self::Suunto => "type",
        }
    }
}

#[derive(Debug)]
struct PoiTranslation {
    /// Nome interno stabile, utile per leggere e ampliare il dizionario.
    #[allow(dead_code)]
    canonical: &'static str,
    garmin: &'static str,
    suunto: &'static str,
    aliases: &'static [&'static str],
}

// Garmin usa il testo di <sym>; Suunto usa il testo di <type>.
// I valori Garmin qui presenti appartengono al vocabolario storico
// MapSource/BaseCamp, mentre i valori Suunto sono quelli documentati nei
// manuali dei dispositivi. La tabella e' volutamente piccola ed espandibile.
static POI_DICTIONARY: &[PoiTranslation] = &[
    PoiTranslation {
        canonical: "waypoint",
        garmin: "Waypoint",
        suunto: "Waypoint",
        aliases: &[
            "waypoint",
            "generic",
            "generic point",
            "generic point of interest",
            "basic",
            "poi",
        ],
    },
    PoiTranslation {
        canonical: "drinking_water",
        garmin: "Drinking Water",
        suunto: "Water",
        aliases: &[
            "drinking water",
            "water",
            "water source",
            "water point",
            "waterpoint",
        ],
    },
    PoiTranslation {
        canonical: "peak",
        garmin: "Summit",
        suunto: "Peak",
        aliases: &["summit", "peak", "mountain top"],
    },
    PoiTranslation {
        canonical: "lodging",
        garmin: "Lodge",
        suunto: "Lodging",
        aliases: &[
            "lodge",
            "lodging",
            "hotel",
            "hostel",
            "alpine hut",
            "mountain hut",
        ],
    },
    PoiTranslation {
        canonical: "food",
        garmin: "Restaurant",
        suunto: "Food",
        aliases: &["food", "restaurant", "cafe", "bar"],
    },
    PoiTranslation {
        canonical: "camp",
        garmin: "Campground",
        suunto: "Camp",
        aliases: &["camp", "camping", "campground", "camping site"],
    },
    PoiTranslation {
        canonical: "parking",
        garmin: "Parking Area",
        suunto: "Parking",
        aliases: &["parking", "parking area", "car"],
    },
    PoiTranslation {
        canonical: "trail",
        garmin: "Trail Head",
        suunto: "Trail",
        aliases: &["trail", "trail head", "trailhead"],
    },
    PoiTranslation {
        canonical: "crossroad",
        garmin: "Crossing",
        suunto: "Crossroads",
        aliases: &["crossing", "crossroad", "crossroads"],
    },
    PoiTranslation {
        canonical: "geocache",
        garmin: "Geocache",
        suunto: "Geocache",
        aliases: &["geocache", "geocaching"],
    },
    PoiTranslation {
        canonical: "forest",
        garmin: "Forest",
        suunto: "Forest",
        aliases: &["forest", "wood", "woods"],
    },
    PoiTranslation {
        canonical: "rock",
        garmin: "Rock",
        suunto: "Rock",
        aliases: &["rock", "boulder"],
    },
    PoiTranslation {
        canonical: "building",
        garmin: "Building",
        suunto: "Building",
        aliases: &["building"],
    },
    PoiTranslation {
        canonical: "home",
        garmin: "Residence",
        suunto: "Home",
        aliases: &["home", "residence"],
    },
    PoiTranslation {
        canonical: "information",
        garmin: "Information",
        suunto: "Info",
        aliases: &["information", "info"],
    },
    PoiTranslation {
        canonical: "sight",
        garmin: "Scenic Area",
        suunto: "Sight",
        aliases: &["sight", "scenic area", "viewpoint"],
    },
];

pub(crate) struct Translation<'a> {
    pub value: &'static str,
    pub source: Option<&'a str>,
    pub used_fallback: bool,
}

pub(crate) fn translate<'a>(
    symbol: Option<&'a str>,
    poi_type: Option<&'a str>,
    target: Vendor,
) -> Translation<'a> {
    // Il campo nativo della destinazione ha precedenza: rende idempotente una
    // seconda conversione dello stesso file. Poi proviamo il campo dell'altro
    // vendor.
    let candidates = match target {
        Vendor::Garmin => [symbol, poi_type],
        Vendor::Suunto => [poi_type, symbol],
    };

    for source in candidates.into_iter().flatten() {
        if let Some(entry) = find(source) {
            let value = match target {
                Vendor::Garmin => entry.garmin,
                Vendor::Suunto => entry.suunto,
            };
            return Translation {
                value,
                source: Some(source),
                used_fallback: false,
            };
        }
    }

    Translation {
        value: "Waypoint",
        source: symbol.or(poi_type),
        used_fallback: symbol.is_some() || poi_type.is_some(),
    }
}

fn find(value: &str) -> Option<&'static PoiTranslation> {
    let normalized = normalize(value);
    POI_DICTIONARY.iter().find(|entry| {
        normalize(entry.garmin) == normalized
            || normalize(entry.suunto) == normalized
            || entry
                .aliases
                .iter()
                .any(|alias| normalize(alias) == normalized)
    })
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| match character {
            '_' | '-' | '/' => ' ',
            other => other.to_ascii_lowercase(),
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_both_vendor_vocabularies() {
        assert_eq!(
            translate(Some("Drinking Water"), None, Vendor::Suunto).value,
            "Water"
        );
        assert_eq!(
            translate(None, Some("Peak"), Vendor::Garmin).value,
            "Summit"
        );
        assert_eq!(
            translate(Some("LODGE"), None, Vendor::Suunto).value,
            "Lodging"
        );
    }

    #[test]
    fn accepts_normalized_aliases() {
        assert_eq!(
            translate(Some("alpine_hut"), None, Vendor::Suunto).value,
            "Lodging"
        );
    }
}
