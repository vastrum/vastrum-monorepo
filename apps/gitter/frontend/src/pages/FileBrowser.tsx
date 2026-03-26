import React, { useState, useEffect } from 'react';
import { Link, useParams } from 'react-router-dom';
import { get_repo_page_data, type GetRepoDetail } from '../../wasm/pkg';
import FileExplorer from '../components/repository/FileExplorer';

function FileBrowser(): React.JSX.Element {
    const { repoId, '*': filePath } = useParams<{ repoId: string; '*': string }>();
    const [repoData, setRepoData] = useState<GetRepoDetail | null>(null);
    const [loading, setLoading] = useState(true);

    // Extract path segments from the URL path
    const pathSegments = filePath ? filePath.split('/').filter(Boolean) : [];

    useEffect(() => {
        const fetchRepoData = async () => {
            if (!repoId) return;
            try {
                const data = await get_repo_page_data(repoId);
                setRepoData(data);
            } catch (error) {
                console.error('Failed to fetch repository data:', error);
            } finally {
                setLoading(false);
            }
        };
        fetchRepoData();
    }, [repoId]);

    if (loading) {
        return (
            <div className="max-w-7xl mx-auto px-5 py-5 md:px-6 md:py-6">
                <div className="text-app-text-secondary">Loading...</div>
            </div>
        );
    }

    if (!repoData) {
        return (
            <div className="max-w-7xl mx-auto px-5 py-5 md:px-6 md:py-6">
                <p className="text-app-text-secondary">Repository not found</p>
            </div>
        );
    }

    const { git_repo } = repoData;

    return (
        <div className="max-w-7xl mx-auto px-5 py-5 md:px-6 md:py-6">
            {/* Breadcrumb */}
            <div className="flex items-center gap-2 mb-3 md:mb-4 text-xs md:text-sm overflow-x-auto scrollbar-thin pb-1">
                <Link to={`/repo/${git_repo.name}`} className="text-app-accent-blue hover:underline whitespace-nowrap flex-shrink-0">
                    {git_repo.name}
                </Link>
                {pathSegments.map((segment, index) => (
                    <span key={index} className="flex items-center gap-2">
                        <span className="text-app-text-secondary">/</span>
                        {index === pathSegments.length - 1 ? (
                            <span className="text-app-text-primary">{segment}</span>
                        ) : (
                            <Link
                                to={`/repo/${git_repo.name}/tree/${pathSegments.slice(0, index + 1).join('/')}`}
                                className="text-app-accent-blue hover:underline"
                            >
                                {segment}
                            </Link>
                        )}
                    </span>
                ))}
            </div>

            {/* File Explorer */}
            <FileExplorer
                repoData={repoData}
                initialPath={pathSegments}
            />
        </div>
    );
}

export default FileBrowser;
