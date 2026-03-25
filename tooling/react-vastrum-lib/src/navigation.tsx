import { get_current_path, update_current_path } from '../wasm/pkg';

type RouterCreator<TRouter> = (routes: any[], opts?: { initialEntries?: string[] }) => TRouter;

export async function createVastrumReactRouter<TRouter extends { subscribe: Function; navigate: Function }>(
    routes: any[],
    createRouterFn: RouterCreator<TRouter>,
): Promise<TRouter> {
    let initialPath = await get_current_path();
    initialPath = initialPath || '/';
    let lastSyncedPath = initialPath;

    const router = createRouterFn(routes, {
        initialEntries: [initialPath],
    });

    // Handle in app/react navigation
    router.subscribe((state: any) => {
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

    // Handle browser navigation (forward, backward)
    window.addEventListener('wasm-navigate', ((event: CustomEvent<string>) => {
        router.navigate(event.detail, { state: { fromWasm: true } });
    }) as EventListener);

    return router;
}