import React, { type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import { X } from 'lucide-react';

interface ModalProps {
    isOpen: boolean;
    onClose: () => void;
    title: string;
    children: ReactNode;
}

function Modal({ isOpen, onClose, title, children }: ModalProps): React.JSX.Element | null {
    if (!isOpen) return null;

    return createPortal(
        <div
            className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4"
            onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
        >
            <div className="bg-dc-bg-primary rounded-lg max-w-md w-full max-h-[90vh] overflow-hidden shadow-xl">
                <div className="flex items-center justify-between px-4 py-3">
                    <h2 className="text-xl font-bold text-white">{title}</h2>
                    <button
                        onClick={onClose}
                        className="text-dc-text-muted hover:text-dc-text p-1 rounded transition-colors"
                    >
                        <X size={20} />
                    </button>
                </div>
                <div className="overflow-y-auto max-h-[calc(90vh-56px)]">
                    {children}
                </div>
            </div>
        </div>,
        document.body,
    );
}

export default Modal;
