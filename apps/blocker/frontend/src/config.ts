import { get_page_size } from '../wasm/pkg';

export const PAGE_SIZE = Number(get_page_size());

export function getSiteUrl(siteIdOrDomain: string): string {
    return `https://${siteIdOrDomain}.vastrum.net`;
}
