import React, { useMemo } from 'react';
import { highlightCodeLines } from '../../utils/fileHelpers';

interface SyntaxHighlighterProps {
    code: string;
    language: string;
    showLineNumbers?: boolean;
}

function SyntaxHighlighter({ code, language, showLineNumbers = true }: SyntaxHighlighterProps): React.JSX.Element {
    const highlightedLines = useMemo(() => {
        const lines = code.split('\n');
        return highlightCodeLines(lines, language, { wrapEmptyLines: true });
    }, [code, language]);

    const lines = code.split('\n');

    return (
        <div className="font-mono text-xs md:text-sm overflow-x-auto overflow-y-hidden scrollbar-thin" style={{ lineHeight: '1.5' }}>
            <div className="inline-block min-w-full">
                {lines.map((_line, index) => (
                    <div key={index} className="flex hover:bg-app-hover/30 w-max">
                        {showLineNumbers && (
                            <div className="w-8 md:w-10 lg:w-12 px-2 md:px-3 text-app-text-secondary text-right select-none bg-app-bg-tertiary border-r border-app-border flex-shrink-0" style={{ lineHeight: '1.5' }}>
                                {index + 1}
                            </div>
                        )}
                        <div className="px-2 md:px-3 bg-transparent" style={{ lineHeight: '1.5' }}>
                            <code
                                className={`language-${language} hljs`}
                                style={{ whiteSpace: 'pre', background: 'transparent' }}
                                dangerouslySetInnerHTML={{ __html: highlightedLines[index] || '<span>&nbsp;</span>' }}
                            />
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
}

export default SyntaxHighlighter;
