#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

PBF="monaco-latest.osm.pbf"
[[ -f "$PBF" ]] || curl -L -o "$PBF" "https://download.geofabrik.de/europe/monaco-latest.osm.pbf"
podman run --rm -v "$PWD:/data" ghcr.io/systemed/tilemaker:master "/data/$PBF" --output /data/output.mbtiles
