import { useState } from 'react';
import { RouterProvider, Outlet, createMemoryRouter, Navigate } from 'react-router-dom';
import Header from './components/layout/Header';
import CategoryList from './pages/CategoryList';
import ForumHome from './pages/ForumHome';
import PostPage from './pages/PostPage';
import WelcomeModal from './components/common/WelcomeModal';
import './styles/index.css';
import { createVastrumReactRouter } from '@vastrum/react-lib';

function Layout() {
    const [showWelcomeModal, setShowWelcomeModal] = useState(true);

    return (
        <div style={{ minHeight: '100vh', backgroundColor: '#eef0f3' }}>
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
            { path: '/', element: <CategoryList /> },
            { path: '/category/:category', element: <ForumHome /> },
            { path: '/category/:category/topic/:id', element: <PostPage /> },
            { path: '*', element: <Navigate to="/" replace /> },
        ],
    },
];


export const router = await createVastrumReactRouter(routes, createMemoryRouter);


function App() {
    return <RouterProvider router={router} />;
}

export default App;
