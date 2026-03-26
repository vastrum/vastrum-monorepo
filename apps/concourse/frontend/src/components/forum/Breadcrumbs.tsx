import React from 'react';
import { Link } from 'react-router-dom';

interface BreadcrumbItem {
    label: string;
    to?: string;
}

function Breadcrumbs({ items }: { items: BreadcrumbItem[] }): React.JSX.Element {
    return (
        <nav style={{
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            fontSize: 13,
            color: '#919191',
            padding: '16px 0',
        }}>
            {items.map((item, idx) => (
                <span key={idx} style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                    {idx > 0 && <span>/</span>}
                    {item.to ? (
                        <Link
                            to={item.to}
                            style={{
                                color: '#919191',
                                textDecoration: 'none',
                            }}
                        >
                            {item.label}
                        </Link>
                    ) : (
                        <span style={{ color: '#666' }}>{item.label}</span>
                    )}
                </span>
            ))}
        </nav>
    );
}

export default Breadcrumbs;
