const form = document.querySelector("#converter-form");
const fileInput = document.querySelector("#source-file");
const dropZone = document.querySelector("#drop-zone");
const fileLabel = document.querySelector("#file-label");
const fileHint = document.querySelector("#file-hint");
const target = document.querySelector("#target");
const button = document.querySelector("#convert-button");
const status = document.querySelector("#status");

let selectedFile = null;
let convertGpxForWeb;
const wasmReady = fetch("./pkg/manifest.json", { cache: "no-store" })
  .then((response) => {
    if (!response.ok) {
      throw new Error(`Manifest WebAssembly non disponibile (${response.status})`);
    }
    return response.json();
  })
  .then(({ module }) => import(`./pkg/${module}`))
  .then(async (wasm) => {
    await wasm.default();
    convertGpxForWeb = wasm.convert_gpx_for_web;
  });

function chooseFile(file) {
  if (!file) return;

  if (!file.name.toLowerCase().endsWith(".gpx")) {
    selectedFile = null;
    button.disabled = true;
    dropZone.classList.remove("has-file");
    showStatus("Seleziona un file con estensione .gpx.", true);
    return;
  }

  selectedFile = file;
  fileLabel.textContent = file.name;
  fileHint.textContent = formatBytes(file.size);
  dropZone.classList.add("has-file");
  button.disabled = false;
  status.hidden = true;
}

function showStatus(message, isError = false) {
  status.textContent = message;
  status.classList.toggle("error", isError);
  status.hidden = false;
}

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
}

function outputName(fileName, vendor) {
  const stem = fileName.replace(/\.gpx$/i, "");
  return `${stem}-${vendor}.gpx`;
}

function download(content, name) {
  const url = URL.createObjectURL(new Blob([content], { type: "application/gpx+xml;charset=utf-8" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}

fileInput.addEventListener("change", () => chooseFile(fileInput.files[0]));

for (const eventName of ["dragenter", "dragover"]) {
  dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropZone.classList.add("is-dragging");
  });
}

for (const eventName of ["dragleave", "drop"]) {
  dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropZone.classList.remove("is-dragging");
  });
}

dropZone.addEventListener("drop", (event) => chooseFile(event.dataTransfer.files[0]));

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!selectedFile) return;

  button.disabled = true;
  button.querySelector("span").textContent = "Conversione…";
  status.hidden = true;

  try {
    await wasmReady;
    const input = await selectedFile.text();
    const result = convertGpxForWeb(input, target.value);
    download(result.output, outputName(selectedFile.name, target.value));

    let message = `Fatto: ${result.waypoints} waypoint, ${result.translated} tradotti.`;
    if (result.unknown_values) {
      const fallback = target.value === "suunto" ? "POI" : "Waypoint";
      message += ` Valori non riconosciuti convertiti in ${fallback}: ${result.unknown_values}.`;
    }
    showStatus(message);
    result.free();
  } catch (error) {
    showStatus(`Conversione non riuscita: ${String(error)}`, true);
  } finally {
    button.disabled = false;
    button.querySelector("span").textContent = "Converti e scarica";
  }
});
