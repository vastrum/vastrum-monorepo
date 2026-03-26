import React, { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Menu, X } from 'lucide-react';
import { type GetRepoDetail, type ExplorerEntry } from '../../../wasm/pkg';
import FileTree from './FileTree';
import FilePreview from './FilePreview';
import { walkTreePath } from '../../utils/fileHelpers';

interface FileExplorerProps {
    repoData: GetRepoDetail;
    initialPath: string[];  // ["src", "components", "Button.tsx"]
}

function FileExplorer({ repoData, initialPath }: FileExplorerProps): React.JSX.Element {
    const navigate = useNavigate();
    const [isSidebarOpen, setIsSidebarOpen] = useState(false);
    const [selectedEntry, setSelectedEntry] = useState<ExplorerEntry | null>(null);

    const { git_repo, top_level_files } = repoData;

    // Walk the path to find and select the entry
    useEffect(() => {
        const loadPath = async () => {
            if (initialPath.length === 0) {
                setSelectedEntry(null);
                return;
            }
            try {
                const { targetEntry } = await walkTreePath(initialPath, top_level_files);
                if (targetEntry) {
                    setSelectedEntry(targetEntry);
                }
            } catch (error) {
                console.error('Failed to walk path:', error);
            }
        };
        loadPath();
    }, [initialPath, top_level_files]);

    const handleEntrySelect = useCallback((entry: ExplorerEntry, parentPath: string[]) => {
        setSelectedEntry(entry);
        const newPath = [...parentPath, entry.name].join('/');
        navigate(`/repo/${git_repo.name}/tree/${newPath}`);
    }, [git_repo.name, navigate]);

    return (
        <div className="space-y-3 md:space-y-4">
            {/* Mobile sidebar toggle button */}
            <button
                onClick={() => setIsSidebarOpen(!isSidebarOpen)}
                className="md:hidden flex items-center gap-2 px-3 py-2 bg-app-bg-secondary border border-app-border rounded-lg text-app-text-primary hover:bg-app-hover transition-colors"
                aria-label="Toggle file tree"
                aria-expanded={isSidebarOpen}
            >
                {isSidebarOpen ? <X className="w-4 h-4" /> : <Menu className="w-4 h-4" />}
                <span className="text-sm font-semibold">{isSidebarOpen ? 'Hide' : 'Show'} Files</span>
            </button>

            <div className="flex flex-col lg:flex-row gap-4 md:gap-6 lg:items-start">
                {/* File tree sidebar */}
                <div className={`${isSidebarOpen ? 'block' : 'hidden'} md:block w-full md:w-64 lg:w-[300px] flex-shrink-0 bg-app-bg-secondary border border-app-border rounded-lg overflow-hidden lg:self-start`}>
                    <FileTree
                        repoName={git_repo.name}
                        topLevelEntries={top_level_files}
                        selectedOid={selectedEntry?.oid || null}
                        initialPath={initialPath}
                        onSelect={handleEntrySelect}
                    />
                </div>

                {/* File preview */}
                <div className="flex-1 min-w-0 bg-app-bg-secondary border border-app-border rounded-lg overflow-hidden">
                    <FilePreview entry={selectedEntry} />
                </div>
            </div>
        </div>
    );
}

export default FileExplorer;
