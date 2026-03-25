import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';
import type { EditorView } from '@tiptap/pm/view';

const PAGE_CONTENT_HEIGHT = 939;
const pluginKey = new PluginKey('pageBreaks');

function createPageBreakElement(): HTMLElement {
    const el = document.createElement('div');
    el.className = 'page-break-indicator';
    el.contentEditable = 'false';
    return el;
}

function measureAndComputeBreaks(view: EditorView): number[] {
    const positions: number[] = [];
    const { doc } = view.state;
    let accumulated = 0;

    doc.forEach((_node, offset, _index) => {
        const domNode = view.nodeDOM(offset) as HTMLElement | null;
        if (!domNode || !(domNode instanceof HTMLElement)) return;

        const style = window.getComputedStyle(domNode);
        const marginTop = parseFloat(style.marginTop) || 0;
        const marginBottom = parseFloat(style.marginBottom) || 0;
        const totalHeight = domNode.offsetHeight + marginTop + marginBottom;

        if (accumulated > 0 && accumulated + totalHeight > PAGE_CONTENT_HEIGHT) {
            positions.push(offset);
            accumulated = totalHeight;
        } else {
            accumulated += totalHeight;
        }
    });

    return positions;
}

function arraysEqual(a: number[], b: number[]): boolean {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) return false;
    }
    return true;
}

export const PageBreaks = Extension.create({
    name: 'pageBreaks',

    addProseMirrorPlugins() {
        return [
            new Plugin({
                key: pluginKey,

                state: {
                    init() {
                        return DecorationSet.empty;
                    },
                    apply(tr, value) {
                        const meta = tr.getMeta(pluginKey);
                        if (meta) return meta;
                        if (tr.docChanged) return value.map(tr.mapping, tr.doc);
                        return value;
                    },
                },

                props: {
                    decorations(state) {
                        return pluginKey.getState(state);
                    },
                },

                view(view) {
                    let cachedPositions: number[] = [];
                    let rafId: number | null = null;
                    let resizeObserver: ResizeObserver | null = null;

                    function recalculate() {
                        if (rafId !== null) cancelAnimationFrame(rafId);
                        rafId = requestAnimationFrame(() => {
                            rafId = null;
                            const positions = measureAndComputeBreaks(view);
                            if (arraysEqual(positions, cachedPositions)) return;
                            cachedPositions = positions;

                            const decorations = positions.map(pos =>
                                Decoration.widget(pos, createPageBreakElement, { side: -1 })
                            );
                            const decoSet = DecorationSet.create(view.state.doc, decorations);
                            view.dispatch(view.state.tr.setMeta(pluginKey, decoSet));
                        });
                    }

                    resizeObserver = new ResizeObserver(() => {
                        recalculate();
                    });
                    resizeObserver.observe(view.dom);

                    // Initial calculation after first render
                    recalculate();

                    return {
                        update(view, prevState) {
                            if (!view.state.doc.eq(prevState.doc)) {
                                recalculate();
                            }
                        },
                        destroy() {
                            if (rafId !== null) cancelAnimationFrame(rafId);
                            if (resizeObserver) resizeObserver.disconnect();
                        },
                    };
                },
            }),
        ];
    },
});
