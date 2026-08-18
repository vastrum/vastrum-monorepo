import { useState } from 'react';
import { RouterProvider, Outlet, createMemoryRouter, Navigate } from 'react-router-dom';
import Header from './components/layout/Header';
import DocumentList from './pages/DocumentList';
import DocumentEditor from './pages/DocumentEditor';
import JoinDocument from './pages/JoinDocument';
import WelcomeModal from './components/common/WelcomeModal';
import './styles/index.css';
import { createVastrumReactRouter } from '@vastrum/react-lib';

function Layout() {
    const [showWelcomeModal, setShowWelcomeModal] = useState(true);

    return (
        <div style={{ minHeight: '100vh', backgroundColor: '#fff' }}>
            <Header />
            <Outlet />
            <WelcomeModal isOpen={showWelcomeModal} onClose={() => setShowWelcomeModal(false)} />
        </div>
    );
}

const routes = [
    {
        element: <Layout />,
        children: [
            { path: '/', element: <DocumentList />, handle: { title: 'Documents' } },
            { path: '/doc/:id', element: <DocumentEditor />, handle: { title: 'Document' } },
            { path: '/share/:docKey', element: <JoinDocument />, handle: { title: 'Shared document' } },
            { path: '*', element: <Navigate to="/" replace /> },
        ],
    },
];

export const router = await createVastrumReactRouter(routes, createMemoryRouter, { titleTemplate: '%s - Letterer', defaultTitle: 'Letterer' });

function App() {
    return <RouterProvider router={router} />;
}

export default App;
