import { useEffect, useRef, useState, useCallback } from 'react';
import maplibregl from 'maplibre-gl';
import 'maplibre-gl/dist/maplibre-gl.css';
import { get_tile, get_metadata } from '../../wasm/pkg';
import { createMapStyle } from '../map/style';
import { getGlyphPBF } from '../map/font-data';
import { getSpriteData } from '../map/sprite-data';

// ── Tile cache (LRU, avoids redundant RPC round-trips) ─────────
const MAX_CACHED_TILES = 512;
const tileCache = new Map<string, ArrayBuffer>();

function cacheGet(key: string): ArrayBuffer | undefined {
    const v = tileCache.get(key);
    if (v !== undefined) {
        tileCache.delete(key);
        tileCache.set(key, v);
    }
    return v;
}

function cacheSet(key: string, value: ArrayBuffer) {
    if (tileCache.has(key)) {
        tileCache.delete(key);
    } else if (tileCache.size >= MAX_CACHED_TILES) {
        tileCache.delete(tileCache.keys().next().value!);
    }
    tileCache.set(key, value);
}

// ── Component ───────────────────────────────────────────────────

export default function MapPage() {
    const containerRef = useRef<HTMLDivElement>(null);
    const mapRef = useRef<maplibregl.Map | null>(null);
    const [loading, setLoading] = useState(true);
    const [coords, setCoords] = useState({ lat: 0, lng: 0, zoom: 2 });
    const [search, setSearch] = useState('');
    const [searchFocused, setSearchFocused] = useState(false);

    // ── Map initialization ──────────────────────────────────────
    useEffect(() => {
        if (!containerRef.current || mapRef.current) return;

        // Remove stale protocol registration (React strict-mode double-mount)
        try { maplibregl.removeProtocol('vastrum'); } catch { /* noop */ }

        // Register the Vastrum protocol (tiles, fonts, sprites)
        maplibregl.addProtocol('vastrum', async (params: { url: string }) => {
            const path = params.url.replace('vastrum://', '');

            // Font glyph requests: fonts/{fontstack}/{start}-{end}.pbf
            if (path.startsWith('fonts/')) {
                const match = path.match(/^fonts\/(.+?)\/(\d+-\d+)\.pbf$/);
                if (match) {
                    const data = getGlyphPBF(decodeURIComponent(match[1]), match[2]);
                    return { data: data ?? new ArrayBuffer(0) };
                }
                return { data: new ArrayBuffer(0) };
            }

            // Sprite requests: sprites/sprite{@2x}.{json,png}
            if (path.startsWith('sprites/')) {
                const filename = path.replace('sprites/', '');
                const data = await getSpriteData(filename);
                return { data };
            }

            // Tile requests: {z}/{x}/{y}
            const parts = path.split('/');
            const z = parseInt(parts[0]);
            const x = parseInt(parts[1]);
            const y = parseInt(parts[2]);
            const key = `${z}/${x}/${y}`;

            const cached = cacheGet(key);
            if (cached) return { data: cached.slice(0) };

            try {
                const raw: Uint8Array = await get_tile(z, x, y);
                if (raw.length === 0) return { data: new ArrayBuffer(0) };
                const buf = raw.buffer.slice(raw.byteOffset, raw.byteOffset + raw.byteLength) as ArrayBuffer;
                cacheSet(key, buf);
                return { data: buf.slice(0) };
            } catch {
                return { data: new ArrayBuffer(0) };
            }
        });

        const init = async () => {
            let center: [number, number] = [12.5, 41.9];
            let zoom = 3;
            let minZoom = 0;
            let maxZoom = 14;
            let bounds: { minLng: number; minLat: number; maxLng: number; maxLat: number } | null = null;

            try {
                const m = await get_metadata();
                if (m && m.max_zoom > 0) {
                    minZoom = m.min_zoom;
                    maxZoom = m.max_zoom;
                }
                if (m && (m.center_lat !== 0 || m.center_lng !== 0)) {
                    center = [m.center_lng, m.center_lat];
                }
                if (m && (m.bounds_min_lat !== 0 || m.bounds_max_lat !== 0)) {
                    const lngSpan = Math.abs(m.bounds_max_lng - m.bounds_min_lng);
                    const latSpan = Math.abs(m.bounds_max_lat - m.bounds_min_lat);
                    const span = Math.max(lngSpan, latSpan);
                    if (span > 0 && span < 180) {
                        zoom = Math.min(maxZoom - 1, Math.floor(Math.log2(360 / span)));
                    }
                    bounds = {
                        minLng: m.bounds_min_lng - lngSpan,
                        minLat: m.bounds_min_lat - latSpan,
                        maxLng: m.bounds_max_lng + lngSpan,
                        maxLat: m.bounds_max_lat + latSpan,
                    };
                }
            } catch { /* metadata unavailable  use defaults */ }

            const MAP_MAX_ZOOM = 20;
            const map = new maplibregl.Map({
                container: containerRef.current!,
                style: createMapStyle(minZoom, maxZoom),
                center,
                zoom,
                minZoom,
                maxZoom: MAP_MAX_ZOOM,
                attributionControl: false,
                canvasContextAttributes: { alpha: false },
            } as maplibregl.MapOptions);

            mapRef.current = map;

            // Suppress missing sprite icon warnings  the OSM Bright style references
            // generic class-based icons (e.g. "amenity_11") that don't exist in our
            // sprite sheet, which only has specific icons (e.g. "restaurant_11").
            map.on('styleimagemissing', (e) => {
                if (!map.hasImage(e.id)) {
                    map.addImage(e.id, { width: 1, height: 1, data: new Uint8Array(4) });
                }
            });

            setLoading(false);

            // Force resize  iframe may expand after map creation
            const ro = new ResizeObserver(() => map.resize());
            ro.observe(containerRef.current!);
            setTimeout(() => map.resize(), 100);

            // Soft viewport clamp  keeps user within bounds without forcing minZoom
            if (bounds) {
                const b = bounds;
                map.on('moveend', () => {
                    const c = map.getCenter();
                    const clampLng = Math.max(b.minLng, Math.min(b.maxLng, c.lng));
                    const clampLat = Math.max(b.minLat, Math.min(b.maxLat, c.lat));
                    if (clampLng !== c.lng || clampLat !== c.lat) {
                        map.panTo([clampLng, clampLat], { duration: 300 });
                    }
                });
            }

            const sync = () => {
                const c = map.getCenter();
                setCoords({
                    lat: Math.round(c.lat * 10000) / 10000,
                    lng: Math.round(c.lng * 10000) / 10000,
                    zoom: Math.round(map.getZoom() * 10) / 10,
                });
            };

            map.on('move', sync);
            map.on('load', sync);
        };

        init();

        return () => {
            mapRef.current?.remove();
            mapRef.current = null;
            try { maplibregl.removeProtocol('vastrum'); } catch { /* noop */ }
        };
    }, []);

    // ── Controls ────────────────────────────────────────────────
    const zoomIn = useCallback(() => mapRef.current?.zoomIn({ duration: 200 }), []);
    const zoomOut = useCallback(() => mapRef.current?.zoomOut({ duration: 200 }), []);
    const resetNorth = useCallback(() => mapRef.current?.resetNorth({ duration: 200 }), []);

    const handleSearchKey = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key !== 'Enter' || !mapRef.current) return;
        const parts = search.split(/[,\s]+/).filter(Boolean);
        if (parts.length >= 2) {
            const lat = parseFloat(parts[0]);
            const lng = parseFloat(parts[1]);
            if (!isNaN(lat) && !isNaN(lng) && Math.abs(lat) <= 90 && Math.abs(lng) <= 180) {
                mapRef.current.flyTo({ center: [lng, lat], zoom: 12, duration: 1200 });
                setSearch('');
            }
        }
    }, [search]);

    const fmt = (v: number, p: string, n: string) => {
        const dir = v >= 0 ? p : n;
        return `${Math.abs(v).toFixed(4)}\u00B0${dir}`;
    };

    // ── Render ──────────────────────────────────────────────────
    return (
        <div className="relative w-full overflow-hidden" style={{ height: '100vh' }}>
            {/* Map canvas  inline styles to prevent MapLibre's position:relative from overriding */}
            <div ref={containerRef} style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%' }} />

            {/* Loading overlay */}
            {loading && (
                <div className="absolute inset-0 z-50 flex items-center justify-center"
                     style={{ background: '#080a0f' }}>
                    <div className="relative flex items-center justify-center">
                        <div className="sonar sonar-1" />
                        <div className="sonar sonar-2" />
                        <div className="sonar sonar-3" />
                        <div className="sonar-core" />
                        <p className="absolute top-28 tracking-[0.25em] uppercase text-xs"
                           style={{ color: '#4a5568', fontFamily: 'SF Mono, Menlo, Consolas, monospace' }}>
                            Initializing
                        </p>
                    </div>
                </div>
            )}

            {/* Search */}
            <div className="absolute top-4 left-1/2 -translate-x-1/2 z-10 w-full max-w-lg px-4"
                 style={{ pointerEvents: 'none' }}>
                <div className="search-pill" data-focused={searchFocused}
                     style={{ pointerEvents: 'auto' }}>
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none"
                         stroke="#94a3b8" strokeWidth="2" strokeLinecap="round">
                        <circle cx="11" cy="11" r="8" />
                        <line x1="21" y1="21" x2="16.65" y2="16.65" />
                    </svg>
                    <input
                        type="text"
                        value={search}
                        onChange={e => setSearch(e.target.value)}
                        onFocus={() => setSearchFocused(true)}
                        onBlur={() => setSearchFocused(false)}
                        onKeyDown={handleSearchKey}
                        placeholder="Enter coordinates (lat, lng)..."
                        className="search-input"
                    />
                </div>
            </div>

            {/* Zoom + compass */}
            <div className="absolute bottom-24 right-4 z-10 flex flex-col gap-px"
                 style={{ pointerEvents: 'none' }}>
                <button onClick={zoomIn} className="ctrl-btn ctrl-btn-top">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                         stroke="currentColor" strokeWidth="2.5" strokeLinecap="round">
                        <line x1="12" y1="5" x2="12" y2="19" />
                        <line x1="5" y1="12" x2="19" y2="12" />
                    </svg>
                </button>
                <button onClick={resetNorth} className="ctrl-btn ctrl-btn-mid">
                    <span style={{ fontSize: '9px', fontWeight: 700, letterSpacing: '0.08em' }}>N</span>
                </button>
                <button onClick={zoomOut} className="ctrl-btn ctrl-btn-bot">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                         stroke="currentColor" strokeWidth="2.5" strokeLinecap="round">
                        <line x1="5" y1="12" x2="19" y2="12" />
                    </svg>
                </button>
            </div>

            {/* Coordinates */}
            <div className="hidden md:block absolute bottom-3 left-3 z-10 coord-badge">
                <span>{fmt(coords.lat, 'N', 'S')}</span>
                <span style={{ opacity: 0.3 }}>&ensp;|&ensp;</span>
                <span>{fmt(coords.lng, 'E', 'W')}</span>
                <span style={{ opacity: 0.3 }}>&ensp;|&ensp;</span>
                <span style={{ opacity: 0.4 }}>z{coords.zoom}</span>
            </div>

            {/* Attribution */}
            <div className="absolute bottom-3 right-3 z-10"
                 style={{ color: '#64748b', fontSize: '10px', letterSpacing: '0.04em' }}>
                Style &copy; <a href="https://openmaptiles.org/" target="_blank" rel="noopener noreferrer"
                   style={{ color: '#64748b', textDecoration: 'underline' }}>OpenMapTiles</a>
                {' | Data \u00a9 '}
                <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener noreferrer"
                   style={{ color: '#64748b', textDecoration: 'underline' }}>OpenStreetMap</a>
            </div>
        </div>
    );
}
