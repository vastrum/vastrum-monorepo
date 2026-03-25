import React, { useState, useEffect, useRef } from 'react';
import { ChevronRight, ChevronDown, Folder, File, Code, FileText, FileJson, Loader2 } from 'lucide-react';
import { type ExplorerEntry, get_directory_contents } from '../../../wasm/pkg';
import { getFileExtension, walkTreePath } from '../../utils/fileHelpers';

interface DirectoryState {
    [oid: string]: {
        expanded: boolean;
        children: ExplorerEntry[] | null;
        loading: boolean;
    };
}

function getFileIcon(name: string, isDirectory: boolean) {
    if (isDirectory) {
        return <Folder className="w-4 h-4 text-app-accent-blue flex-shrink-0" />;
    }

    const ext = getFileExtension(name);
    const iconMap: Record<string, React.JSX.Element> = {
        'ts': <Code className="w-4 h-4 text-app-accent-blue flex-shrink-0" />,
        'tsx': <Code className="w-4 h-4 text-app-accent-blue flex-shrink-0" />,
        'js': <Code className="w-4 h-4 text-[#f1e05a] flex-shrink-0" />,
        'jsx': <Code className="w-4 h-4 text-[#f1e05a] flex-shrink-0" />,
        'py': <FileText className="w-4 h-4 text-[#3572A5] flex-shrink-0" />,
        'dart': <FileText className="w-4 h-4 text-app-accent-blue flex-shrink-0" />,
        'json': <FileJson className="w-4 h-4 text-[#f1e05a] flex-shrink-0" />,
        'md': <FileText className="w-4 h-4 text-app-text-secondary flex-shrink-0" />,
        'html': <Code className="w-4 h-4 text-[#e34c26] flex-shrink-0" />,
        'css': <FileText className="w-4 h-4 text-[#563d7c] flex-shrink-0" />,
        'rs': <Code className="w-4 h-4 text-[#dea584] flex-shrink-0" />,
        'toml': <FileText className="w-4 h-4 text-[#9c4221] flex-shrink-0" />,
    };

    return iconMap[ext] || <File className="w-4 h-4 text-app-text-secondary flex-shrink-0" />;
}

interface FileTreeNodeProps {
    entry: ExplorerEntry;
    repoName: string;
    depth: number;
    parentPath: string[];
    directories: DirectoryState;
    selectedOid: string | null;
    onDirectoryToggle: (entry: ExplorerEntry) => void;
    onSelect: (entry: ExplorerEntry, parentPath: string[]) => void;
}

const FileTreeNode = ({
    entry,
    repoName,
    depth,
    parentPath,
    directories,
    selectedOid,
    onDirectoryToggle,
    onSelect,
}: FileTreeNodeProps): React.JSX.Element => {
    const dirState = directories[entry.oid];
    const isExpanded = dirState?.expanded ?? false;
    const isLoading = dirState?.loading ?? false;
    const children = dirState?.children ?? null;
    const isSelected = entry.oid === selectedOid;
    const isDirectory = entry.is_directory;

    const handleClick = () => {
        if (isDirectory) {
            onDirectoryToggle(entry);
        }
        onSelect(entry, parentPath);
    };

    const currentPath = [...parentPath, entry.name];

    return (
        <div>
            <div
                className={`flex items-center gap-2 px-3 py-1.5 hover:bg-app-hover cursor-pointer transition-colors ${
                    isSelected ? 'bg-app-hover' : ''
                }`}
                style={{ paddingLeft: `${depth * 16 + 12}px` }}
                onClick={handleClick}
            >
                {isDirectory && (
                    <span className="flex-shrink-0">
                        {isLoading ? (
                            <Loader2 className="w-4 h-4 text-app-text-secondary animate-spin" />
                        ) : isExpanded ? (
                            <ChevronDown className="w-4 h-4 text-app-text-secondary" />
                        ) : (
                            <ChevronRight className="w-4 h-4 text-app-text-secondary" />
                        )}
                    </span>
                )}
                {!isDirectory && <span className="w-4 flex-shrink-0" />}
                {getFileIcon(entry.name, isDirectory)}
                <span className="flex-1 text-sm text-app-text-primary truncate">{entry.name}</span>
            </div>

            {/* Render children recursively when folder is expanded */}
            {isDirectory && isExpanded && children && children.length > 0 && (
                <div>
                    {children.map((child, index) => (
                        <FileTreeNode
                            key={`${child.oid}-${index}`}
                            entry={child}
                            repoName={repoName}
                            depth={depth + 1}
                            parentPath={currentPath}
                            directories={directories}
                            selectedOid={selectedOid}
                            onDirectoryToggle={onDirectoryToggle}
                            onSelect={onSelect}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};

interface FileTreeProps {
    repoName: string;
    topLevelEntries: ExplorerEntry[];
    selectedOid: string | null;
    initialPath: string[];  // Path to expand
    onSelect: (entry: ExplorerEntry, parentPath: string[]) => void;
}

function FileTree({
    repoName,
    topLevelEntries,
    selectedOid,
    initialPath,
    onSelect,
}: FileTreeProps): React.JSX.Element {
    const [directories, setDirectories] = useState<DirectoryState>({});
    const hasExpandedInitialPath = useRef(false);

    // Expand directories along the initial path
    useEffect(() => {
        const expandPath = async () => {
            if (initialPath.length === 0) return;
            if (hasExpandedInitialPath.current) return;
            hasExpandedInitialPath.current = true;

            try {
                const { expandedDirs } = await walkTreePath(initialPath, topLevelEntries);

                const newDirState: DirectoryState = {};
                for (const { entry, children } of expandedDirs) {
                    newDirState[entry.oid] = { expanded: true, children, loading: false };
                }
                setDirectories(d => ({ ...d, ...newDirState }));
            } catch (error) {
                console.error('Failed to expand path:', error);
            }
        };
        expandPath();
    }, [initialPath, topLevelEntries]);

    const handleDirectoryToggle = async (entry: ExplorerEntry) => {
        if (!entry.is_directory) return;

        // Validate OID before making WASM call
        if (!entry.oid || entry.oid.length === 0) {
            console.error('Invalid OID: empty string');
            return;
        }

        const current = directories[entry.oid];

        if (current?.expanded) {
            // Collapse
            setDirectories(d => ({ ...d, [entry.oid]: { ...current, expanded: false } }));
            return;
        }

        if (current?.children) {
            // Already loaded, just expand
            setDirectories(d => ({ ...d, [entry.oid]: { ...current, expanded: true } }));
            return;
        }

        // Fetch children
        setDirectories(d => ({ ...d, [entry.oid]: { expanded: true, children: null, loading: true } }));

        try {
            const children = await get_directory_contents(entry.oid);
            setDirectories(d => ({ ...d, [entry.oid]: { expanded: true, children, loading: false } }));
        } catch (error) {
            console.error('Failed to fetch directory contents:', error);
            setDirectories(d => ({ ...d, [entry.oid]: { expanded: false, children: null, loading: false } }));
        }
    };

    return (
        <div className="py-2">
            {topLevelEntries.map((entry, index) => (
                <FileTreeNode
                    key={`${entry.oid}-${index}`}
                    entry={entry}
                    repoName={repoName}
                    depth={0}
                    parentPath={[]}
                    directories={directories}
                    selectedOid={selectedOid}
                    onDirectoryToggle={handleDirectoryToggle}
                    onSelect={onSelect}
                />
            ))}
        </div>
    );
}

export default FileTree;
