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
            { path: '/blocks', element: <BlocksList />, handle: { title: 'Blocks' } },
            { path: '/block/:height', element: <BlockDetail />, handle: { title: 'Block #:height' } },
            { path: '/transactions', element: <TransactionsList />, handle: { title: 'Transactions' } },
            { path: '/tx/:hash', element: <TxDetail />, handle: { title: 'Transaction' } },
            { path: '/account/:pubkey', element: <AccountPage />, handle: { title: 'Account' } },
            { path: '/sites', element: <SitesList />, handle: { title: 'Sites' } },
            { path: '/site/:id', element: <SiteDetail />, handle: { title: 'Site' } },
            { path: '*', element: <Navigate to="/" replace /> },
        ],
    },
];

export const router = await createVastrumReactRouter(routes, createMemoryRouter, { titleTemplate: '%s - Blocker', defaultTitle: 'Blocker' });

function App() {
    return <RouterProvider router={router} />;
}

export default App;
