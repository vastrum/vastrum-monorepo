import React, { useState } from 'react';
import MarkdownRenderer from './MarkdownRenderer';

interface MarkdownEditorProps {
    value: string;
    onChange: (value: string) => void;
    placeholder?: string;
    minHeight?: string;
    id?: string;
}

function MarkdownEditor({
    value,
    onChange,
    placeholder = 'Type here. Use Markdown to format.',
    minHeight = '120px',
    id,
}: MarkdownEditorProps): React.JSX.Element {
    const [showPreview, setShowPreview] = useState(false);

    return (
        <div>
            {/* Write/Preview Tab Buttons */}
            <div className="flex gap-1 mb-2">
                <button
                    type="button"
                    onClick={() => setShowPreview(false)}
                    className={`px-3 py-1.5 text-sm font-medium rounded transition-colors ${
                        !showPreview
                            ? 'bg-app-bg-tertiary text-app-text-primary'
                            : 'bg-transparent text-app-text-secondary hover:text-app-text-primary'
                    }`}
                >
                    Write
                </button>
                <button
                    type="button"
                    onClick={() => setShowPreview(true)}
                    className={`px-3 py-1.5 text-sm font-medium rounded transition-colors ${
                        showPreview
                            ? 'bg-app-bg-tertiary text-app-text-primary'
                            : 'bg-transparent text-app-text-secondary hover:text-app-text-primary'
                    }`}
                >
                    Preview
                </button>
            </div>

            {/* Textarea or Preview */}
            {!showPreview ? (
                <textarea
                    id={id}
                    className="input-field resize-y"
                    style={{ minHeight }}
                    placeholder={placeholder}
                    value={value}
                    onChange={(e) => onChange(e.target.value)}
                />
            ) : (
                <div
                    className="border border-app-border rounded-md bg-app-bg-primary p-4"
                    style={{ minHeight }}
                >
                    {value.trim() ? (
                        <MarkdownRenderer content={value} />
                    ) : (
                        <span className="text-app-text-secondary text-sm">
                            Nothing to preview
                        </span>
                    )}
                </div>
            )}
        </div>
    );
}

export default MarkdownEditor;
