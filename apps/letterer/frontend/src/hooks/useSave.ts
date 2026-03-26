import { useState, useEffect, useCallback, useRef } from 'react';
import { save_content } from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import type { Editor } from '@tiptap/react';

export type SaveStatus = 'saved' | 'unsaved' | 'saving';

export function useSave(editor: Editor | null, docId: string) {
    const [status, setStatus] = useState<SaveStatus>('saved');
    const statusRef = useRef<SaveStatus>('saved');
    const contentDirtyRef = useRef(false);

    useEffect(() => {
        if (!editor) return;
        const handler = () => {
            contentDirtyRef.current = true;
            statusRef.current = 'unsaved';
            setStatus('unsaved');
        };
        editor.on('update', handler);
        return () => { editor.off('update', handler); };
    }, [editor]);

    const markUnsaved = useCallback(() => {
        statusRef.current = 'unsaved';
        setStatus('unsaved');
    }, []);

    const save = useCallback(async () => {
        if (!editor || statusRef.current === 'saving') return;
        statusRef.current = 'saving';
        setStatus('saving');
        if (contentDirtyRef.current) {
            const json = editor.getJSON();
            const jsonStr = JSON.stringify(json);
            const txHash = await save_content(docId, jsonStr);
            if (txHash) {
                await await_tx_inclusion(txHash);
            }
            contentDirtyRef.current = false;
        }
        statusRef.current = 'saved';
        setStatus('saved');
    }, [editor, docId]);

    return { status, save, markUnsaved };
}
