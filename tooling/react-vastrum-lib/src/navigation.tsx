import { get_current_path, update_current_path } from '../wasm/pkg';
import { applyDocumentTitle, type DocumentTitleOptions } from './document_title';

type RouterCreator<TRouter> = (routes: any[], opts?: { initialEntries?: string[] }) => TRouter;

export async function createVastrumReactRouter<TRouter extends { subscribe: Function; navigate: Function; state?: any }>(
    routes: any[],
    createRouterFn: RouterCreator<TRouter>,
    titleOptions: DocumentTitleOptions = {},
): Promise<TRouter> {
    let initialPath = await get_current_path();
    initialPath = initialPath || '/';
    let lastSyncedPath = initialPath;

    const router = createRouterFn(routes, {
        initialEntries: [initialPath],
    });

    router.subscribe((state: any) => {
        applyDocumentTitle(state, titleOptions);

        const currentPath = state.location.pathname;
        if (state.location.state?.fromWasm) {
            lastSyncedPath = currentPath;
            return;
        }
        if (currentPath !== lastSyncedPath) {
            lastSyncedPath = currentPath;
            const replace = state.historyAction === "REPLACE";
            update_current_path(currentPath, replace);
        }
    });

    if (router.state !== undefined) {
        applyDocumentTitle(router.state, titleOptions);
    }

    window.addEventListener('wasm-navigate', ((event: CustomEvent<string>) => {
        router.navigate(event.detail, { state: { fromWasm: true } });
    }) as EventListener);

    return router;
}
