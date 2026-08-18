import { useCallback, useEffect, useRef, useState } from 'react';
import maplibregl from 'maplibre-gl';
import { useVastrumMap } from '../map/useVastrumMap';
import { usePlaceSelect, type SelectedPlace } from '../map/usePlaceSelect';
import { useSearch } from '../search/useSearch';
import SearchBox from '../components/SearchBox';
import PlaceCard from '../components/PlaceCard';

export default function MapPage() {
    const { mapRef, containerRef, loading, coords } = useVastrumMap();
    const [place, setPlace] = useState<SelectedPlace | null>(null);
    usePlaceSelect(mapRef, !loading, setPlace);
    const { search, results, handleSearchChange, handleSearchKey, selectPlace, clearResults } =
        useSearch(mapRef);

    const markerRef = useRef<maplibregl.Marker | null>(null);
    useEffect(() => {
        markerRef.current?.remove();
        markerRef.current = null;
        if (place && mapRef.current) {
            markerRef.current = new maplibregl.Marker({ color: '#0ea5e9' })
                .setLngLat([place.lng, place.lat])
                .addTo(mapRef.current);
        }
        return () => {
            markerRef.current?.remove();
            markerRef.current = null;
        };
    }, [place, mapRef]);

    const zoomIn = useCallback(() => mapRef.current?.zoomIn({ duration: 200 }), [mapRef]);
    const zoomOut = useCallback(() => mapRef.current?.zoomOut({ duration: 200 }), [mapRef]);
    const resetNorth = useCallback(() => mapRef.current?.resetNorth({ duration: 200 }), [mapRef]);

    const fmt = (v: number, p: string, n: string) => {
        const dir = v >= 0 ? p : n;
        return `${Math.abs(v).toFixed(4)}°${dir}`;
    };

    return (
        <div className="relative w-full overflow-hidden" style={{ height: '100vh' }}>
            {}
            <div ref={containerRef} style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%' }} />

            {}
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

            {}
            <SearchBox
                search={search}
                results={results}
                onSearchChange={handleSearchChange}
                onSearchKey={handleSearchKey}
                onSelectPlace={selectPlace}
                onClearResults={clearResults}
            />

            {}
            {place && <PlaceCard place={place} onClose={() => setPlace(null)} />}

            {}
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

            {}
            <div className="hidden md:block absolute bottom-3 left-3 z-10 coord-badge">
                <span>{fmt(coords.lat, 'N', 'S')}</span>
                <span style={{ opacity: 0.3 }}>&ensp;|&ensp;</span>
                <span>{fmt(coords.lng, 'E', 'W')}</span>
                <span style={{ opacity: 0.3 }}>&ensp;|&ensp;</span>
                <span style={{ opacity: 0.4 }}>z{coords.zoom}</span>
            </div>

            {}
            <div className="absolute bottom-3 right-3 z-10"
                 style={{ color: '#64748b', fontSize: '10px', letterSpacing: '0.04em' }}>
                Style &copy; <a href="https://openmaptiles.org/" target="_blank" rel="noopener noreferrer"
                   style={{ color: '#64748b', textDecoration: 'underline' }}>OpenMapTiles</a>
                {' | Data © '}
                <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener noreferrer"
                   style={{ color: '#64748b', textDecoration: 'underline' }}>OpenStreetMap</a>
            </div>
        </div>
    );
}
