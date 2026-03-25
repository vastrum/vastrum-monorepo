import { useState } from 'react';
import { RouterProvider, Outlet, createMemoryRouter, Navigate } from 'react-router-dom';
import Header from './components/layout/Header';
import SearchBar from './components/layout/SearchBar';
import Home from './pages/Home';
import BlockDetail from './pages/BlockDetail';
import TxDetail from './pages/TxDetail';
import AccountPage from './pages/AccountPage';
import SitesList from './pages/SitesList';
import SiteDetail from './pages/SiteDetail';
import BlocksList from './pages/BlocksList';
import TransactionsList from './pages/TransactionsList';
import WelcomeModal from './components/common/WelcomeModal';
import './styles/index.css';
import { createVastrumReactRouter } from '@vastrum/react-lib';

function Layout() {
    const [showWelcomeModal, setShowWelcomeModal] = useState(true);

    return (
        <div className="min-h-screen bg-blocker-bg">
            <Header />
            <div className="max-w-6xl mx-auto px-2 sm:px-4 py-4">
                <SearchBar />
                <Outlet />
            </div>
            <WelcomeModal isOpen={showWelcomeModal} onClose={() => setShowWelcomeModal(false)} />
        </div>
    );
}

const routes = [
    {
        element: <Layout />,
        children: [
            { path: '/', element: <Home /> },
            { path: '/blocks', element: <BlocksList /> },
            { path: '/block/:height', element: <BlockDetail /> },
            { path: '/transactions', element: <TransactionsList /> },
            { path: '/tx/:hash', element: <TxDetail /> },
            { path: '/account/:pubkey', element: <AccountPage /> },
            { path: '/sites', element: <SitesList /> },
            { path: '/site/:id', element: <SiteDetail /> },
            { path: '*', element: <Navigate to="/" replace /> },
        ],
    },
];

export const router = await createVastrumReactRouter(routes, createMemoryRouter);

function App() {
    return <RouterProvider router={router} />;
}

export default App;
