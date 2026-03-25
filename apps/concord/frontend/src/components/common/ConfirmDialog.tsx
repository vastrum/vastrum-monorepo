import React, { type ReactNode } from 'react';
import Modal from '@/components/common/Modal';

interface ConfirmDialogProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    title: string;
    message: ReactNode;
    confirmLabel?: string;
    danger?: boolean;
}

function ConfirmDialog({ isOpen, onClose, onConfirm, title, message, confirmLabel = 'Confirm', danger = false }: ConfirmDialogProps): React.JSX.Element | null {
    return (
        <Modal isOpen={isOpen} onClose={onClose} title={title}>
            <div className="p-4">
                <p className="text-sm text-dc-text mb-6">{message}</p>
                <div className="flex justify-end gap-3">
                    <button
                        onClick={onClose}
                        className="px-4 py-2 text-sm text-dc-text hover:underline"
                    >
                        Cancel
                    </button>
                    <button
                        onClick={onConfirm}
                        className={`px-4 py-2 text-sm text-white rounded transition-colors ${
                            danger
                                ? 'bg-dc-red hover:bg-dc-red/80'
                                : 'bg-dc-blurple hover:bg-dc-blurple/80'
                        }`}
                    >
                        {confirmLabel}
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default ConfirmDialog;
