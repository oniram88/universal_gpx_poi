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
    /// Valore testuale usato dall'elemento `<type>` dei file GPX.
    suunto: &'static str,
    aliases: &'static [&'static str],
}

// Garmin usa il testo di <sym>; Suunto usa il testo di <type>.
// I valori Garmin qui presenti appartengono al vocabolario storico
// MapSource/BaseCamp, mentre i nomi Suunto sono quelli documentati nei manuali
// dei dispositivi. Quando non esiste una corrispondenza Garmin affidabile viene
// usato il simbolo generico Waypoint.
macro_rules! poi {
    ($canonical:literal, $garmin:literal, $suunto:literal, [$($alias:literal),* $(,)?]) => {
        PoiTranslation {
            canonical: $canonical,
            garmin: $garmin,
            suunto: $suunto,
            aliases: &[$($alias),*],
        }
    };
}

static POI_DICTIONARY: &[PoiTranslation] = &[
    poi!(
        "poi",
        "Waypoint",
        "POI",
        [
            "waypoint",
            "generic",
            "generic point",
            "generic point of interest",
            "basic"
        ]
    ),
    poi!("unknown", "Waypoint", "Unknown", []),
    poi!("building", "Building", "Building", []),
    poi!("home", "Residence", "Home", []),
    poi!("car", "Car", "Car", []),
    poi!("parking", "Parking Area", "Parking", ["parking area"]),
    poi!("camp", "Campground", "Camp", []),
    poi!(
        "camping",
        "Campground",
        "Camping",
        ["campground", "camping site"]
    ),
    poi!(
        "food",
        "Picnic Area",
        "Food",
        ["bar", "picnic area", "picnic spot", "picnic site"]
    ),
    poi!("restaurant", "Restaurant", "Restaurant", []),
    poi!("cafe", "Restaurant", "Cafe", ["coffee shop"]),
    poi!(
        "lodging",
        "Lodge",
        "Lodging",
        ["lodge", "alpine hut", "mountain hut"]
    ),
    poi!("hostel", "Lodge", "Hostel", []),
    poi!("hotel", "Lodge", "Hotel", []),
    poi!(
        "water",
        "Drinking Water",
        "Water",
        ["drinking water", "water source"]
    ),
    poi!("river", "Waypoint", "River", []),
    poi!("lake", "Waypoint", "Lake", []),
    poi!("coast", "Waypoint", "Coast", []),
    poi!("mountain", "Summit", "Mountain", []),
    poi!("hill", "Summit", "Hill", []),
    poi!("valley", "Waypoint", "Valley", []),
    poi!("cliff", "Waypoint", "Cliff", []),
    poi!("forest", "Forest", "Forest", ["wood", "woods"]),
    poi!(
        "crossroads",
        "Crossing",
        "Crossroads",
        ["crossing", "crossroad"]
    ),
    poi!(
        "sight",
        "Scenic Area",
        "Sight",
        ["scenic area", "viewpoint"]
    ),
    poi!("begin", "Trail Head", "Begin", ["start"]),
    poi!("end", "Waypoint", "End", ["finish"]),
    poi!("geocache", "Geocache", "Geocache", ["geocaching"]),
    poi!("road", "Waypoint", "Road", []),
    poi!("trail", "Trail Head", "Trail", ["trail head", "trailhead"]),
    poi!("rock", "Rock", "Rock", ["boulder"]),
    poi!("meadow", "Waypoint", "Meadow", []),
    poi!("cave", "Waypoint", "Cave", []),
    poi!("emergency", "Medical Facility", "Emergency", ["sos"]),
    poi!("information", "Information", "Information", ["info"]),
    poi!("peak", "Summit", "Peak", ["summit", "mountain top"]),
    poi!("waterfall", "Waypoint", "Waterfall", []),
    poi!(
        "fishing_spot",
        "Fishing Area",
        "FishingSpot",
        ["fishing spot"]
    ),
    poi!("bedding", "Waypoint", "Bedding", []),
    poi!("prints", "Waypoint", "Prints", ["animal prints", "tracks"]),
    poi!("rub", "Waypoint", "Rub", []),
    poi!("scrape", "Waypoint", "Scrape", []),
    poi!("stand", "Waypoint", "Stand", []),
    poi!(
        "trail_cam",
        "Waypoint",
        "TrailCam",
        ["trail cam", "trail camera"]
    ),
    poi!("big_game", "Waypoint", "BigGame", ["big game"]),
    poi!("small_game", "Waypoint", "SmallGame", ["small game"]),
    poi!("bird", "Waypoint", "Bird", []),
    poi!("shot", "Waypoint", "Shot", []),
    poi!("fish", "Fishing Area", "Fish", []),
    poi!("big_fish", "Fishing Area", "BigFish", ["big fish"]),
    poi!("coral_reef", "Waypoint", "CoralReef", ["coral reef"]),
    poi!("beach", "Waypoint", "Beach", []),
    poi!(
        "marine_mammals",
        "Waypoint",
        "MarineMammals",
        ["marine mammals"]
    ),
    poi!("kelp_forest", "Waypoint", "KelpForest", ["kelp forest"]),
    poi!("lagoon", "Waypoint", "Lagoon", []),
    poi!("wreck", "Waypoint", "Wreck", ["shipwreck"]),
    poi!(
        "marine_reserve",
        "Waypoint",
        "MarineReserve",
        ["marine reserve"]
    ),
    poi!("avalanche", "Danger Area", "Avalanche", []),
    poi!("danger", "Danger Area", "Danger", ["hazard", "alert"]),
    poi!(
        "aid_station",
        "Medical Facility",
        "AidStation",
        ["aid station"]
    ),
    poi!(
        "water_point",
        "Drinking Water",
        "WaterPoint",
        ["water point", "waterpoint"]
    ),
    poi!("mushrooms", "Waypoint", "Mushrooms", ["mushroom"]),
    poi!("campfire", "Campground", "Campfire", ["camp fire"]),
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
        value: match target {
            Vendor::Garmin => "Waypoint",
            Vendor::Suunto => "POI",
        },
        source: symbol.or(poi_type),
        used_fallback: symbol.is_some() || poi_type.is_some(),
    }
}

fn find(value: &str) -> Option<&'static PoiTranslation> {
    let normalized = normalize(value);

    // I nomi Suunto hanno precedenza globale: alcuni coincidono con il valore
    // Garmin di un'altra voce (per esempio Restaurant) e devono restare
    // idempotenti quando il file e' gia' nel formato di destinazione.
    POI_DICTIONARY
        .iter()
        .find(|entry| normalize(entry.suunto) == normalized)
        .or_else(|| {
            POI_DICTIONARY
                .iter()
                .find(|entry| normalize(entry.garmin) == normalized)
        })
        .or_else(|| {
            POI_DICTIONARY.iter().find(|entry| {
                entry
                    .aliases
                    .iter()
                    .any(|alias| normalize(alias) == normalized)
            })
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

    #[test]
    fn contains_every_documented_suunto_type_once() {
        let expected = [
            "Unknown",
            "Building",
            "Home",
            "Car",
            "Parking",
            "Camp",
            "Camping",
            "Food",
            "Restaurant",
            "Cafe",
            "Lodging",
            "Hostel",
            "Hotel",
            "Water",
            "River",
            "Lake",
            "Coast",
            "Mountain",
            "Hill",
            "Valley",
            "Cliff",
            "Forest",
            "Crossroads",
            "Sight",
            "Begin",
            "End",
            "Geocache",
            "POI",
            "Road",
            "Trail",
            "Rock",
            "Meadow",
            "Cave",
            "Emergency",
            "Information",
            "Peak",
            "Waterfall",
            "FishingSpot",
            "Bedding",
            "Prints",
            "Rub",
            "Scrape",
            "Stand",
            "TrailCam",
            "BigGame",
            "SmallGame",
            "Bird",
            "Shot",
            "Fish",
            "BigFish",
            "CoralReef",
            "Beach",
            "MarineMammals",
            "KelpForest",
            "Lagoon",
            "Wreck",
            "MarineReserve",
            "Avalanche",
            "Danger",
            "AidStation",
            "WaterPoint",
            "Mushrooms",
            "Campfire",
        ];

        assert_eq!(POI_DICTIONARY.len(), expected.len());
        for name in expected {
            let matches = POI_DICTIONARY
                .iter()
                .filter(|entry| entry.suunto == name)
                .count();
            assert_eq!(matches, 1, "tipo Suunto mancante o duplicato: {name}");
            assert_eq!(translate(None, Some(name), Vendor::Suunto).value, name);
        }
    }
}
