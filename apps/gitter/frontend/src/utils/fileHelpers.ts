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
            break;
        }
    }

    return { targetEntry, expandedDirs };
}

export function getFileExtension(filename: string): string {
    const parts = filename.split('.');
    return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
}

export function getFileName(path: string): string {
    return path.split('/').pop() || path;
}

export function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

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
        const validLanguage = language && hljs.getLanguage(language) ? language : 'plaintext';

        const result = hljs.highlight(content, {
            language: validLanguage,
            ignoreIllegals: true
        });
        return DOMPurify.sanitize(result.value);
    } catch (e) {
        return DOMPurify.sanitize(content);
    }
}

export function highlightCodeLines(
    lines: string[],
    language: string,
    options?: { wrapEmptyLines?: boolean }
): string[] {
    return lines.map(line => highlightCodeLine(line, language, options));
}
