import { useEffect, useRef, useState } from 'react';
import maplibregl from 'maplibre-gl';
import 'maplibre-gl/dist/maplibre-gl.css';
import { get_tile } from '../../wasm/pkg';
import { createMapStyle } from './style';
import { getGlyphPBF } from './font-data';
import { getSpriteData } from './sprite-data';

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

export function useVastrumMap() {
    const containerRef = useRef<HTMLDivElement>(null);
    const mapRef = useRef<maplibregl.Map | null>(null);
    const [loading, setLoading] = useState(true);
    const [coords, setCoords] = useState({ lat: 0, lng: 0, zoom: 2 });

    useEffect(() => {
        if (!containerRef.current || mapRef.current) return;

        try { maplibregl.removeProtocol('vastrum'); } catch { }

        maplibregl.addProtocol('vastrum', async (params: { url: string }) => {
            const path = params.url.replace('vastrum://', '');

            if (path.startsWith('fonts/')) {
                const match = path.match(/^fonts\/(.+?)\/(\d+-\d+)\.pbf$/);
                if (match) {
                    const data = getGlyphPBF(decodeURIComponent(match[1]), match[2]);
                    return { data: data ?? new ArrayBuffer(0) };
                }
                return { data: new ArrayBuffer(0) };
            }

            if (path.startsWith('sprites/')) {
                const filename = path.replace('sprites/', '');
                const data = await getSpriteData(filename);
                return { data };
            }

            const parts = path.split('/');
            const z = parseInt(parts[0]);
            const x = parseInt(parts[1]);
            const y = parseInt(parts[2]);
            const key = `${z}/${x}/${y}`;

            const cached = cacheGet(key);
            if (cached) return { data: cached.slice(0) };

            const bytes = await get_tile(z, x, y);
            if (bytes.length === 0) return { data: new ArrayBuffer(0) };
            const buf = bytes.buffer.slice(
                bytes.byteOffset,
                bytes.byteOffset + bytes.byteLength,
            ) as ArrayBuffer;
            cacheSet(key, buf);
            return { data: buf.slice(0) };
        });

        const init = async () => {
            const center: [number, number] = [12.5, 41.9];
            const zoom = 3;
            const minZoom = 0;
            const maxZoom = 14;

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
                fadeDuration: 0,
                refreshExpiredTiles: false,
                maxTileCacheZoomLevels: 10,
            } as maplibregl.MapOptions);

            mapRef.current = map;

            map.on('styleimagemissing', (e) => {
                if (!map.hasImage(e.id)) {
                    map.addImage(e.id, { width: 1, height: 1, data: new Uint8Array(4) });
                }
            });

            setLoading(false);

            const ro = new ResizeObserver(() => map.resize());
            ro.observe(containerRef.current!);
            setTimeout(() => map.resize(), 100);

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
            try { maplibregl.removeProtocol('vastrum'); } catch { }
        };
    }, []);

    return { mapRef, containerRef, loading, coords };
}
