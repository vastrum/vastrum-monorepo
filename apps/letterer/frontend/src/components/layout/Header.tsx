import React from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { FileText, ChevronLeft } from 'lucide-react';

function Header(): React.JSX.Element {
    const navigate = useNavigate();
    const location = useLocation();
    const isHome = location.pathname === '/';

    return (
        <header style={{
            display: 'flex',
            alignItems: 'center',
            gap: 12,
            padding: '12px 24px',
            borderBottom: '1px solid #e5e7eb',
            backgroundColor: '#fff',
        }}>
            {!isHome && (
                <button
                    onClick={() => navigate('/')}
                    style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: 4,
                        background: 'none',
                        border: 'none',
                        color: '#6b7280',
                        fontSize: 14,
                        padding: '4px 8px',
                        borderRadius: 6,
                    }}
                    className="header-btn"
                >
                    <ChevronLeft size={18} />
                    Documents
                </button>
            )}
            {isHome && (
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <FileText size={22} color="#2563eb" />
                    <span style={{ fontSize: 18, fontWeight: 600, color: '#111' }}>Letterer</span>
                </div>
            )}
        </header>
    );
}

export default Header;
