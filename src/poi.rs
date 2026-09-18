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
    /// Nome dell'icona Lucide associata al POI, se disponibile.
    #[allow(dead_code)]
    icon: Option<&'static str>,
}

// Sia Garmin sia Suunto usano il testo di <type>. Continuiamo comunque a
// riconoscere <sym> in input per poter convertire i GPX esistenti.
// I valori Garmin sono maiuscoli; quelli Suunto hanno le iniziali maiuscole.
// La prima voce resta il fallback generale: WAYPOINT per Garmin e POI per Suunto.
macro_rules! poi {
    ($canonical:literal, $garmin:literal, $suunto:literal, [$($alias:literal),* $(,)?]) => {
        PoiTranslation {
            canonical: $canonical,
            garmin: $garmin,
            suunto: $suunto,
            aliases: &[$($alias),*],
            icon: None,
        }
    };
    ($canonical:literal, $garmin:literal, $suunto:literal, $icon:literal) => {
        PoiTranslation {
            canonical: $canonical,
            garmin: $garmin,
            suunto: $suunto,
            aliases: &[],
            icon: Some($icon),
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
    poi!("alert", "ALERT", "Alert", "TriangleAlert"),
    poi!("anchor", "ANCHOR", "Anchor", "Anchor"),
    poi!("bank", "BANK", "Bank", "Landmark"),
    poi!("beach", "BEACH", "Beach", "Shell"),
    poi!("bike_trail", "BIKE TRAIL", "Bike Trail", "Bike"),
    poi!("binoculars", "BINOCULARS", "Binoculars", "Binoculars"),
    poi!("bridge", "BRIDGE", "Bridge", []),
    poi!("building", "BUILDING", "Building", "Building"),
    poi!("campground", "CAMPGROUND", "Campground", "Tent"),
    poi!("car", "CAR", "Car", "Car"),
    poi!("car_repair", "CAR REPAIR", "Car Repair", "Wrench"),
    poi!(
        "convenience_store",
        "CONVENIENCE STORE",
        "Convenience Store",
        "ShoppingBasket"
    ),
    poi!("crossing", "CROSSING", "Crossing", "X"),
    poi!(
        "department_store",
        "DEPARTMENT STORE",
        "Department Store",
        "ShoppingBasket"
    ),
    poi!(
        "drinking_water",
        "DRINKING WATER",
        "Drinking Water",
        "Droplet"
    ),
    poi!("exit", "EXIT", "Exit", "DoorOpen"),
    poi!("lodge", "LODGE", "Lodge", "House"),
    poi!("lodging", "LODGING", "Lodging", "Bed"),
    poi!("forest", "FOREST", "Forest", "Trees"),
    poi!("gas_station", "GAS STATION", "Gas Station", "Fuel"),
    poi!(
        "ground_transportation",
        "GROUND TRANSPORTATION",
        "Ground Transportation",
        "TrainFront"
    ),
    poi!("hotel", "HOTEL", "Hotel", "Bed"),
    poi!("house", "HOUSE", "House", "House"),
    poi!("information", "INFORMATION", "Information", "Info"),
    poi!("park", "PARK", "Park", "TreeDeciduous"),
    poi!(
        "parking_area",
        "PARKING AREA",
        "Parking Area",
        "CircleParking"
    ),
    poi!("pharmacy", "PHARMACY", "Pharmacy", "Cross"),
    poi!("picnic_area", "PICNIC AREA", "Picnic Area", "Utensils"),
    poi!("restaurant", "RESTAURANT", "Restaurant", "Utensils"),
    poi!(
        "restricted_area",
        "RESTRICTED AREA",
        "Restricted Area",
        "Construction"
    ),
    poi!("restroom", "RESTROOM", "Restroom", "Toilet"),
    poi!("road", "ROAD", "Road", "BrickWall"),
    poi!("scenic_area", "SCENIC AREA", "Scenic Area", "Binoculars"),
    poi!("shelter", "SHELTER", "Shelter", "Tent"),
    poi!(
        "shopping_center",
        "SHOPPING CENTER",
        "Shopping Center",
        "ShoppingBasket"
    ),
    poi!("shower", "SHOWER", "Shower", "ShowerHead"),
    poi!("summit", "SUMMIT", "Summit", "Mountain"),
    poi!("telephone", "TELEPHONE", "Telephone", "Phone"),
    poi!("tunnel", "TUNNEL", "Tunnel", []),
    poi!("water_source", "WATER SOURCE", "Water Source", "Droplet"),
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
    fn translates_values_for_both_targets() {
        assert_eq!(
            translate(Some("Drinking Water"), None, Vendor::Suunto).value,
            "Drinking Water"
        );
        assert_eq!(
            translate(None, Some("Summit"), Vendor::Garmin).value,
            "SUMMIT"
        );
        assert_eq!(
            translate(Some("LODGE"), None, Vendor::Suunto).value,
            "Lodge"
        );
    }

    #[test]
    fn accepts_normalized_values() {
        assert_eq!(
            translate(Some("car_repair"), None, Vendor::Suunto).value,
            "Car Repair"
        );
    }

    #[test]
    fn waypoint_is_the_only_general_fallback() {
        assert_eq!(POI_DICTIONARY[0].garmin, "WAYPOINT");
        assert_eq!(POI_DICTIONARY[0].suunto, "POI");
        assert!(
            POI_DICTIONARY[1..]
                .iter()
                .all(|entry| entry.garmin != "WAYPOINT" && entry.suunto != "POI")
        );
    }

    #[test]
    fn dictionary_matches_the_scratch_mapping() {
        let expected = [
            ("alert", "Alert", Some("TriangleAlert")),
            ("anchor", "Anchor", Some("Anchor")),
            ("bank", "Bank", Some("Landmark")),
            ("beach", "Beach", Some("Shell")),
            ("bike_trail", "Bike Trail", Some("Bike")),
            ("binoculars", "Binoculars", Some("Binoculars")),
            ("bridge", "Bridge", None),
            ("building", "Building", Some("Building")),
            ("campground", "Campground", Some("Tent")),
            ("car", "Car", Some("Car")),
            ("car_repair", "Car Repair", Some("Wrench")),
            (
                "convenience_store",
                "Convenience Store",
                Some("ShoppingBasket"),
            ),
            ("crossing", "Crossing", Some("X")),
            (
                "department_store",
                "Department Store",
                Some("ShoppingBasket"),
            ),
            ("drinking_water", "Drinking Water", Some("Droplet")),
            ("exit", "Exit", Some("DoorOpen")),
            ("lodge", "Lodge", Some("House")),
            ("lodging", "Lodging", Some("Bed")),
            ("forest", "Forest", Some("Trees")),
            ("gas_station", "Gas Station", Some("Fuel")),
            (
                "ground_transportation",
                "Ground Transportation",
                Some("TrainFront"),
            ),
            ("hotel", "Hotel", Some("Bed")),
            ("house", "House", Some("House")),
            ("information", "Information", Some("Info")),
            ("park", "Park", Some("TreeDeciduous")),
            ("parking_area", "Parking Area", Some("CircleParking")),
            ("pharmacy", "Pharmacy", Some("Cross")),
            ("picnic_area", "Picnic Area", Some("Utensils")),
            ("restaurant", "Restaurant", Some("Utensils")),
            ("restricted_area", "Restricted Area", Some("Construction")),
            ("restroom", "Restroom", Some("Toilet")),
            ("road", "Road", Some("BrickWall")),
            ("scenic_area", "Scenic Area", Some("Binoculars")),
            ("shelter", "Shelter", Some("Tent")),
            ("shopping_center", "Shopping Center", Some("ShoppingBasket")),
            ("shower", "Shower", Some("ShowerHead")),
            ("summit", "Summit", Some("Mountain")),
            ("telephone", "Telephone", Some("Phone")),
            ("tunnel", "Tunnel", None),
            ("water_source", "Water Source", Some("Droplet")),
        ];

        assert_eq!(POI_DICTIONARY.len(), expected.len() + 1);
        for (entry, (canonical, suunto, icon)) in POI_DICTIONARY[1..].iter().zip(expected) {
            assert_eq!(entry.canonical, canonical);
            assert_eq!(entry.garmin, suunto.to_uppercase());
            assert_eq!(entry.suunto, suunto);
            assert_eq!(entry.icon, icon);
        }
    }
}
