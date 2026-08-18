use vastrum_native_lib::deployers::build::run;

const TILES_DIR: &str = "../tiles";
const MONACO_MBTILES: &str = "../tiles/output.mbtiles";
const PLANET_MBTILES: &str = "../tiles/planet.mbtiles";
pub const PLANET_PBF: &str = "../tiles/planet-latest.osm.pbf";

pub fn ensure_tiles(planet: bool) -> String {
    if planet {
        if !std::path::Path::new(PLANET_MBTILES).exists() {
            generate_planet_tiles();
        }
        return PLANET_MBTILES.to_string();
    }
    if !std::path::Path::new(MONACO_MBTILES).exists() {
        generate_monaco_tiles();
    }
    return MONACO_MBTILES.to_string();
}

fn generate_monaco_tiles() {
    println!("generating Monaco tiles...");
    if !std::path::Path::new("../tiles/monaco-latest.osm.pbf").exists() {
        run(
            "curl -L -o monaco-latest.osm.pbf https://download.geofabrik.de/europe/monaco-latest.osm.pbf",
            TILES_DIR,
        );
    }
    let mount = tiles_mount();
    run(
        &format!(
            "podman run --rm -v {mount} ghcr.io/systemed/tilemaker:master \
             /data/monaco-latest.osm.pbf --output /data/output.mbtiles"
        ),
        TILES_DIR,
    );
}

fn generate_planet_tiles() {
    if !std::path::Path::new(PLANET_PBF).exists() {
        println!("downloading planet pbf (~70 GB)...");
        run(
            "curl -fSL --retry 3 --retry-delay 5 -C - -o planet-latest.osm.pbf.tmp https://planet.openstreetmap.org/pbf/planet-latest.osm.pbf",
            TILES_DIR,
        );
        run("mv planet-latest.osm.pbf.tmp planet-latest.osm.pbf", TILES_DIR);
    }

    // Ocean coastlines
    if !std::path::Path::new("../tiles/coastline").exists() {
        println!("downloading coastline shapefiles...");
        run(
            "curl -fSL --retry 3 --retry-delay 5 -o water_polygons.shp.zip https://osmdata.openstreetmap.de/download/water-polygons-split-4326.zip",
            TILES_DIR,
        );
        run("unzip -o water_polygons.shp.zip", TILES_DIR);
        run("mv water-polygons-split-4326 coastline", TILES_DIR);
        run("rm -f water_polygons.shp.zip", TILES_DIR);
    }

    // Natural Earth landcover (urban areas, ice shelves, glaciers)
    if !std::path::Path::new("../tiles/landcover").exists() {
        println!("downloading Natural Earth landcover...");
        run(
            "curl -fSL --retry 3 --retry-delay 5 -o ne_urban.zip https://naciscdn.org/naturalearth/10m/cultural/ne_10m_urban_areas.zip",
            TILES_DIR,
        );
        run(
            "curl -fSL --retry 3 --retry-delay 5 -o ne_ice.zip https://naciscdn.org/naturalearth/10m/physical/ne_10m_antarctic_ice_shelves_polys.zip",
            TILES_DIR,
        );
        run(
            "curl -fSL --retry 3 --retry-delay 5 -o ne_glaciers.zip https://naciscdn.org/naturalearth/10m/physical/ne_10m_glaciated_areas.zip",
            TILES_DIR,
        );
        run("mkdir -p landcover", TILES_DIR);
        run("unzip -o ne_urban.zip -d landcover/ne_10m_urban_areas", TILES_DIR);
        run("unzip -o ne_ice.zip -d landcover/ne_10m_antarctic_ice_shelves_polys", TILES_DIR);
        run("unzip -o ne_glaciers.zip -d landcover/ne_10m_glaciated_areas", TILES_DIR);
        run("rm -f ne_urban.zip ne_ice.zip ne_glaciers.zip", TILES_DIR);
    }

    prepare_process_lua();
    println!("running tilemaker (6-24h for the planet)...");
    let mount = tiles_mount();
    run(
        &format!(
            "podman run --rm -v {mount} -w /data -e THREADS=8 {TILEMAKER_IMAGE} \
             /data/planet-latest.osm.pbf --output /data/planet.mbtiles --store /data/tmp-store \
             --config /usr/src/app/config.json --process /data/process.lua"
        ),
        TILES_DIR,
    );
}

const TILEMAKER_IMAGE: &str = "ghcr.io/systemed/tilemaker:master";

fn prepare_process_lua() {
    let out_path = format!("{TILES_DIR}/process.lua");
    if std::path::Path::new(&out_path).exists() {
        return;
    }
    let output = std::process::Command::new("podman")
        .args(["run", "--rm", "--entrypoint", "cat", TILEMAKER_IMAGE, "/usr/src/app/process.lua"])
        .output()
        .expect("failed to read process.lua from the tilemaker image");
    assert!(
        output.status.success(),
        "podman cat process.lua failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lua = String::from_utf8(output.stdout).expect("process.lua is not utf-8");
    let patched = lua.replace("additional_languages = { }", "additional_languages = { \"en\" }");
    assert!(
        patched != lua,
        "process.lua did not contain the expected `additional_languages = {{ }}` line — the \
         tilemaker image changed; update prepare_process_lua()"
    );
    std::fs::write(&out_path, patched).expect("failed to write patched process.lua");
    println!("wrote patched process.lua (emits name:en for English labels)");
}

fn tiles_mount() -> String {
    let abs = std::fs::canonicalize(TILES_DIR).expect("tiles dir not found");
    return format!("{}:/data", abs.display());
}
