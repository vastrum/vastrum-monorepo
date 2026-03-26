import { useState, useEffect } from 'react';
import { get_site_count, get_sites_page, get_site_detail } from '../../wasm/pkg';
import type { SiteDetail } from '../types';
import HashLink from '../components/shared/HashLink';
import Pagination from '../components/shared/Pagination';
import { PAGE_SIZE, getSiteUrl } from '../config';

function SitesList() {
    const [sites, setSites] = useState<SiteDetail[]>([]);
    const [total, setTotal] = useState(0);
    const [page, setPage] = useState(0);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        loadPage(0);
    }, []);

    const loadPage = async (p: number) => {
        setLoading(true);
        setPage(p);
        const count = Number(await get_site_count());
        setTotal(count);

        if (count > 0) {
            const totalPages = Math.ceil(count / PAGE_SIZE);
            const reversePage = Math.max(0, totalPages - 1 - p);
            const ids = await get_sites_page(BigInt(reversePage));
            const reversed = [...ids].reverse();
            const results = await Promise.all(reversed.map(id => get_site_detail(id)));
            const details = results.filter(Boolean) as SiteDetail[];
            setSites(details);
        } else {
            setSites([]);
        }
        setLoading(false);
    };

    if (loading && sites.length === 0) {
        return <div className="text-blocker-text-muted text-center py-12">Loading...</div>;
    }

    return (
        <div>
            <h1 className="text-lg font-semibold text-blocker-text-primary mb-4">Deployed Sites ({total})</h1>

            <div className="bg-blocker-surface border border-blocker-border rounded-lg overflow-x-auto">
                <table className="w-full text-sm">
                    <thead>
                        <tr className="text-blocker-text-muted text-xs uppercase tracking-wider border-b border-blocker-border">
                            <th className="text-left px-2 sm:px-4 py-2.5 font-medium">Site ID</th>
                            <th className="text-left px-2 sm:px-4 py-2.5 font-medium">Domain</th>
                            <th className="text-right px-2 sm:px-4 py-2.5 font-medium hidden sm:table-cell">Block</th>
                            <th className="text-right px-2 sm:px-4 py-2.5 font-medium">Link</th>
                        </tr>
                    </thead>
                    <tbody>
                        {sites.map((site) => (
                            <tr key={site.site_id} className="border-b border-blocker-border last:border-0 hover:bg-blocker-surface-hover">
                                <td className="px-2 sm:px-4 py-2.5">
                                    <HashLink hash={site.site_id} to={`/site/${site.site_id}`} />
                                </td>
                                <td className="px-2 sm:px-4 py-2.5">
                                    {site.domain ? (
                                        <span className="text-blocker-success">{site.domain}</span>
                                    ) : (
                                        <span className="text-blocker-text-muted">-</span>
                                    )}
                                </td>
                                <td className="px-2 sm:px-4 py-2.5 text-right hidden sm:table-cell">
                                    <HashLink hash={String(site.block_height)} to={`/block/${site.block_height}`} truncate={20} />
                                </td>
                                <td className="px-2 sm:px-4 py-2.5 text-right">
                                    <a
                                        href={getSiteUrl(site.domain ?? site.site_id)}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="text-blocker-accent hover:text-blocker-accent-hover transition-colors"
                                        title={`Visit ${site.domain ?? site.site_id}`}
                                    >
                                        Visit
                                    </a>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
                {sites.length === 0 && (
                    <div className="text-center py-6 text-blocker-text-muted">No sites deployed yet</div>
                )}
                <Pagination currentPage={page} totalItems={total} pageSize={PAGE_SIZE} onPageChange={loadPage} />
            </div>
        </div>
    );
}

export default SitesList;
