import { useState } from 'react';
import { RouterProvider, Outlet, createMemoryRouter, Navigate } from 'react-router-dom';
import MapPage from './pages/MapPage';
import WelcomeModal from './components/common/WelcomeModal';
import './styles/index.css';
import { createVastrumReactRouter } from '@vastrum/react-lib';

function Layout() {
    const [showWelcomeModal, setShowWelcomeModal] = useState(true);

    return (
        <>
            <Outlet />
            <WelcomeModal isOpen={showWelcomeModal} onClose={() => setShowWelcomeModal(false)} />
        </>
    );
}

const routes = [
    {
        element: <Layout />,
        children: [
            { path: '/', element: <MapPage /> },
            { path: '*', element: <Navigate to="/" replace /> },
        ],
    },
];

export const router = await createVastrumReactRouter(routes, createMemoryRouter);

function App() {
    return <RouterProvider router={router} />;
}

export default App;
