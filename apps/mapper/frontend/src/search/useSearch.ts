import { useCallback, useRef, useState } from 'react';
import maplibregl from 'maplibre-gl';
import { search_places } from '../../wasm/pkg';

export interface Place {
    name: string;
    lat: number;
    lng: number;
    rank: number;
}

export const RANKS = [
    { label: 'Country', zoom: 4 },
    { label: 'Region', zoom: 6 },
    { label: 'City', zoom: 11 },
    { label: 'Town', zoom: 12 },
    { label: 'Village', zoom: 13 },
    { label: 'Place', zoom: 14 },
    { label: 'Place', zoom: 14 },
    { label: 'Street', zoom: 16 },
    { label: 'POI', zoom: 17 },
];

function goTo(map: maplibregl.Map, center: [number, number], zoom: number) {
    const near = map.getBounds().contains(center) && Math.abs(map.getZoom() - zoom) <= 2;
    if (near) {
        map.flyTo({ center, zoom, duration: 800 });
    } else {
        map.jumpTo({ center, zoom });
    }
}

function parseCoords(text: string): [number, number] | null {
    const parts = text.split(/[,\s]+/).filter(Boolean);
    if (parts.length !== 2) return null;
    const lat = parseFloat(parts[0]);
    const lng = parseFloat(parts[1]);
    if (isNaN(lat) || isNaN(lng) || Math.abs(lat) > 90 || Math.abs(lng) > 180) return null;
    return [lat, lng];
}

function parseAddress(text: string): [string, string] | null {
    const tail = text.trim().match(/^(.*[^\d\s].*)\s+(\d+[a-zA-Z]?)$/);
    if (tail) return [tail[1], tail[2].toLowerCase()];
    const head = text.trim().match(/^(\d+[a-zA-Z]?)\s+(.*[^\d\s].*)$/);
    if (head) return [head[2], head[1].toLowerCase()];
    return null;
}

export function useSearch(mapRef: React.RefObject<maplibregl.Map | null>) {
    const [search, setSearch] = useState('');
    const [results, setResults] = useState<Place[]>([]);
    const searchSeq = useRef(0);
    const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    const pendingHouseNumber = useRef<{ num: string; tries: number } | null>(null);
    const markerRef = useRef<maplibregl.Marker | null>(null);

    const resolveHouseNumber = useCallback(() => {
        const map = mapRef.current;
        const pending = pendingHouseNumber.current;
        if (!map || !pending) return;

        const features = map.querySourceFeatures('vastrum', { sourceLayer: 'housenumber' });
        const center = map.getCenter();
        let best: { lng: number; lat: number; d: number } | null = null;
        for (const f of features) {
            const num = (f.properties?.housenumber ?? '').toString().toLowerCase();
            if (num !== pending.num || f.geometry.type !== 'Point') continue;
            const [lng, lat] = (f.geometry.coordinates as [number, number]);
            const d = (lat - center.lat) ** 2 + (lng - center.lng) ** 2;
            if (!best || d < best.d) best = { lng, lat, d };
        }
        if (!best) {
            if (pending.tries === 0) {
                pending.tries = 1;
                map.once('idle', resolveHouseNumber);
                map.easeTo({ zoom: map.getZoom() - 1.5, duration: 400 });
                return;
            }
            pendingHouseNumber.current = null;
            return;
        }
        pendingHouseNumber.current = null;
        markerRef.current?.remove();
        markerRef.current = new maplibregl.Marker({ color: '#3b82f6' })
            .setLngLat([best.lng, best.lat])
            .addTo(map);
        map.easeTo({ center: [best.lng, best.lat], zoom: 17, duration: 600 });
    }, [mapRef]);

    const selectPlace = useCallback((place: Place) => {
        const map = mapRef.current;
        if (!map) return;
        const zoom = RANKS[Math.min(place.rank, RANKS.length - 1)].zoom;
        goTo(map, [place.lng, place.lat], zoom);
        if (pendingHouseNumber.current !== null) {
            map.once('idle', resolveHouseNumber);
        }
        setSearch(place.name);
        setResults([]);
    }, [mapRef, resolveHouseNumber]);

    const handleSearchChange = useCallback((value: string) => {
        setSearch(value);
        if (debounceRef.current) clearTimeout(debounceRef.current);
        pendingHouseNumber.current = null;
        if (value.trim().length < 3 || parseCoords(value)) {
            setResults([]);
            return;
        }
        const seq = ++searchSeq.current;
        debounceRef.current = setTimeout(async () => {
            const address = parseAddress(value);
            const query = address ? address[0] : value;
            try {
                const center = mapRef.current?.getCenter();
                const res = await search_places(query, center?.lat ?? 0, center?.lng ?? 0);
                if (seq === searchSeq.current) {
                    pendingHouseNumber.current = address ? { num: address[1], tries: 0 } : null;
                    setResults(res.results as Place[]);
                }
            } catch {
                if (seq === searchSeq.current) setResults([]);
            }
        }, 250);
    }, [mapRef]);

    const handleSearchKey = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Escape') {
            setResults([]);
            return;
        }
        if (e.key !== 'Enter' || !mapRef.current) return;
        const coords = parseCoords(search);
        if (coords) {
            goTo(mapRef.current, [coords[1], coords[0]], 12);
            setSearch('');
            setResults([]);
            return;
        }
        if (results.length > 0) {
            selectPlace(results[0]);
        }
    }, [mapRef, search, results, selectPlace]);

    const clearResults = useCallback(() => setResults([]), []);

    return { search, results, handleSearchChange, handleSearchKey, selectPlace, clearResults };
}
