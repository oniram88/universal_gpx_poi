#!/bin/sh

set -eu

project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
pkg_dir="$project_root/web/pkg"
crate_name="universal_gpx_poi"

rm -rf "$pkg_dir"

wasm-pack build \
    --target web \
    --out-dir "$pkg_dir" \
    --release \
    "$project_root"

wasm_source="$pkg_dir/${crate_name}_bg.wasm"
wasm_hash=$(sha256sum "$wasm_source" | cut -c1-16)
wasm_file="${crate_name}_bg.${wasm_hash}.wasm"
mv "$wasm_source" "$pkg_dir/$wasm_file"

module_source="$pkg_dir/${crate_name}.js"
module_tmp="$pkg_dir/${crate_name}.js.tmp"
sed "s/${crate_name}_bg\\.wasm/$wasm_file/g" "$module_source" > "$module_tmp"
mv "$module_tmp" "$module_source"

module_hash=$(sha256sum "$module_source" | cut -c1-16)
module_file="${crate_name}.${module_hash}.js"
mv "$module_source" "$pkg_dir/$module_file"

printf '{\n  "module": "%s"\n}\n' "$module_file" > "$pkg_dir/manifest.json"

echo "Web build completata: $module_file, $wasm_file"
