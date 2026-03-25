import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { Plus, Menu, X } from 'lucide-react';

function Header(): React.JSX.Element {
    const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

    return (
        <header className="bg-app-bg-secondary border-b border-app-border px-4 py-3 md:px-6 md:py-4 lg:px-8 sticky top-0 z-50 backdrop-blur-sm">
            <div className="flex items-center gap-2 md:gap-4">
                <Link
                    to="/"
                    className="text-xl md:text-2xl lg:text-3xl font-bold text-app-text-primary flex items-center gap-2 hover:opacity-80 transition-opacity"
                >
                    Gitter
                </Link>

                {/* Desktop navigation */}
                <nav className="hidden lg:flex items-center gap-6 ml-auto">
                    <Link
                        to="/"
                        className="text-app-text-primary font-semibold text-sm px-3 py-2 rounded-md hover:bg-app-hover transition-colors"
                    >
                        View all repositories
                    </Link>
                    <Link to="/new" className="btn-primary flex items-center gap-2">
                        <Plus className="w-4 h-4" />
                        New Repository
                    </Link>
                </nav>

                {/* Mobile menu button */}
                <button
                    onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
                    className="lg:hidden ml-auto p-2 text-app-text-primary hover:bg-app-hover rounded-md transition-colors"
                    aria-label="Toggle menu"
                    aria-expanded={isMobileMenuOpen}
                >
                    {isMobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
                </button>
            </div>

            {/* Mobile menu dropdown */}
            {isMobileMenuOpen && (
                <nav className="lg:hidden mt-3 pt-3 border-t border-app-border">
                    <div className="flex flex-col gap-2">
                        <Link
                            to="/"
                            onClick={() => setIsMobileMenuOpen(false)}
                            className="text-app-text-primary font-semibold text-sm px-3 py-3 rounded-md hover:bg-app-hover transition-colors"
                        >
                            View all repositories
                        </Link>
                        <Link
                            to="/new"
                            onClick={() => setIsMobileMenuOpen(false)}
                            className="btn-primary flex items-center justify-center gap-2 py-3"
                        >
                            <Plus className="w-4 h-4" />
                            New Repository
                        </Link>
                    </div>
                </nav>
            )}
        </header>
    );
}

export default Header;
