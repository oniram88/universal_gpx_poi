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
        "type"
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

// Sia Garmin sia Suunto usano il testo di <type>. Continuiamo comunque a
// riconoscere <sym> in input per poter convertire i GPX esistenti.
// I valori Garmin qui presenti appartengono al vocabolario storico
// MapSource/BaseCamp, mentre i nomi Suunto sono quelli documentati nei manuali
// dei dispositivi. Quando non esiste una corrispondenza Garmin affidabile viene
// usato il tipo generico WAYPOINT.
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
        "WAYPOINT",
        "POI",
        [
            "waypoint",
            "generic",
            "generic point",
            "generic point of interest",
            "basic"
        ]
    ),
    poi!("unknown", "WAYPOINT", "Unknown", []),
    poi!("building", "BUILDING", "Building", []),
    poi!("home", "RESIDENCE", "Home", []),
    poi!("car", "CAR", "Car", []),
    poi!("parking", "PARKING AREA", "Parking", ["parking area"]),
    poi!("camp", "CAMPGROUND", "Camp", []),
    poi!(
        "camping",
        "CAMPGROUND",
        "Camping",
        ["campground", "camping site"]
    ),
    poi!(
        "food",
        "FOOD",
        "Food",
        ["bar", "picnic area", "picnic spot", "picnic site"]
    ),
    poi!("restaurant", "RESTAURANT", "Restaurant", []),
    poi!("cafe", "RESTAURANT", "Cafe", ["coffee shop"]),
    poi!(
        "lodging",
        "LODGE",
        "Lodging",
        ["lodge", "alpine hut", "mountain hut"]
    ),
    poi!("hostel", "LODGE", "Hostel", []),
    poi!("hotel", "LODGE", "Hotel", []),
    poi!(
        "water",
        "WATER",
        "Water",
        ["drinking water", "water source","water point", "waterpoint"]
    ),
    poi!("river", "WAYPOINT", "River", []),
    poi!("lake", "WAYPOINT", "Lake", []),
    poi!("coast", "WAYPOINT", "Coast", []),
    poi!("mountain", "SUMMIT", "Mountain", []),
    poi!("hill", "SUMMIT", "Hill", []),
    poi!("valley", "WAYPOINT", "Valley", []),
    poi!("cliff", "WAYPOINT", "Cliff", []),
    poi!("forest", "FOREST", "Forest", ["wood", "woods"]),
    poi!(
        "crossroads",
        "CROSSING",
        "Crossroads",
        ["crossing", "crossroad"]
    ),
    poi!(
        "sight",
        "SCENIC AREA",
        "Sight",
        ["scenic area", "viewpoint"]
    ),
    poi!("begin", "TRAIL HEAD", "Begin", ["start"]),
    poi!("end", "WAYPOINT", "End", ["finish"]),
    poi!("geocache", "GEOCACHE", "Geocache", ["geocaching"]),
    poi!("road", "WAYPOINT", "Road", []),
    poi!("trail", "TRAIL HEAD", "Trail", ["trail head", "trailhead"]),
    poi!("rock", "ROCK", "Rock", ["boulder"]),
    poi!("meadow", "WAYPOINT", "Meadow", []),
    poi!("cave", "WAYPOINT", "Cave", []),
    poi!("emergency", "MEDICAL FACILITY", "Emergency", ["sos"]),
    poi!("information", "INFORMATION", "Information", ["info"]),
    poi!("peak", "SUMMIT", "Peak", ["summit", "mountain top"]),
    poi!("waterfall", "WAYPOINT", "Waterfall", []),
    poi!(
        "fishing_spot",
        "FISHING AREA",
        "FishingSpot",
        ["fishing spot"]
    ),
    poi!("bedding", "WAYPOINT", "Bedding", []),
    poi!("prints", "WAYPOINT", "Prints", ["animal prints", "tracks"]),
    poi!("rub", "WAYPOINT", "Rub", []),
    poi!("scrape", "WAYPOINT", "Scrape", []),
    poi!("stand", "WAYPOINT", "Stand", []),
    poi!(
        "trail_cam",
        "WAYPOINT",
        "TrailCam",
        ["trail cam", "trail camera"]
    ),
    poi!("big_game", "WAYPOINT", "BigGame", ["big game"]),
    poi!("small_game", "WAYPOINT", "SmallGame", ["small game"]),
    poi!("bird", "WAYPOINT", "Bird", []),
    poi!("shot", "WAYPOINT", "Shot", []),
    poi!("fish", "FISHING AREA", "Fish", []),
    poi!("big_fish", "FISHING AREA", "BigFish", ["big fish"]),
    poi!("coral_reef", "WAYPOINT", "CoralReef", ["coral reef"]),
    poi!("beach", "WAYPOINT", "Beach", []),
    poi!(
        "marine_mammals",
        "WAYPOINT",
        "MarineMammals",
        ["marine mammals"]
    ),
    poi!("kelp_forest", "WAYPOINT", "KelpForest", ["kelp forest"]),
    poi!("lagoon", "WAYPOINT", "Lagoon", []),
    poi!("wreck", "WAYPOINT", "Wreck", ["shipwreck"]),
    poi!(
        "marine_reserve",
        "WAYPOINT",
        "MarineReserve",
        ["marine reserve"]
    ),
    poi!("avalanche", "DANGER AREA", "Avalanche", []),
    poi!("danger", "CHECKPOINT", "Danger", ["hazard", "alert","checkpoint"]),
    poi!(
        "aid_station",
        "MEDICAL FACILITY",
        "AidStation",
        ["aid station"]
    ),
    poi!("mushrooms", "WAYPOINT", "Mushrooms", ["mushroom"]),
    poi!("campfire", "CAMPGROUND", "Campfire", ["camp fire"]),
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
            Vendor::Garmin => "WAYPOINT",
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
            "SUMMIT"
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
    fn all_garmin_values_are_uppercase() {
        for entry in POI_DICTIONARY {
            assert_eq!(
                entry.garmin,
                entry.garmin.to_uppercase(),
                "valore Garmin non maiuscolo: {}",
                entry.garmin
            );
        }
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
