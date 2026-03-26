# Tile Generation

## Prerequisites

Download planet OSM data (~70 GB):
```bash
curl -L -o planet-latest.osm.pbf https://planet.openstreetmap.org/pbf/planet-latest.osm.pbf
```

Download shapefiles (run from this directory):
```bash
# Ocean coastlines 
curl -L -o water_polygons.shp.zip https://osmdata.openstreetmap.de/download/water-polygons-split-4326.zip
unzip -o water_polygons.shp.zip
mv water-polygons-split-4326 coastline
rm -f water_polygons.shp.zip 

# Natural Earth data
curl -L -o ne_urban.zip https://naciscdn.org/naturalearth/10m/cultural/ne_10m_urban_areas.zip
curl -L -o ne_ice.zip https://naciscdn.org/naturalearth/10m/physical/ne_10m_antarctic_ice_shelves_polys.zip
curl -L -o ne_glaciers.zip https://naciscdn.org/naturalearth/10m/physical/ne_10m_glaciated_areas.zip
mkdir -p landcover
unzip -o ne_urban.zip -d landcover/ne_10m_urban_areas
unzip -o ne_ice.zip -d landcover/ne_10m_antarctic_ice_shelves_polys
unzip -o ne_glaciers.zip -d landcover/ne_10m_glaciated_areas
rm -f ne_urban.zip ne_ice.zip ne_glaciers.zip
```

## Build

```bash
podman run --rm -v "$PWD:/data" -w /data -e THREADS=8 ghcr.io/systemed/tilemaker:master /data/planet-latest.osm.pbf --output /data/planet.mbtiles --store /data/tmp-store --config /usr/src/app/config.json --process /usr/src/app/process.lua  
```

Needs 16-32 GB RAM with `--store` (disk-backed). Without `--store` needs 64+ GB. Takes 6-24+ hours for full planet.
