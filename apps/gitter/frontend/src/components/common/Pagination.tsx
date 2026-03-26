import React from 'react';

interface PaginationProps {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
}

function Pagination({ currentPage, totalPages, onPageChange }: PaginationProps): React.JSX.Element | null {
    if (totalPages <= 1) return null;

    return (
        <div className="flex justify-center items-center gap-3 py-5">
            <button
                onClick={() => onPageChange(currentPage - 1)}
                disabled={currentPage <= 1}
                className={`px-3 py-1.5 text-sm border border-app-border rounded ${
                    currentPage <= 1
                        ? 'text-app-text-secondary cursor-default'
                        : 'text-app-text-primary hover:bg-app-hover cursor-pointer'
                }`}
            >
                Prev
            </button>
            <span className="text-sm text-app-text-secondary">
                Page {currentPage} of {totalPages}
            </span>
            <button
                onClick={() => onPageChange(currentPage + 1)}
                disabled={currentPage >= totalPages}
                className={`px-3 py-1.5 text-sm border border-app-border rounded ${
                    currentPage >= totalPages
                        ? 'text-app-text-secondary cursor-default'
                        : 'text-app-text-primary hover:bg-app-hover cursor-pointer'
                }`}
            >
                Next
            </button>
        </div>
    );
}

export default Pagination;
