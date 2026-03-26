import hljs from 'highlight.js';
import DOMPurify from 'dompurify';
import { type ExplorerEntry, get_directory_contents } from '../../wasm/pkg';

export interface PathWalkStep {
    entry: ExplorerEntry;
    children: ExplorerEntry[];
}

export interface PathWalkResult {
    targetEntry: ExplorerEntry | null;
    expandedDirs: PathWalkStep[];
}

/**
 * Walk a file tree path, fetching directory contents as needed.
 * Returns the target entry and all directories expanded along the way.
 */
export async function walkTreePath(
    pathSegments: string[],
    topLevelEntries: ExplorerEntry[]
): Promise<PathWalkResult> {
    if (pathSegments.length === 0) {
        return { targetEntry: null, expandedDirs: [] };
    }

    const expandedDirs: PathWalkStep[] = [];
    let currentEntries = topLevelEntries;
    let targetEntry: ExplorerEntry | null = null;

    for (let i = 0; i < pathSegments.length; i++) {
        const segment = pathSegments[i];
        const entry = currentEntries.find(e => e.name === segment);
        if (!entry) break;

        if (i === pathSegments.length - 1) {
            // Final segment
            targetEntry = entry;
            if (entry.is_directory) {
                const children = await get_directory_contents(entry.oid);
                expandedDirs.push({ entry, children });
            }
        } else if (entry.is_directory) {
            const children = await get_directory_contents(entry.oid);
            expandedDirs.push({ entry, children });
            currentEntries = children;
        } else {
            break; // Non-directory in middle of path
        }
    }

    return { targetEntry, expandedDirs };
}


export function getFileExtension(filename: string): string {
    const parts = filename.split('.');
    return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
}

/**
 * Get the filename from a path
 */
export function getFileName(path: string): string {
    return path.split('/').pop() || path;
}

export function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}


/**
 * Highlights a single line of code using highlight.js
 * @param content - The line of code to highlight
 * @param language - The language to use for syntax highlighting (e.g., 'typescript', 'python')
 * @param options - Optional configuration for highlighting behavior
 * @returns HTML string with syntax highlighting, or escaped plain text on error
 */
export function highlightCodeLine(
    content: string,
    language: string,
    options: { wrapEmptyLines?: boolean } = {}
): string {
    const { wrapEmptyLines = true } = options;

    if (!content.trim()) {
        return wrapEmptyLines ? '<span>&nbsp;</span>' : '&nbsp;';
    }

    try {
        // Check if the language is registered in highlight.js
        const validLanguage = language && hljs.getLanguage(language) ? language : 'plaintext';

        const result = hljs.highlight(content, {
            language: validLanguage,
            ignoreIllegals: true
        });
        // Sanitize the HTML output to prevent XSS attacks
        return DOMPurify.sanitize(result.value);
    } catch (e) {
        return DOMPurify.sanitize(content);
    }
}

/**
 * Highlights multiple lines of code using highlight.js
 * Useful for pre-processing entire code blocks
 * @param lines - Array of code lines to highlight
 * @param language - The language to use for syntax highlighting
 * @param options - Optional configuration for highlighting behavior
 * @returns Array of HTML strings with syntax highlighting
 */
export function highlightCodeLines(
    lines: string[],
    language: string,
    options?: { wrapEmptyLines?: boolean }
): string[] {
    return lines.map(line => highlightCodeLine(line, language, options));
}
