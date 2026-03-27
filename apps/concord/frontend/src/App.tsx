import { useState } from 'react';
import { RouterProvider, Outlet, createMemoryRouter, Navigate } from 'react-router-dom';
import ServerView from './pages/ServerView';
import DmList from './pages/DmList';
import DmView from './pages/DmView';
import JoinServer from './pages/JoinServer';
import ServerSidebar from './components/layout/ServerSidebar';
import { UnreadProvider } from './context/UnreadProvider';
import { MobileSidebarProvider, useMobileSidebar } from './context/MobileSidebarProvider';
import WelcomeModal from './components/common/WelcomeModal';
import './styles/index.css';
import 'highlight.js/styles/github-dark.css';
import { createVastrumReactRouter } from '@vastrum/react-lib';

function Layout() {
    const [showWelcomeModal, setShowWelcomeModal] = useState(true);

    return (
        <UnreadProvider>
            <MobileSidebarProvider>
                <LayoutInner />
            </MobileSidebarProvider>
            <WelcomeModal isOpen={showWelcomeModal} onClose={() => setShowWelcomeModal(false)} />
        </UnreadProvider>
    );
}

function LayoutInner() {
    const { sidebarOpen, closeSidebar } = useMobileSidebar();

    return (
        <div className="flex h-screen overflow-hidden bg-dc-bg-primary">
            {sidebarOpen && (
                <div className="fixed inset-0 bg-black/50 z-40 md:hidden" onClick={closeSidebar} />
            )}
            <div className={`fixed inset-y-0 left-0 z-50 flex transition-transform duration-200 ${sidebarOpen ? 'translate-x-0' : '-translate-x-full'} md:relative md:translate-x-0 md:z-auto md:transition-none`}>
                <ServerSidebar />
            </div>
            <div className="flex-1 flex overflow-hidden min-w-0">
                <Outlet />
            </div>
        </div>
    );
}

const routes = [
    {
        element: <Layout />,
        children: [
            { path: '/', element: <DmList /> },
            { path: '/server/:serverId', element: <ServerView /> },
            { path: '/server/:serverId/:channelId', element: <ServerView /> },
            { path: '/join/:serverId/:serverKey', element: <JoinServer /> },
            { path: '/dms', element: <DmList /> },
            { path: '/dms/:dmKey', element: <DmView /> },
            { path: '*', element: <Navigate to="/" replace /> },
        ],
    },
];

export const router = await createVastrumReactRouter(routes, createMemoryRouter);

function App() {
    return <RouterProvider router={router} />;
}

export default App;
