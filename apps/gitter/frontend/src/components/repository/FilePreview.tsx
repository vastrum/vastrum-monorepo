import React, { useState, useEffect } from 'react';
import { FileText, Copy, Check } from 'lucide-react';
import { type ExplorerEntry, get_file, is_file_binary } from '../../../wasm/pkg';
import { getFileExtension } from '../../utils/fileHelpers';
import SyntaxHighlighter from './SyntaxHighlighter';

interface FilePreviewProps {
    entry: ExplorerEntry | null;
}

function FilePreview({ entry }: FilePreviewProps): React.JSX.Element {
    const [copied, setCopied] = useState(false);
    const [content, setContent] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [isBinary, setIsBinary] = useState(false);

    useEffect(() => {
        const fetchContent = async () => {
            if (!entry || entry.is_directory) {
                setContent(null);
                setIsBinary(false);
                return;
            }

            // Validate OID before making WASM call
            if (!entry.oid || entry.oid.length === 0) {
                console.error('Invalid OID: empty string');
                setContent(null);
                setIsBinary(false);
                return;
            }

            setLoading(true);
            try {
                const binary = await is_file_binary(entry.oid);
                setIsBinary(binary);
                if (binary) {
                    setContent(null);
                    setLoading(false);
                    return;
                }
                const fileContent = await get_file(entry.oid);
                setContent(fileContent);
            } catch (error) {
                console.error('Failed to fetch file content:', error);
                setContent(null);
            } finally {
                setLoading(false);
            }
        };

        fetchContent();
    }, [entry]);

    if (!entry) {
        return (
            <div className="flex-1 flex items-center justify-center bg-app-bg-secondary">
                <div className="text-center">
                    <FileText className="w-16 h-16 text-app-text-secondary mx-auto mb-4" />
                    <p className="text-app-text-secondary">Select a file to preview</p>
                </div>
            </div>
        );
    }

    if (entry.is_directory) {
        return (
            <div className="flex-1 flex items-center justify-center bg-app-bg-secondary">
                <div className="text-center">
                    <FileText className="w-16 h-16 text-app-text-secondary mx-auto mb-4" />
                    <p className="text-app-text-secondary">This is a folder. Select a file to preview.</p>
                </div>
            </div>
        );
    }

    const ext = getFileExtension(entry.name);

    const handleCopy = async () => {
        if (content) {
            await navigator.clipboard.writeText(content);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    return (
        <div className="flex-1 flex flex-col bg-app-bg-secondary">
            {/* Header */}
            <div className="px-4 py-3 bg-app-bg-tertiary border-b border-app-border flex items-center justify-between flex-shrink-0">
                <div className="flex items-center gap-2 flex-1 min-w-0">
                    <FileText className="w-4 h-4 text-app-text-secondary flex-shrink-0" />
                    <span className="font-mono text-sm font-semibold truncate">{entry.name}</span>
                </div>
                {!isBinary && (
                    <div className="flex items-center gap-2 flex-shrink-0 ml-4">
                        <button
                            onClick={handleCopy}
                            className="p-1.5 hover:bg-app-hover rounded transition-colors"
                            title="Copy content"
                        >
                            {copied ? (
                                <Check className="w-4 h-4 text-app-accent-green" />
                            ) : (
                                <Copy className="w-4 h-4 text-app-text-secondary" />
                            )}
                        </button>
                    </div>
                )}
            </div>

            {/* Content */}
            <div className="flex-1">
                {loading ? (
                    <div className="flex items-center justify-center h-full min-h-[300px]">
                        <p className="text-app-text-secondary">Loading...</p>
                    </div>
                ) : isBinary ? (
                    <div className="flex items-center justify-center h-full min-h-[300px]">
                        <p className="text-app-text-secondary">Binary file not shown.</p>
                    </div>
                ) : !content ? (
                    <div className="flex items-center justify-center h-full min-h-[300px]">
                        <p className="text-app-text-secondary">No preview available</p>
                    </div>
                ) : (
                    <SyntaxHighlighter code={content} language={ext} showLineNumbers={true} />
                )}
            </div>
        </div>
    );
}

export default FilePreview;
