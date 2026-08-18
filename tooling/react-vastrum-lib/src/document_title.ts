const TITLE_PLACEHOLDER = "%s";


export interface DocumentTitleOptions {
  titleTemplate?: string;
  defaultTitle?: string;
}

interface RouteMatch {
  route?: { handle?: { title?: unknown } };
  handle?: { title?: unknown };
  params?: Record<string, string | undefined>;
}

interface RouterState {
  matches?: RouteMatch[];
}

export function resolveDocumentTitle(
  state: RouterState,
  options: DocumentTitleOptions,
): string | null {
  const match = deepestTitledMatch(state.matches);
  const pageTitle = match === null ? "" : substituteParams(match.title, match.params);

  if (pageTitle.length === 0) {
    return options.defaultTitle ?? null;
  }
  if (typeof options.titleTemplate === "string") {
    return options.titleTemplate.split(TITLE_PLACEHOLDER).join(pageTitle);
  }
  return pageTitle;
}

export function applyDocumentTitle(state: RouterState, options: DocumentTitleOptions) {
  const title = resolveDocumentTitle(state, options);
  if (title !== null) {
    document.title = title;
  }
}

interface TitledMatch {
  title: string;
  params: Record<string, string | undefined>;
}

function deepestTitledMatch(matches: RouteMatch[] | undefined): TitledMatch | null {
  if (!Array.isArray(matches)) return null;
  for (let i = matches.length - 1; i >= 0; i--) {
    const match = matches[i];
    const title = match?.route?.handle?.title ?? match?.handle?.title;
    if (typeof title === "string" && title.trim().length > 0) {
      return { title: title.trim(), params: match.params ?? {} };
    }
  }
  return null;
}

function substituteParams(
  template: string,
  params: Record<string, string | undefined>,
): string {
  let out = template;
  for (const [key, value] of Object.entries(params)) {
    if (typeof value !== "string") continue;
    out = out.split(":" + key).join(value);
  }
  return out.replace(/:[A-Za-z0-9_]+/g, "").trim();
}
