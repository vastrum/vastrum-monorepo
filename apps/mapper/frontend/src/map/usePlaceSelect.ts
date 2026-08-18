import { useEffect } from 'react';
import maplibregl from 'maplibre-gl';

export interface SelectedPlace {
    name: string;
    category: string;
    rawCategory: string;
    kind: string;
    approximate: boolean;
    lng: number;
    lat: number;
}

const CLICKABLE = ['poi', 'place', 'mountain_peak', 'aerodrome_label'];
const NEARBY = [...CLICKABLE, 'housenumber', 'transportation_name'];

function labelOf(props: Record<string, unknown>): string | undefined {
    return (props['name:en'] || props['name:latin'] || props.name) as string | undefined;
}

function titleCase(s: string): string {
    const t = s.replace(/_/g, ' ').trim();
    return t ? t[0].toUpperCase() + t.slice(1) : t;
}

const DBLCLICK_GRACE_MS = 250;
const FALLBACK_RADIUS_PX = 8;

export function usePlaceSelect(
    mapRef: React.RefObject<maplibregl.Map | null>,
    ready: boolean,
    onSelect: (place: SelectedPlace | null) => void,
) {
    useEffect(() => {
        if (!ready) return;
        const map = mapRef.current;
        if (!map) return;

        const selectAt = (point: maplibregl.Point, lngLat: maplibregl.LngLat) => {
            let features = map.queryRenderedFeatures(point);
            let feature = features.find(
                f => CLICKABLE.includes((f as any).sourceLayer) && labelOf(f.properties ?? {}),
            );
            let approximate = false;
            if (!feature) {
                const r = FALLBACK_RADIUS_PX;
                const box: [maplibregl.PointLike, maplibregl.PointLike] = [
                    [point.x - r, point.y - r],
                    [point.x + r, point.y + r],
                ];
                features = map.queryRenderedFeatures(box);
                feature = features.find(
                    f =>
                        NEARBY.includes((f as any).sourceLayer) &&
                        (labelOf(f.properties ?? {}) || f.properties?.housenumber),
                );
                approximate = true;
            }
            if (!feature) {
                onSelect(null);
                return;
            }
            const props = feature.properties ?? {};
            const name =
                labelOf(props) ||
                (props.housenumber ? `No. ${props.housenumber}` : 'Dropped pin');
            const rawCategory = (
                props.subclass ||
                props.class ||
                (feature as any).sourceLayer ||
                ''
            ).toString();
            onSelect({
                name,
                category: titleCase(rawCategory),
                rawCategory,
                kind: (feature as any).sourceLayer,
                approximate,
                lng: lngLat.lng,
                lat: lngLat.lat,
            });
        };

        let pendingSelect = 0;
        const onClick = (e: maplibregl.MapMouseEvent) => {
            if (pendingSelect) window.clearTimeout(pendingSelect);
            const point = e.point;
            const lngLat = e.lngLat;
            pendingSelect = window.setTimeout(() => {
                pendingSelect = 0;
                selectAt(point, lngLat);
            }, DBLCLICK_GRACE_MS);
        };
        const onDblClick = () => {
            if (pendingSelect) {
                window.clearTimeout(pendingSelect);
                pendingSelect = 0;
            }
        };

        let cursorRaf = 0;
        const onMouseMove = (e: maplibregl.MapMouseEvent) => {
            if (cursorRaf) return;
            cursorRaf = requestAnimationFrame(() => {
                cursorRaf = 0;
                const hit = map
                    .queryRenderedFeatures(e.point)
                    .some(f => CLICKABLE.includes((f as any).sourceLayer) && labelOf(f.properties ?? {}));
                map.getCanvas().style.cursor = hit ? 'pointer' : '';
            });
        };

        map.on('click', onClick);
        map.on('dblclick', onDblClick);
        map.on('mousemove', onMouseMove);
        return () => {
            if (cursorRaf) cancelAnimationFrame(cursorRaf);
            if (pendingSelect) window.clearTimeout(pendingSelect);
            map.off('click', onClick);
            map.off('dblclick', onDblClick);
            map.off('mousemove', onMouseMove);
        };
    }, [mapRef, ready, onSelect]);
}
