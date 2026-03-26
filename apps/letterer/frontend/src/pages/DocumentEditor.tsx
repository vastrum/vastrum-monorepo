import React, { useState, useEffect, useRef, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Underline from '@tiptap/extension-underline';
import Link from '@tiptap/extension-link';
import Image from '@tiptap/extension-image';
import Placeholder from '@tiptap/extension-placeholder';
import TextAlign from '@tiptap/extension-text-align';
import Highlight from '@tiptap/extension-highlight';
import { PageBreaks } from '@/extensions/PageBreaks';
import Toolbar from '@/components/editor/Toolbar';
import ShareModal from '@/components/common/ShareModal';
import { useSave } from '@/hooks/useSave';
import { get_document_meta, get_document_content, rename_document } from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import type { JSDocumentMeta } from '../../wasm/pkg';

function DocumentEditor(): React.JSX.Element {
    const { id } = useParams<{ id: string }>();
    const docId = id!;
    const navigate = useNavigate();
    const [meta, setMeta] = useState<JSDocumentMeta | null>(null);
    const [title, setTitle] = useState('');
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [shareOpen, setShareOpen] = useState(false);
    const contentLoaded = useRef(false);

    const editor = useEditor({
        extensions: [
            StarterKit,
            Underline,
            Link.configure({ openOnClick: false }),
            Image,
            Placeholder.configure({ placeholder: 'Start writing...' }),
            TextAlign.configure({ types: ['heading', 'paragraph'] }),
            Highlight,
            PageBreaks,
        ],
    });

    const { status: saveStatus, save, markUnsaved } = useSave(editor, docId);

    useEffect(() => {
        if (!editor || contentLoaded.current) return;
        (async () => {
            try {
                const m = await get_document_meta(docId);
                if (!m) {
                    setError('You don\'t have access to this document');
                    setLoading(false);
                    return;
                }
                setMeta(m);
                setTitle(m.title);
                const content = await get_document_content(docId);
                if (content && content.length > 0) {
                    const json = JSON.parse(content);
                    editor.commands.setContent(json);
                }
                contentLoaded.current = true;
                setLoading(false);
            } catch {
                setError('Failed to load document');
                setLoading(false);
            }
        })();
    }, [editor, docId]);

    const handleSave = useCallback(async () => {
        if (meta && title !== meta.title) {
            const txHash = await rename_document(docId, title);
            if (txHash) {
                await await_tx_inclusion(txHash);
                const m = await get_document_meta(docId);
                if (m) setMeta(m);
            }
        }
        await save();
    }, [meta, title, docId, save]);


    if (loading || !editor) {
        return (
            <div style={{ display: 'flex', justifyContent: 'center', padding: 60 }}>
                <p style={{ color: '#9ca3af' }}>Loading document...</p>
            </div>
        );
    }

    if (error) {
        return (
            <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                padding: 60,
            }}>
                <div style={{ textAlign: 'center' }}>
                    <p style={{ color: '#ef4444', marginBottom: 16 }}>{error}</p>
                    <button
                        onClick={() => navigate('/')}
                        style={{
                            color: '#2563eb',
                            background: 'none',
                            border: 'none',
                            fontSize: 14,
                            cursor: 'pointer',
                        }}
                    >
                        Back to Documents
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: 'calc(100vh - 53px)' }}>
            <div style={{
                padding: '12px 24px 0',
                backgroundColor: '#fff',
            }}>
                <input
                    value={title}
                    onChange={e => { setTitle(e.target.value); markUnsaved(); }}
                    placeholder="Untitled"
                    style={{
                        width: '100%',
                        border: 'none',
                        outline: 'none',
                        fontSize: 24,
                        fontWeight: 600,
                        color: '#111',
                        backgroundColor: 'transparent',
                        padding: '4px 0',
                    }}
                />
            </div>

            <Toolbar
                editor={editor}
                saveStatus={saveStatus}
                onSave={handleSave}
                onShare={() => setShareOpen(true)}
                readOnly={false}
            />

            <div style={{
                flex: 1,
                overflow: 'auto',
                backgroundColor: '#f3f4f6',
            }}>
                <div className="editor-container">
                    <EditorContent editor={editor} />
                </div>
            </div>

            <ShareModal
                isOpen={shareOpen}
                onClose={() => setShareOpen(false)}
                docId={docId}
            />
        </div>
    );
}

export default DocumentEditor;
