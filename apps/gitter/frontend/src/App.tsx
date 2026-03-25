import { useState } from 'react';
import { RouterProvider, Outlet, createMemoryRouter, Navigate } from 'react-router-dom';
import Header from './components/layout/Header';
import Repository from './pages/Repository';
import AllRepositories from './pages/AllRepositories';
import PullRequest from './pages/PullRequest';
import CreateRepository from './pages/CreateRepository';
import IssuePage from './pages/IssuePage';
import FileBrowser from './pages/FileBrowser';
import DiscussionPage from './pages/DiscussionPage';
import WelcomeModal from './components/common/WelcomeModal';
import './styles/index.css';
import 'highlight.js/styles/github-dark.css';
import { createVastrumReactRouter } from '@vastrum/react-lib';



function Layout() {
    const [showWelcomeModal, setShowWelcomeModal] = useState(true);

    return (
        <div className="min-h-screen bg-app-bg-primary overflow-x-hidden">
            <Header />
            <Outlet />
            <WelcomeModal isOpen={showWelcomeModal} onClose={() => setShowWelcomeModal(false)} />
        </div>
    );
}
const routes = [
    {
        element: (
            <Layout />
        ),
        children: [
            { path: '/', element: <AllRepositories /> },
            { path: '/repo/:repoId', element: <Repository /> },
            { path: '/repo/:repoId/code', element: <Repository /> },
            { path: '/repo/:repoId/issues', element: <Repository /> },
            { path: '/repo/:repoId/pulls', element: <Repository /> },
            { path: '/repo/:repoId/discussions', element: <Repository /> },
            { path: '/repo/:repoId/tree/*', element: <FileBrowser /> },
            { path: '/repo/:repoId/issue/:id', element: <IssuePage /> },
            { path: '/repo/:repoId/pull/:id', element: <PullRequest /> },
            { path: '/repo/:repoId/discussion/:id', element: <DiscussionPage /> },
            { path: '/new', element: <CreateRepository /> },
            { path: '*', element: <Navigate to="/" replace /> },
        ],
    },
];


export const router = await createVastrumReactRouter(routes, createMemoryRouter);


function App() {
    return <RouterProvider router={router} />;
}

export default App;