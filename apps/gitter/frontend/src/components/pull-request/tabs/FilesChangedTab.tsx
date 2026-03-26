import React, { useMemo } from 'react';
import { FileText, FilePlus, FileX, File } from 'lucide-react';
import { type FileDiff, type FileStatus } from '../../../../wasm/pkg';
import { getFileExtension, highlightCodeLine } from '../../../utils/fileHelpers';

function DiffContent({ file }: { file: FileDiff }): React.JSX.Element {
    const ext = getFileExtension(file.path);

    const lineNumbers = useMemo(() => {
        let oldNum = 1, newNum = 1;
        return file.diff.map(line => {
            const old = line.line_type === 'Add' ? '' : String(oldNum++);
            const new_ = line.line_type === 'Remove' ? '' : String(newNum++);
            return { old, new: new_ };
        });
    }, [file.diff]);

    return (
        <div className="inline-block min-w-full">
            {/* Unified diff header */}
            <div className="bg-app-bg-tertiary border-b border-app-border px-3 py-1 text-app-text-secondary w-full">
                @@ -{(() => {
                    const firstChange = file.diff.findIndex(l => l.line_type !== 'Context');
                    const start = Math.max(1, firstChange - 2);
                    const count = file.diff.filter(l => l.line_type !== 'Add').length;
                    return `${start},${count}`;
                })()} +{(() => {
                    const firstChange = file.diff.findIndex(l => l.line_type !== 'Context');
                    const start = Math.max(1, firstChange - 2);
                    const count = file.diff.filter(l => l.line_type !== 'Remove').length;
                    return `${start},${count}`;
                })()} @@
            </div>
            {file.diff.map((line, lineIndex) => {
                const bgColor = line.line_type === 'Add'
                    ? 'bg-[#238636]/15 hover:bg-[#238636]/25'
                    : line.line_type === 'Remove'
                        ? 'bg-[#da3633]/15 hover:bg-[#da3633]/25'
                        : 'hover:bg-app-hover/30';

                const highlightedContent = highlightCodeLine(line.content, ext, { wrapEmptyLines: false });

                return (
                    <div key={lineIndex} className={`flex ${bgColor} w-max`} style={{ lineHeight: '1.5' }}>
                        <div className="w-8 md:w-10 lg:w-12 px-1 md:px-2 lg:px-3 text-app-text-secondary text-right select-none bg-app-bg-tertiary border-r border-app-border flex-shrink-0" style={{ lineHeight: '1.5' }}>
                            {lineNumbers[lineIndex].old}
                        </div>
                        <div className="w-8 md:w-10 lg:w-12 px-1 md:px-2 lg:px-3 text-app-text-secondary text-right select-none bg-app-bg-tertiary border-r border-app-border flex-shrink-0" style={{ lineHeight: '1.5' }}>
                            {lineNumbers[lineIndex].new}
                        </div>
                        <div className="px-2 md:px-3 bg-transparent text-app-text-primary" style={{ lineHeight: '1.5' }}>
                            <code className={`hljs`} style={{ background: 'transparent', whiteSpace: 'pre' }}>
                                {line.line_type === 'Add' && <span className="text-app-accent-green mr-1 md:mr-2">+</span>}
                                {line.line_type === 'Remove' && <span className="text-app-accent-red mr-1 md:mr-2">-</span>}
                                {line.line_type === 'Context' && <span className="text-app-text-secondary mr-1 md:mr-2"> </span>}
                                <span dangerouslySetInnerHTML={{ __html: highlightedContent }} />
                            </code>
                        </div>
                    </div>
                );
            })}
        </div>
    );
}

interface FilesChangedTabProps {
    fileChanges: FileDiff[];
}

function FilesChangedTab({ fileChanges }: FilesChangedTabProps): React.JSX.Element {
    const files = fileChanges;
    const totalAdditions = files.reduce((sum, file) => sum + file.additions, 0);
    const totalDeletions = files.reduce((sum, file) => sum + file.deletions, 0);

    const getFileIcon = (status: FileStatus) => {
        switch (status) {
            case 'Added':
                return <FilePlus className="w-4 h-4 text-app-accent-green" />;
            case 'Deleted':
                return <FileX className="w-4 h-4 text-app-accent-red" />;
            default:
                return <File className="w-4 h-4 text-app-text-secondary" />;
        }
    };

    return (
        <div className="lg:col-span-2 space-y-4 md:space-y-5">
            {/* Files Summary */}
            <div className="bg-app-bg-secondary border border-app-border rounded-lg p-4 md:p-5">
                <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                    <h3 className="font-semibold text-sm md:text-base">
                        {files.length} changed {files.length === 1 ? 'file' : 'files'}
                    </h3>
                    <div className="text-xs md:text-sm">
                        <span className="text-app-accent-green font-semibold">+{totalAdditions}</span>
                        <span className="text-app-text-secondary mx-2">additions</span>
                        <span className="text-app-accent-red font-semibold">-{totalDeletions}</span>
                        <span className="text-app-text-secondary ml-2">deletions</span>
                    </div>
                </div>
            </div>

            {/* File Changes */}
            {files.length > 0 ? (
                files.map((file, fileIndex) => (
                    <div key={fileIndex} className="bg-app-bg-secondary border border-app-border rounded-lg overflow-hidden text-xs md:text-sm">
                        {/* File Header */}
                        <div className="px-3 py-2 md:px-4 md:py-3 bg-app-bg-tertiary border-b border-app-border flex items-center justify-between gap-2 md:gap-4">
                            <div className="flex items-center gap-1 md:gap-2 flex-1 min-w-0">
                                {getFileIcon(file.status)}
                                <span className="font-mono text-xs md:text-sm font-semibold truncate">{file.path}</span>
                                {file.status === 'Added' && (
                                    <span className="text-xs text-app-text-secondary bg-app-bg-secondary px-2 py-0.5 rounded whitespace-nowrap">New file</span>
                                )}
                            </div>
                            <div className="flex items-center gap-2 md:gap-3 text-xs whitespace-nowrap">
                                {file.additions > 0 && (
                                    <span className="text-app-accent-green font-semibold">+{file.additions}</span>
                                )}
                                {file.deletions > 0 && (
                                    <span className="text-app-accent-red font-semibold">-{file.deletions}</span>
                                )}
                            </div>
                        </div>

                        {/* Diff Content */}
                        <div className="font-mono text-xs md:text-sm overflow-x-auto overflow-y-hidden scrollbar-thin">
                            <DiffContent file={file} />
                        </div>
                    </div>
                ))
            ) : (
                <div className="bg-app-bg-secondary border border-app-border rounded-lg p-8 text-center">
                    <FileText className="w-12 h-12 text-app-text-secondary mx-auto mb-3" />
                    <p className="text-app-text-secondary">No file changes available for this pull request</p>
                </div>
            )}
        </div>
    );
}

export default FilesChangedTab;
