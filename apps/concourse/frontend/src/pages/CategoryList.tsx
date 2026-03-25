import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { type JSCategory, get_all_categories } from '../../wasm/pkg';
import { formatRelativeTime } from '../utils/timeUtils';

function CategoryList(): React.JSX.Element {
    const [categories, setCategories] = useState<JSCategory[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        (async () => {
            const cats = await get_all_categories();
            setCategories(cats);
            setLoading(false);
        })();
    }, []);

    if (loading) {
        return (
            <main style={{ maxWidth: 1200, margin: '0 auto', padding: '40px 20px' }}>
                <div style={{ textAlign: 'center', color: '#919191' }}>Loading...</div>
            </main>
        );
    }

    return (
        <main style={{ maxWidth: 1200, margin: '0 auto', padding: '0 20px 20px' }}>
            <div style={{ padding: '24px 0 16px' }}>
                <h2 style={{ fontSize: 18, fontWeight: 500, color: '#222', margin: 0 }}>
                    Categories
                </h2>
            </div>

            <div style={{
                backgroundColor: '#fff',
                borderRadius: 8,
                border: '1px solid #e1e3e5',
                padding: '0 20px',
            }}>
                {categories.length === 0 ? (
                    <div style={{
                        padding: '40px 0',
                        textAlign: 'center',
                        color: '#919191',
                    }}>
                        No categories yet.
                    </div>
                ) : (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                        {/* Table header */}
                        <div style={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            padding: '12px 0',
                            borderBottom: '2px solid #e9e9e9',
                            color: '#919191',
                            fontSize: 13,
                        }}>
                            <div style={{ fontWeight: 500, flex: 1 }}>Category</div>
                            <div style={{
                                display: 'flex',
                                gap: 40,
                                minWidth: 160,
                                justifyContent: 'flex-end',
                            }}>
                                <span style={{ width: 60, textAlign: 'center' }}>Topics</span>
                                <span style={{ width: 80, textAlign: 'center' }}>Latest</span>
                            </div>
                        </div>

                        {categories.map((cat) => (
                            <Link
                                key={cat.name}
                                to={`/category/${encodeURIComponent(cat.name)}`}
                                className="category-card"
                                style={{
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                    alignItems: 'center',
                                    padding: '16px 0',
                                    borderBottom: '1px solid #e9e9e9',
                                    textDecoration: 'none',
                                    color: 'inherit',
                                }}
                            >
                                <div style={{ flex: 1, minWidth: 0 }}>
                                    <div style={{
                                        fontSize: 16,
                                        fontWeight: 500,
                                        color: '#08c',
                                        marginBottom: 4,
                                    }}>
                                        {cat.name}
                                    </div>
                                    <div style={{
                                        fontSize: 13,
                                        color: '#919191',
                                        lineHeight: 1.4,
                                    }}>
                                        {cat.description}
                                    </div>
                                </div>
                                <div style={{
                                    display: 'flex',
                                    gap: 40,
                                    minWidth: 160,
                                    justifyContent: 'flex-end',
                                }}>
                                    <span style={{
                                        width: 60,
                                        textAlign: 'center',
                                        fontSize: 14,
                                        color: '#666',
                                    }}>
                                        {Number(cat.post_count)}
                                    </span>
                                    <span style={{
                                        width: 80,
                                        textAlign: 'center',
                                        fontSize: 14,
                                        color: '#666',
                                    }}>
                                        {cat.latest_activity ? formatRelativeTime(Number(cat.latest_activity)) : '-'}
                                    </span>
                                </div>
                            </Link>
                        ))}
                    </div>
                )}
            </div>
        </main>
    );
}

export default CategoryList;
