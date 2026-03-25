import { createContext, useContext, useState, useCallback, useEffect, useRef, type ReactNode } from 'react';
import { useLocation } from 'react-router-dom';

interface MobileSidebarState {
    sidebarOpen: boolean;
    openSidebar: () => void;
    closeSidebar: () => void;
}

const MobileSidebarContext = createContext<MobileSidebarState>(null!);

export function useMobileSidebar() {
    return useContext(MobileSidebarContext);
}

export function MobileSidebarProvider({ children }: { children: ReactNode }) {
    const [sidebarOpen, setSidebarOpen] = useState(true);
    const location = useLocation();
    const isFirstRender = useRef(true);

    useEffect(() => {
        if (isFirstRender.current) {
            isFirstRender.current = false;
            return;
        }
        setSidebarOpen(false);
    }, [location.pathname]);

    const openSidebar = useCallback(() => setSidebarOpen(true), []);
    const closeSidebar = useCallback(() => setSidebarOpen(false), []);

    return (
        <MobileSidebarContext.Provider value={{ sidebarOpen, openSidebar, closeSidebar }}>
            {children}
        </MobileSidebarContext.Provider>
    );
}
