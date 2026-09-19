# universal_gpx_poi

Convertitore CLI dei waypoint/POI contenuti nei file GPX fra i vocabolari
usati da Garmin e Suunto.

## Uso da CLI

```bash
cargo run -- test_data/super-baldo.gpx
```

Il programma chiede il formato di destinazione e crea, accanto all'originale:

- `super-baldo-suunto.gpx`, usando `<type>` per il tipo POI;
- `super-baldo-garmin.gpx`, usando `<type>` con valori in maiuscolo.

Il sorgente non viene sovrascritto. I campi e le estensioni GPX non relative ai
POI vengono conservati.

## Uso dal browser

La pagina in `web/` usa lo stesso core Rust della CLI, compilato in WebAssembly.
Il GPX viene letto, convertito e scaricato interamente nel browser: non viene
inviato a un server.

Installa una volta `wasm-pack`, quindi compila e avvia un server statico:

```bash
cargo install wasm-pack
npm run build
npm run serve
```

Apri <http://localhost:8080>. La build genera in `web/pkg/` il modulo
JavaScript e il file `.wasm` con un hash del contenuto nel nome, insieme al
`manifest.json` usato per individuarli. I file sono pronti per la pubblicazione
su qualunque hosting statico. Non aprire direttamente `web/index.html` tramite
`file://`, perché i browser caricano i moduli WebAssembly via HTTP.

## Container Docker

L'immagine usa una build multi-stage: il primo stage compila il core Rust in
WebAssembly, mentre l'immagine finale contiene soltanto Nginx e i file statici
della pagina.

```bash
docker build -t universal-gpx-poi .
docker run --rm -p 8080:80 universal-gpx-poi
```

Apri <http://localhost:8080>. La conversione continua ad avvenire nel browser:
il container serve soltanto HTML, CSS, JavaScript e WebAssembly e non riceve il
contenuto dei file GPX.

In alternativa, usa Docker Compose:

```bash
docker compose up --build
```

La porta predefinita e' `8080`; per cambiarla, ad esempio in `3000`:

```bash
PORT=3000 docker compose up --build
```

Per arrestare e rimuovere lo stack:

```bash
docker compose down
```

## Dizionario POI

Le corrispondenze sono raccolte in `src/poi.rs`, nella tabella
`POI_DICTIONARY`. Ogni voce ha:

- un nome canonico interno;
- i valori GPX per Garmin e le etichette testuali supportate da Suunto;
- una lista di alias riconosciuti in input;
- il nome dell'icona Lucide, quando disponibile.

La prima voce del dizionario e' il fallback generale. Nei file GPX viene scritto
il valore testuale previsto da `<type>`. Un valore sconosciuto viene convertito
nel tipo generico `WAYPOINT` per Garmin o `POI` per Suunto e mostrato in un
avviso, così è immediatamente evidente quale voce aggiungere al dizionario.

## Compatibilità

GPX 1.1 definisce `sym` e `type` come stringhe, non come enum. Il convertitore
scrive i valori di entrambi i formati in `type`: quelli Garmin interamente in
maiuscolo, quelli Suunto usando etichette testuali (per esempio `Water` e
`Peak`), mai gli ID numerici. `sym` viene comunque
riconosciuto nei file sorgente per compatibilita'.
Gli insiemi effettivi possono variare in base a modello e firmware: per questo
il dizionario è esplicito, conservativo ed estendibile.
