# universal_gpx_poi

Convertitore CLI dei waypoint/POI contenuti nei file GPX fra i vocabolari
usati da Garmin e Suunto.

## Uso

```bash
cargo run -- test_data/super-baldo.gpx
```

Il programma chiede il formato di destinazione e crea, accanto all'originale:

- `super-baldo-suunto.gpx`, usando `<type>` per il tipo POI;
- `super-baldo-garmin.gpx`, usando `<sym>` per il simbolo waypoint.

Il sorgente non viene sovrascritto. I campi e le estensioni GPX non relative ai
POI vengono conservati.

## Dizionario POI

Le corrispondenze sono raccolte in `src/poi.rs`, nella tabella
`POI_DICTIONARY`. Ogni voce ha:

- un nome canonico interno;
- il valore Garmin;
- il valore Suunto;
- una lista di alias riconosciuti in input.

Un valore sconosciuto viene convertito nel tipo generico `Waypoint` e mostrato
in un avviso, così è immediatamente evidente quale voce aggiungere al
dizionario.

## Compatibilità

GPX 1.1 definisce `sym` e `type` come stringhe, non come enum. Garmin usa
principalmente `sym`, mentre Suunto usa `type` per scegliere l'icona del POI.
Gli insiemi effettivi possono variare in base a modello e firmware: per questo
il dizionario è esplicito, conservativo ed estendibile.
