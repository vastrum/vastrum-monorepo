import React from 'react';
import type { Editor } from '@tiptap/react';
import type { SaveStatus } from '@/hooks/useSave';
import {
    Bold, Italic, Underline as UnderlineIcon, Strikethrough, Code,
    Heading1, Heading2, Heading3,
    List, ListOrdered, Quote, Minus,
    AlignLeft, AlignCenter, AlignRight, Highlighter,
    Save, Share2,
} from 'lucide-react';

interface ToolbarProps {
    editor: Editor;
    saveStatus: SaveStatus;
    onSave: () => void;
    onShare: () => void;
    readOnly: boolean;
}

function ToolbarButton({ active, disabled, onClick, children, title }: {
    active?: boolean;
    disabled?: boolean;
    onClick: () => void;
    children: React.ReactNode;
    title: string;
}) {
    return (
        <button
            onClick={onClick}
            disabled={disabled}
            title={title}
            style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                width: 32,
                height: 32,
                border: 'none',
                borderRadius: 4,
                background: active ? '#e5e7eb' : 'transparent',
                color: disabled ? '#d1d5db' : '#374151',
                cursor: disabled ? 'default' : 'pointer',
            }}
            className="toolbar-btn"
        >
            {children}
        </button>
    );
}

function Divider() {
    return <div style={{ width: 1, height: 24, backgroundColor: '#e5e7eb', margin: '0 4px' }} />;
}

function Toolbar({ editor, saveStatus, onSave, onShare, readOnly }: ToolbarProps): React.JSX.Element {
    const statusText = saveStatus === 'saving' ? 'Saving...'
        : saveStatus === 'unsaved' ? 'Unsaved changes'
        : 'Saved';

    const statusColor = saveStatus === 'saving' ? '#f59e0b'
        : saveStatus === 'unsaved' ? '#ef4444'
        : '#10b981';

    return (
        <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: 2,
            padding: '8px 16px',
            borderBottom: '1px solid #e5e7eb',
            backgroundColor: '#fafafa',
            flexWrap: 'wrap',
        }}>
            <button
                onClick={onSave}
                disabled={readOnly || saveStatus === 'saving' || saveStatus === 'saved'}
                style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 6,
                    padding: '6px 14px',
                    border: 'none',
                    borderRadius: 6,
                    backgroundColor: saveStatus === 'unsaved' ? '#2563eb' : '#e5e7eb',
                    color: saveStatus === 'unsaved' ? '#fff' : '#6b7280',
                    fontSize: 13,
                    fontWeight: 500,
                    cursor: readOnly || saveStatus !== 'unsaved' ? 'default' : 'pointer',
                }}
            >
                <Save size={15} />
                Save
            </button>
            <span style={{ fontSize: 12, color: statusColor, marginLeft: 4, marginRight: 8 }}>
                {statusText}
            </span>

            <Divider />

            <ToolbarButton title="Bold" active={editor.isActive('bold')} onClick={() => editor.chain().focus().toggleBold().run()} disabled={readOnly}>
                <Bold size={16} />
            </ToolbarButton>
            <ToolbarButton title="Italic" active={editor.isActive('italic')} onClick={() => editor.chain().focus().toggleItalic().run()} disabled={readOnly}>
                <Italic size={16} />
            </ToolbarButton>
            <ToolbarButton title="Underline" active={editor.isActive('underline')} onClick={() => editor.chain().focus().toggleUnderline().run()} disabled={readOnly}>
                <UnderlineIcon size={16} />
            </ToolbarButton>
            <ToolbarButton title="Strikethrough" active={editor.isActive('strike')} onClick={() => editor.chain().focus().toggleStrike().run()} disabled={readOnly}>
                <Strikethrough size={16} />
            </ToolbarButton>
            <ToolbarButton title="Highlight" active={editor.isActive('highlight')} onClick={() => editor.chain().focus().toggleHighlight().run()} disabled={readOnly}>
                <Highlighter size={16} />
            </ToolbarButton>
            <ToolbarButton title="Code" active={editor.isActive('code')} onClick={() => editor.chain().focus().toggleCode().run()} disabled={readOnly}>
                <Code size={16} />
            </ToolbarButton>

            <Divider />

            <ToolbarButton title="Heading 1" active={editor.isActive('heading', { level: 1 })} onClick={() => editor.chain().focus().toggleHeading({ level: 1 }).run()} disabled={readOnly}>
                <Heading1 size={16} />
            </ToolbarButton>
            <ToolbarButton title="Heading 2" active={editor.isActive('heading', { level: 2 })} onClick={() => editor.chain().focus().toggleHeading({ level: 2 }).run()} disabled={readOnly}>
                <Heading2 size={16} />
            </ToolbarButton>
            <ToolbarButton title="Heading 3" active={editor.isActive('heading', { level: 3 })} onClick={() => editor.chain().focus().toggleHeading({ level: 3 }).run()} disabled={readOnly}>
                <Heading3 size={16} />
            </ToolbarButton>

            <Divider />

            <ToolbarButton title="Bullet List" active={editor.isActive('bulletList')} onClick={() => editor.chain().focus().toggleBulletList().run()} disabled={readOnly}>
                <List size={16} />
            </ToolbarButton>
            <ToolbarButton title="Ordered List" active={editor.isActive('orderedList')} onClick={() => editor.chain().focus().toggleOrderedList().run()} disabled={readOnly}>
                <ListOrdered size={16} />
            </ToolbarButton>
            <ToolbarButton title="Blockquote" active={editor.isActive('blockquote')} onClick={() => editor.chain().focus().toggleBlockquote().run()} disabled={readOnly}>
                <Quote size={16} />
            </ToolbarButton>
            <ToolbarButton title="Code Block" active={editor.isActive('codeBlock')} onClick={() => editor.chain().focus().toggleCodeBlock().run()} disabled={readOnly}>
                <Code size={16} />
            </ToolbarButton>
            <ToolbarButton title="Horizontal Rule" onClick={() => editor.chain().focus().setHorizontalRule().run()} disabled={readOnly}>
                <Minus size={16} />
            </ToolbarButton>

            <Divider />

            <ToolbarButton title="Align Left" active={editor.isActive({ textAlign: 'left' })} onClick={() => editor.chain().focus().setTextAlign('left').run()} disabled={readOnly}>
                <AlignLeft size={16} />
            </ToolbarButton>
            <ToolbarButton title="Align Center" active={editor.isActive({ textAlign: 'center' })} onClick={() => editor.chain().focus().setTextAlign('center').run()} disabled={readOnly}>
                <AlignCenter size={16} />
            </ToolbarButton>
            <ToolbarButton title="Align Right" active={editor.isActive({ textAlign: 'right' })} onClick={() => editor.chain().focus().setTextAlign('right').run()} disabled={readOnly}>
                <AlignRight size={16} />
            </ToolbarButton>

            <div style={{ flex: 1 }} />

            <button
                onClick={onShare}
                style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 6,
                    padding: '6px 14px',
                    border: '1px solid #e5e7eb',
                    borderRadius: 6,
                    backgroundColor: '#fff',
                    color: '#374151',
                    fontSize: 13,
                    fontWeight: 500,
                }}
                className="toolbar-btn"
            >
                <Share2 size={15} />
                Share
            </button>
        </div>
    );
}

export default Toolbar;
