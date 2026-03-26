import React from 'react';

interface PaginationProps {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
}

function Pagination({ currentPage, totalPages, onPageChange }: PaginationProps): React.JSX.Element | null {
    if (totalPages <= 1) return null;

    return (
        <div style={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            gap: 12,
            padding: '20px 0',
        }}>
            <button
                className="page-btn"
                onClick={() => onPageChange(currentPage - 1)}
                disabled={currentPage <= 1}
                style={{
                    background: 'none',
                    border: '1px solid #ddd',
                    borderRadius: 4,
                    padding: '6px 12px',
                    fontSize: 13,
                    color: currentPage <= 1 ? '#ccc' : '#666',
                    cursor: currentPage <= 1 ? 'default' : 'pointer',
                }}
            >
                Prev
            </button>
            <span style={{ fontSize: 13, color: '#666' }}>
                Page {currentPage} of {totalPages}
            </span>
            <button
                className="page-btn"
                onClick={() => onPageChange(currentPage + 1)}
                disabled={currentPage >= totalPages}
                style={{
                    background: 'none',
                    border: '1px solid #ddd',
                    borderRadius: 4,
                    padding: '6px 12px',
                    fontSize: 13,
                    color: currentPage >= totalPages ? '#ccc' : '#666',
                    cursor: currentPage >= totalPages ? 'default' : 'pointer',
                }}
            >
                Next
            </button>
        </div>
    );
}

export default Pagination;
