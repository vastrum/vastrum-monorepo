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
            { path: '/', element: <AllRepositories />, handle: { title: 'Repositories' } },
            { path: '/repo/:repoId', element: <Repository />, handle: { title: ':repoId' } },
            { path: '/repo/:repoId/code', element: <Repository />, handle: { title: 'Code - :repoId' } },
            { path: '/repo/:repoId/issues', element: <Repository />, handle: { title: 'Issues - :repoId' } },
            { path: '/repo/:repoId/pulls', element: <Repository />, handle: { title: 'Pull requests - :repoId' } },
            { path: '/repo/:repoId/discussions', element: <Repository />, handle: { title: 'Discussions - :repoId' } },
            { path: '/repo/:repoId/tree/*', element: <FileBrowser />, handle: { title: 'Files - :repoId' } },
            { path: '/repo/:repoId/issue/:id', element: <IssuePage />, handle: { title: 'Issue #:id - :repoId' } },
            { path: '/repo/:repoId/pull/:id', element: <PullRequest />, handle: { title: 'Pull request #:id - :repoId' } },
            { path: '/repo/:repoId/discussion/:id', element: <DiscussionPage />, handle: { title: 'Discussion #:id - :repoId' } },
            { path: '/new', element: <CreateRepository />, handle: { title: 'New repository' } },
            { path: '*', element: <Navigate to="/" replace /> },
        ],
    },
];

export const router = await createVastrumReactRouter(routes, createMemoryRouter, {
    titleTemplate: '%s - Gitter',
    defaultTitle: 'Gitter',
});

function App() {
    return <RouterProvider router={router} />;
}

export default App;
