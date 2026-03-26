import React from 'react';
import { Link } from 'react-router-dom';

function Header(): React.JSX.Element {
    return (
        <header style={{
            borderBottom: '3px solid #e45735',
            padding: '12px 20px',
            backgroundColor: '#fff',
        }}>
            <div style={{
                maxWidth: 1200,
                margin: '0 auto',
                display: 'flex',
                alignItems: 'center',
            }}>
                <Link to="/" style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 10,
                    textDecoration: 'none',
                }}>
                    <span style={{
                        fontSize: 22,
                        fontWeight: 400,
                        letterSpacing: '-0.5px',
                    }}>
                        <span>Concourse</span>
                    </span>
                </Link>
            </div>
        </header>
    );
}

export default Header;
