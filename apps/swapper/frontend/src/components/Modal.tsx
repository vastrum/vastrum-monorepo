import { useState, useEffect, useRef, type ReactNode } from 'react';
import { CloseIcon } from './Icons';

interface ModalProps {
    isOpen: boolean;
    onClose: () => void;
    title: string;
    children: ReactNode;
}

export function Modal({ isOpen, onClose, title, children }: ModalProps) {
    const contentRef = useRef<HTMLDivElement>(null);
    const [showScrollIndicator, setShowScrollIndicator] = useState(false);

    useEffect(() => {
        const checkScroll = (): void => {
            if (contentRef.current) {
                const { scrollTop, scrollHeight, clientHeight } = contentRef.current;
                const isScrollable = scrollHeight > clientHeight;
                const isAtBottom = scrollHeight - scrollTop - clientHeight < 10;
                setShowScrollIndicator(isScrollable && !isAtBottom);
            }
        };

        checkScroll();

        const currentRef = contentRef.current;
        if (currentRef) {
            currentRef.addEventListener('scroll', checkScroll);
            window.addEventListener('resize', checkScroll);
        }

        return () => {
            if (currentRef) {
                currentRef.removeEventListener('scroll', checkScroll);
            }
            window.removeEventListener('resize', checkScroll);
        };
    }, [isOpen, children]);

    if (!isOpen) {
        return null;
    }

    return (
        <div className="fixed inset-0 flex items-center justify-center z-50 p-4">
            <div className="bg-app-bg-secondary border border-app-border rounded-lg max-w-full md:max-w-2xl lg:max-w-4xl w-full max-h-[90vh] overflow-hidden relative">
                {/* Modal Header */}
                <div className="flex items-center justify-between px-4 py-3 md:p-6 border-b border-app-border bg-app-bg-secondary">
                    <h2 className="text-lg md:text-xl font-semibold text-app-text-primary">{title}</h2>
                    <button
                        type="button"
                        onClick={onClose}
                        className="text-app-text-secondary hover:text-app-text-primary transition-colors"
                    >
                        <CloseIcon />
                    </button>
                </div>

                {/* Modal Content */}
                <div ref={contentRef} className="overflow-y-auto scrollbar-thin max-h-[calc(90vh-88px)]">
                    {children}
                </div>

                {/* Scroll Indicator */}
                {showScrollIndicator && (
                    <div className="absolute bottom-0 left-0 right-0 pointer-events-none">
                        <div className="h-20 bg-gradient-to-t from-app-bg-secondary via-app-bg-secondary/60 to-transparent" />
                        <div className="absolute bottom-3 left-1/2 -translate-x-1/2 flex flex-col items-center gap-0.5 opacity-40">
                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-app-text-secondary">
                                <path d="M6 9l6 6 6-6" />
                            </svg>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
