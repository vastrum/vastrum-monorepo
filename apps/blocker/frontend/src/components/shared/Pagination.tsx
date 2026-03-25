import { ChevronLeft, ChevronRight } from 'lucide-react';

interface PaginationProps {
    currentPage: number;
    totalItems: number;
    pageSize: number;
    onPageChange: (page: number) => void;
}

function Pagination({ currentPage, totalItems, pageSize, onPageChange }: PaginationProps) {
    const totalPages = Math.max(1, Math.ceil(totalItems / pageSize));
    if (totalPages <= 1) return null;

    return (
        <div className="flex items-center justify-center gap-2 mt-4 mb-4 px-2">
            <button
                onClick={() => onPageChange(currentPage - 1)}
                disabled={currentPage === 0}
                className="p-1.5 rounded border border-blocker-border text-blocker-text-secondary hover:bg-blocker-surface-hover disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
            >
                <ChevronLeft size={16} />
            </button>
            <span className="text-sm text-blocker-text-secondary px-3">
                Page {currentPage + 1} of {totalPages}
            </span>
            <button
                onClick={() => onPageChange(currentPage + 1)}
                disabled={currentPage >= totalPages - 1}
                className="p-1.5 rounded border border-blocker-border text-blocker-text-secondary hover:bg-blocker-surface-hover disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
            >
                <ChevronRight size={16} />
            </button>
        </div>
    );
}

export default Pagination;
