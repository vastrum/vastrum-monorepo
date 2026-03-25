import osmBright from './osm-bright.json';
import type { StyleSpecification } from 'maplibre-gl';

export function createMapStyle(minZoom: number, maxZoom: number): StyleSpecification {
    const style = structuredClone(osmBright) as StyleSpecification;
    const source = style.sources.vastrum as { minzoom?: number; maxzoom?: number };
    source.minzoom = minZoom;
    source.maxzoom = maxZoom;
    return style;
}
