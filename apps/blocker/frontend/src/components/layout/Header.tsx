import { Link } from 'react-router-dom';
import { Blocks, Globe, ArrowLeftRight } from 'lucide-react';

function Header() {
    return (
        <header className="border-b border-blocker-border bg-blocker-surface">
            <div className="max-w-6xl mx-auto px-2 sm:px-4 h-14 flex items-center justify-between">
                <Link to="/" className="flex items-center gap-2 text-blocker-text-primary hover:text-blocker-accent transition-colors">
                    <Blocks size={20} className="text-blocker-accent" />
                    <span className="font-semibold text-lg">Blocker</span>
                </Link>
                <nav className="flex items-center gap-2 sm:gap-4 text-xs sm:text-sm">
                    <Link to="/blocks" className="flex items-center gap-1 text-blocker-text-secondary hover:text-blocker-text-primary transition-colors">
                        <Blocks size={14} />
                        Blocks
                    </Link>
                    <Link to="/transactions" className="flex items-center gap-1 text-blocker-text-secondary hover:text-blocker-text-primary transition-colors">
                        <ArrowLeftRight size={14} />
                        Transactions
                    </Link>
                    <Link to="/sites" className="flex items-center gap-1 text-blocker-text-secondary hover:text-blocker-text-primary transition-colors">
                        <Globe size={14} />
                        Sites
                    </Link>
                </nav>
            </div>
        </header>
    );
}

export default Header;
