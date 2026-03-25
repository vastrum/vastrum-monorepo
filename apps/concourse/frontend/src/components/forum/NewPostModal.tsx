import React, { useState } from 'react';
import Modal from '../common/Modal';
import MarkdownRenderer from '../common/MarkdownRenderer';
import { create_post } from '../../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';

interface NewPostModalProps {
    isOpen: boolean;
    onClose: () => void;
    onRefresh: () => void;
    categoryName: string;
}

function NewPostModal({ isOpen, onClose, onRefresh, categoryName }: NewPostModalProps): React.JSX.Element {
    const [title, setTitle] = useState('');
    const [content, setContent] = useState('');
    const [submitting, setSubmitting] = useState(false);
    const [showPreview, setShowPreview] = useState(false);

    const handleSubmit = async (): Promise<void> => {
        if (!title.trim() || !content.trim() || submitting) return;
        setSubmitting(true);
        const txHash = await create_post(categoryName, title, content);
        setTitle('');
        setContent('');
        setSubmitting(false);
        onClose();
        await await_tx_inclusion(txHash);
        onRefresh();
    };

    const handleClose = (): void => {
        setTitle('');
        setContent('');
        setShowPreview(false);
        onClose();
    };

    return (
        <Modal isOpen={isOpen} onClose={handleClose} title="Create a new Topic">
            <div style={{ padding: 16 }}>
                <div style={{ marginBottom: 16 }}>
                    <input
                        type="text"
                        className="input-field"
                        placeholder="Type title, or paste a link here"
                        value={title}
                        onChange={(e) => setTitle(e.target.value)}
                    />
                </div>
                <div style={{ marginBottom: 16 }}>
                    <div style={{
                        display: 'flex',
                        gap: 8,
                        marginBottom: 8,
                    }}>
                        <button
                            type="button"
                            onClick={() => setShowPreview(false)}
                            style={{
                                background: !showPreview ? '#e9e9e9' : 'none',
                                border: 'none',
                                color: !showPreview ? '#333' : '#919191',
                                fontSize: 13,
                                fontWeight: 500,
                                cursor: 'pointer',
                                padding: '4px 10px',
                                borderRadius: 4,
                            }}
                        >
                            Write
                        </button>
                        <button
                            type="button"
                            onClick={() => setShowPreview(true)}
                            style={{
                                background: showPreview ? '#e9e9e9' : 'none',
                                border: 'none',
                                color: showPreview ? '#333' : '#919191',
                                fontSize: 13,
                                fontWeight: 500,
                                cursor: 'pointer',
                                padding: '4px 10px',
                                borderRadius: 4,
                            }}
                        >
                            Preview
                        </button>
                    </div>
                    {!showPreview ? (
                        <textarea
                            className="input-field"
                            style={{ minHeight: 200, resize: 'vertical' }}
                            placeholder="Type here. Use Markdown to format."
                            value={content}
                            onChange={(e) => setContent(e.target.value)}
                        />
                    ) : (
                        <div
                            style={{
                                minHeight: 200,
                                border: '1px solid #e9e9e9',
                                borderRadius: 4,
                                padding: '8px 12px',
                                backgroundColor: '#fafafa',
                            }}
                        >
                            {content.trim() ? (
                                <MarkdownRenderer content={content} />
                            ) : (
                                <span style={{ color: '#919191', fontSize: 14 }}>
                                    Nothing to preview
                                </span>
                            )}
                        </div>
                    )}
                </div>
                <div style={{
                    display: 'flex',
                    justifyContent: 'flex-end',
                    gap: 12,
                }}>
                    <button
                        type="button"
                        onClick={handleClose}
                        style={{
                            background: 'none',
                            border: 'none',
                            color: '#919191',
                            fontSize: 14,
                            cursor: 'pointer',
                            padding: '8px 12px',
                        }}
                    >
                        Cancel
                    </button>
                    <button
                        type="button"
                        onClick={handleSubmit}
                        disabled={!title.trim() || !content.trim() || submitting}
                        className="btn"
                        style={{
                            backgroundColor: '#08c',
                            color: 'white',
                            border: 'none',
                            borderRadius: 4,
                            padding: '8px 14px',
                            fontSize: 14,
                            fontWeight: 500,
                            cursor: 'pointer',
                            opacity: (!title.trim() || !content.trim() || submitting) ? 0.5 : 1,
                        }}
                    >
                        {submitting ? 'Creating...' : '+ Create Topic'}
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default NewPostModal;
