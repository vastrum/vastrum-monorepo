import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { get_all_repos, GitRepository } from '../../wasm/pkg';

function AllRepositories(): React.JSX.Element {
    const [repositories, setRepositories] = useState<GitRepository[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchRepos = async () => {
            try {
                const repos = await get_all_repos();
                setRepositories(repos);
            } catch (error) {
                console.error('Failed to fetch repositories:', error);
            } finally {
                setLoading(false);
            }
        };
        fetchRepos();
    }, []);

    if (loading) {
        return (
            <div className="max-w-7xl mx-auto px-5 py-5 md:px-6 md:py-6">
                <div className="text-app-text-secondary">Loading repositories...</div>
            </div>
        );
    }

    return (
        <div className="max-w-7xl mx-auto px-5 py-5 md:px-6 md:py-6">
            <div className="mb-6 md:mb-8">
                <h1 className="text-2xl md:text-3xl font-bold mb-2">Repositories</h1>
                <p className="text-app-text-secondary">
                    Browse all {repositories.length} repositories
                </p>
            </div>

            <div className="space-y-3 md:space-y-4">
                {repositories.map((repo) => (
                    <Link
                        key={repo.name}
                        to={`/repo/${repo.name}`}
                        className="block bg-app-bg-secondary border border-app-border rounded-lg p-4 md:p-6 hover:bg-app-hover transition-colors"
                    >
                        <div className="flex flex-col gap-3">
                            <div className="flex items-start gap-2 flex-wrap">
                                <h2 className="text-lg md:text-xl font-semibold text-app-accent-blue hover:underline break-words min-w-0">
                                    {repo.name}
                                </h2>
                                <span className="px-2 py-0.5 rounded-full text-xs font-medium border flex-shrink-0 border-app-border text-app-text-secondary">
                                    Public
                                </span>
                            </div>

                            <p className="text-sm md:text-base text-app-text-secondary">
                                {repo.description}
                            </p>
                        </div>
                    </Link>
                ))}
            </div>
        </div>
    );
}

export default AllRepositories;
