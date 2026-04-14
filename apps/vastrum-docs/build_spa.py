#!/usr/bin/env python3
"""
Build mdbook output into a single self-contained SPA HTML file.
Reads book/ output, extracts <main> from each page, inlines all CSS/JS/fonts,
combines into one HTML with hash-based routing. No external tools needed.
"""

import re
import os
import base64
import glob
import mimetypes

BOOK_DIR = 'book'
OUTPUT = 'out/vastrum-docs.html'


def get_pages_from_toc(toc_js):
    """Extract ordered page list from toc JS innerHTML hrefs."""
    hrefs = re.findall(r'href="([^"]+\.html)"', toc_js)
    return [h for h in hrefs if not h.startswith('http://') and not h.startswith('https://')]


def get_page_titles_from_toc(toc_js):
    """Extract href -> title mapping from toc JS. Strips the <strong> numbering prefix."""
    titles = {}
    for m in re.finditer(r'<a href="([^"]+\.html)"><strong[^>]*>[^<]*</strong>\s*([^<]+)</a>', toc_js):
        href, title = m.group(1), m.group(2).strip()
        if not href.startswith('http://') and not href.startswith('https://'):
            titles[href] = title
    return titles


def page_to_id(page):
    """Convert page filename to section ID: 'tech/consensus.html' -> 'tech/consensus'"""
    return page.replace('.html', '')


def read_file(path):
    with open(path, 'r') as f:
        return f.read()


def read_binary(path):
    with open(path, 'rb') as f:
        return f.read()


def b64_data_uri(filepath):
    """Read a file and return a base64 data URI."""
    data = read_binary(filepath)
    mime, _ = mimetypes.guess_type(filepath)
    if filepath.endswith('.woff2'):
        mime = 'font/woff2'
    elif filepath.endswith('.svg'):
        mime = 'image/svg+xml'
    elif filepath.endswith('.png'):
        mime = 'image/png'
    elif filepath.endswith('.mp4'):
        mime = 'video/mp4'
    elif filepath.endswith('.webm'):
        mime = 'video/webm'
    elif not mime:
        mime = 'application/octet-stream'
    return f'data:{mime};base64,{base64.b64encode(data).decode()}'


def inline_css_urls(css_text, css_dir):
    """Replace url('...') references in CSS with base64 data URIs."""
    def replace_url(m):
        url = m.group(1).strip("'\"")
        if url.startswith('data:') or url.startswith('http'):
            return m.group(0)
        filepath = os.path.normpath(os.path.join(css_dir, url))
        if not os.path.exists(filepath):
            # mdbook adds content hashes: "Inter-Regular.woff2" → "Inter-Regular-3100e775.woff2"
            stem, ext = os.path.splitext(filepath)
            candidates = glob.glob(f"{stem}-*{ext}")
            if candidates:
                filepath = candidates[0]
        if os.path.exists(filepath):
            return f"url('{b64_data_uri(filepath)}')"
        return m.group(0)
    return re.sub(r'url\(([^)]+)\)', replace_url, css_text)


def extract_main(html):
    """Extract content between <main> and </main> tags."""
    m = re.search(r'<main>(.*?)</main>', html, re.DOTALL)
    return m.group(1) if m else ''


def rewrite_content_links(content, page_path):
    """Rewrite relative .html links in page content to hash links."""
    page_dir = os.path.dirname(page_path)

    def replace_link(m):
        prefix = m.group(1)
        href = m.group(2)
        suffix = m.group(3)

        if href.startswith('http://') or href.startswith('https://') or href.startswith('#'):
            return m.group(0)

        anchor = ''
        if '#' in href:
            href, anchor = href.split('#', 1)
            anchor = '#' + anchor

        if href.endswith('.html'):
            resolved = os.path.normpath(os.path.join(page_dir, href))
            resolved = resolved.replace('\\', '/')
            section_id = resolved.replace('.html', '')
            return f'{prefix}#{section_id}{anchor}{suffix}'

        return m.group(0)

    return re.sub(r'(href=")([^"]*?)(")', replace_link, content)


def inline_images(content, page_path):
    """Replace <img src="..."> with inline base64 data URIs."""
    page_dir = os.path.dirname(os.path.join(BOOK_DIR, page_path))

    def replace_img_src(m):
        prefix = m.group(1)
        src = m.group(2)
        suffix = m.group(3)
        if src.startswith('data:') or src.startswith('http://') or src.startswith('https://'):
            return m.group(0)
        filepath = os.path.normpath(os.path.join(page_dir, src))
        if os.path.exists(filepath):
            return f'{prefix}{b64_data_uri(filepath)}{suffix}'
        print(f'Warning: image not found: {filepath}')
        return m.group(0)

    return re.sub(r'(src=")([^"]+?)(")', replace_img_src, content)


def rewrite_toc_links(toc_js):
    """Rewrite href="X.html" to href="#X" in TOC JS, skip http links."""
    def replace_toc_link(m):
        href = m.group(1)
        if href.startswith('http://') or href.startswith('https://'):
            return m.group(0)
        section_id = href.replace('.html', '')
        return f'href="#{section_id}"'

    return re.sub(r'href="([^"]*?)\.html"', replace_toc_link, toc_js)


def strip_nav(html):
    """Remove <nav class="nav-wrapper"...> and <nav class="nav-wide-wrapper"...> sections."""
    html = re.sub(r'<nav class="nav-wrapper".*?</nav>', '', html, flags=re.DOTALL)
    html = re.sub(r'<nav class="nav-wide-wrapper".*?</nav>', '', html, flags=re.DOTALL)
    return html


def strip_comments(html):
    """Remove HTML comments."""
    return re.sub(r'<!--.*?-->', '', html, flags=re.DOTALL)






def inline_favicon(shell, tag_pattern, attr, filepath):
    """Replace a favicon link tag with an inline data URI."""
    m = re.search(tag_pattern, shell)
    if m:
        full_path = os.path.join(BOOK_DIR, filepath)
        if os.path.exists(full_path):
            uri = b64_data_uri(full_path)
            shell = shell.replace(m.group(0), m.group(0).replace(m.group(1), uri))
    return shell


SPA_SCRIPT = '''
<script>
// SPA routing — no location.hash usage to avoid iframe history entry conflicts
(function() {
    var pages = PAGE_LIST_PLACEHOLDER;
    var currentPage = pages[0];

    function navigateTo(id) {
        id = id || pages[0];
        if (pages.indexOf(id) === -1) id = pages[0];
        currentPage = id;
        document.querySelectorAll('.spa-page').forEach(function(s) { s.style.display = 'none'; });
        var target = document.getElementById(id);
        if (target) {
            target.style.display = 'block';
            window.scrollTo(0, 0);
        }
        document.querySelectorAll('mdbook-sidebar-scrollbox a').forEach(function(a) {
            var aHref = a.getAttribute('href');
            if (aHref && aHref.startsWith('#')) {
                a.classList.toggle('active', aHref === '#' + id);
            }
        });
        document.querySelectorAll('.on-this-page').forEach(function(el) { el.remove(); });
    }

    // Navigate and sync URL bar with parent web-client
    function navigateAndSync(id, replace) {
        navigateTo(id);
        // Close sidebar on mobile after navigation
        if (window.innerWidth < 620) {
            var checkbox = document.getElementById('mdbook-sidebar-toggle-anchor');
            if (checkbox && checkbox.checked) {
                checkbox.click();
            }
        }
        var msg = JSON.stringify({
            request_id: Date.now(),
            method: 'UpdateCurrentPath',
            params: JSON.stringify({ path: '/' + currentPage, replace: !!replace })
        });
        window.parent.postMessage(msg, '*');
    }

    // Path -> page id mapping
    function pathToPageId(path) {
        if (!path || path === '/') return pages[0];
        var p = path.replace(/^\\//, '').replace(/\\.html$/, '');
        return pages.indexOf(p) !== -1 ? p : pages[0];
    }

    // Keyboard chapter navigation
    document.addEventListener('keydown', function(e) {
        if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
        if (window.search && window.search.hasFocus && window.search.hasFocus()) return;

        var idx = pages.indexOf(currentPage);
        if (idx === -1) idx = 0;

        if (e.key === 'ArrowRight') {
            e.preventDefault();
            e.stopImmediatePropagation();
            if (idx < pages.length - 1) navigateAndSync(pages[idx + 1]);
        } else if (e.key === 'ArrowLeft') {
            e.preventDefault();
            e.stopImmediatePropagation();
            if (idx > 0) navigateAndSync(pages[idx - 1]);
        }
    }, true);

    // Handle link clicks
    document.addEventListener('click', function(e) {
        var a = e.target.closest ? e.target.closest('a') : null;
        if (!a) return;
        var href = a.getAttribute('href');
        if (!href || !href.startsWith('#')) return;

        var target = href.substring(1);
        if (pages.indexOf(target) !== -1) {
            e.preventDefault();
            navigateAndSync(target);
            return;
        }
        // Heading anchor — scroll within current page
        var el = document.getElementById(target);
        if (el) {
            e.preventDefault();
            el.scrollIntoView({ behavior: 'smooth' });
        }
    });

    // Listen for PageNavigationEvent (browser back/forward)
    window.addEventListener('message', function(e) {
        if (typeof e.data !== 'string') return;
        try {
            var data = JSON.parse(e.data);
            if (data.method === 'PageNavigationEvent') {
                var params = JSON.parse(data.params);
                navigateTo(pathToPageId(params.path));
            }
        } catch(err) {}
    });

    // Initial navigation: request current path from web-client host
    function initNavigation() {
        var requestId = Date.now();
        var msg = JSON.stringify({
            request_id: requestId,
            method: 'GetCurrentPath',
            params: '{}'
        });

        function onResponse(e) {
            if (typeof e.data !== 'string') return;
            try {
                var data = JSON.parse(e.data);
                if (data.request_id === requestId && data.method === 'Response') {
                    window.removeEventListener('message', onResponse);
                    var params = JSON.parse(data.params);
                    var pageId = pathToPageId(params.path);
                    navigateTo(pageId);
                    // Sync normalized path (replace so no extra history entry)
                    var syncMsg = JSON.stringify({
                        request_id: Date.now(),
                        method: 'UpdateCurrentPath',
                        params: JSON.stringify({ path: '/' + pageId, replace: true })
                    });
                    window.parent.postMessage(syncMsg, '*');
                }
            } catch(err) {}
        }

        window.addEventListener('message', onResponse);
        window.parent.postMessage(msg, '*');
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initNavigation);
    } else {
        initNavigation();
    }
})();
</script>
'''

HEADER_SCAN_SCRIPT = '''
<script>
// SPA-aware header scanning (replaces the per-page header detection from toc.js)
(function() {
    window._spaUpdateHeaders = function(activePageId) {
        // Remove old on-this-page sections
        document.querySelectorAll('.on-this-page').forEach(function(el) { el.remove(); });
        // Remove current-header classes
        document.querySelectorAll('.current-header').forEach(function(el) {
            el.classList.remove('current-header');
        });

        var section = document.getElementById(activePageId);
        if (!section) return;

        var headers = Array.from(section.querySelectorAll('h2, h3, h4, h5, h6'))
            .filter(function(h) { return h.id !== '' && h.children.length && h.children[0].tagName === 'A'; });

        if (headers.length === 0) return;

        var activeLink = document.querySelector('mdbook-sidebar-scrollbox a.active');
        if (!activeLink) return;

        // Build header tree
        var stack = [];
        var firstLevel = parseInt(headers[0].tagName.charAt(1));
        for (var i = 1; i < firstLevel; i++) {
            var ol = document.createElement('ol');
            ol.classList.add('section');
            if (stack.length > 0) stack[stack.length - 1].ol.appendChild(ol);
            stack.push({level: i + 1, ol: ol});
        }

        for (var i = 0; i < headers.length; i++) {
            var header = headers[i];
            var level = parseInt(header.tagName.charAt(1));
            var currentLevel = stack[stack.length - 1].level;

            if (level > currentLevel) {
                for (var nextLevel = currentLevel + 1; nextLevel <= level; nextLevel++) {
                    var ol = document.createElement('ol');
                    ol.classList.add('section');
                    var last = stack[stack.length - 1];
                    var lastChild = last.ol.lastChild;
                    if (lastChild) lastChild.appendChild(ol);
                    else last.ol.appendChild(ol);
                    stack.push({level: nextLevel, ol: ol});
                }
            } else if (level < currentLevel) {
                while (stack.length > 1 && stack[stack.length - 1].level > level) stack.pop();
            }

            var li = document.createElement('li');
            li.classList.add('header-item', 'expanded');
            var span = document.createElement('span');
            span.classList.add('chapter-link-wrapper');
            var a = document.createElement('a');
            span.appendChild(a);
            a.href = '#' + header.id;
            a.classList.add('header-in-summary');
            var clone = header.children[0].cloneNode(true);
            clone.querySelectorAll('mark').forEach(function(mark) { mark.replaceWith.apply(mark, Array.from(mark.childNodes)); });
            a.append.apply(a, Array.from(clone.childNodes));
            li.appendChild(span);
            stack[stack.length - 1].ol.appendChild(li);
        }

        var onThisPage = document.createElement('div');
        onThisPage.classList.add('on-this-page');
        onThisPage.append(stack[0].ol);
        var activeItemSpan = activeLink.parentElement;
        activeItemSpan.after(onThisPage);
    };
})();
</script>
'''


def build_spa():
    # Read the shell template (index.html)
    shell = read_file(os.path.join(BOOK_DIR, 'index.html'))

    # --- Inline all CSS ---
    def inline_css_tag(m):
        href = m.group(1)
        css_path = os.path.join(BOOK_DIR, href)
        if not os.path.exists(css_path):
            return m.group(0)
        css_text = read_file(css_path)
        css_dir = os.path.dirname(css_path)
        css_text = inline_css_urls(css_text, css_dir)
        # Preserve the id attribute if present
        id_match = re.search(r'id="([^"]*)"', m.group(0))
        id_attr = f' id="{id_match.group(1)}"' if id_match else ''
        return f'<style{id_attr}>\n{css_text}\n</style>'

    shell = re.sub(
        r'<link rel="stylesheet"[^>]*href="([^"]+)"[^>]*/?>',
        inline_css_tag,
        shell
    )

    # --- Inline favicons ---
    # SVG favicon
    m = re.search(r'<link rel="icon" href="([^"]+)"', shell)
    if m:
        fav_path = os.path.join(BOOK_DIR, m.group(1))
        if os.path.exists(fav_path):
            shell = shell.replace(m.group(1), b64_data_uri(fav_path))
    # PNG favicon
    m = re.search(r'<link rel="shortcut icon" href="([^"]+)"', shell)
    if m:
        fav_path = os.path.join(BOOK_DIR, m.group(1))
        if os.path.exists(fav_path):
            shell = shell.replace(m.group(1), b64_data_uri(fav_path))

    # --- Inline TOC JS (rewritten for SPA) ---
    toc_files = [f for f in os.listdir(BOOK_DIR) if f.startswith('toc-') and f.endswith('.js')]
    if not toc_files:
        raise FileNotFoundError('No toc-*.js file found in book/')
    toc_js_raw = read_file(os.path.join(BOOK_DIR, toc_files[0]))
    pages = get_pages_from_toc(toc_js_raw)
    if not pages:
        raise RuntimeError('No pages found in toc JS file')
    toc_js_rewritten = rewrite_toc_links(toc_js_raw)

    # Remove header scanning IIFE  replaced by SPA-aware version
    split_marker = '// ---------------------------------------------------------------------------\n// Support for dynamically adding headers to the sidebar.'
    if split_marker in toc_js_rewritten:
        toc_js_rewritten = toc_js_rewritten[:toc_js_rewritten.index(split_marker)]

    # Patch connectedCallback for hash-based active detection
    old_active_logic = """let current_page = document.location.href.toString().split('#')[0].split('?')[0];
        if (current_page.endsWith('/')) {
            current_page += 'index.html';
        }
        const links = Array.prototype.slice.call(this.querySelectorAll('a'));
        const l = links.length;
        for (let i = 0; i < l; ++i) {
            const link = links[i];
            const href = link.getAttribute('href');
            if (href && !href.startsWith('#') && !/^(?:[a-z+]+:)?\\/\\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The 'index' page is supposed to alias the first chapter in the book.
            if (link.href === current_page
                || i === 0
                && path_to_root === ''
                && current_page.endsWith('/index.html')) {
                link.classList.add('active');
                let parent = link.parentElement;
                while (parent) {
                    if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
                        parent.classList.add('expanded');
                    }
                    parent = parent.parentElement;
                }
            }
        }"""

    first_page_hash = '#' + page_to_id(pages[0])
    new_active_logic = """var currentHash = location.hash || '""" + first_page_hash + """';
        var links = Array.prototype.slice.call(this.querySelectorAll('a'));
        for (var i = 0; i < links.length; ++i) {
            var link = links[i];
            var href = link.getAttribute('href');
            if (href === currentHash || (i === 0 && currentHash === '""" + first_page_hash + """')) {
                link.classList.add('active');
                var parent = link.parentElement;
                while (parent) {
                    if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
                        parent.classList.add('expanded');
                    }
                    parent = parent.parentElement;
                }
            }
        }"""

    toc_js_rewritten = toc_js_rewritten.replace(old_active_logic, new_active_logic)

    # Simplify sidebar scroll restore
    old_scroll = """this.addEventListener('click', e => {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        const sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via
            // 'next/previous chapter' buttons
            const activeSection = document.querySelector('#mdbook-sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }"""

    new_scroll = """var activeSection = document.querySelector('#mdbook-sidebar .active');
        if (activeSection) {
            activeSection.scrollIntoView({ block: 'center' });
        }"""

    toc_js_rewritten = toc_js_rewritten.replace(old_scroll, new_scroll)

    shell = shell.replace(
        f'<script src="{toc_files[0]}"></script>',
        f'<script>\n{toc_js_rewritten}\n</script>'
    )

    # --- Inline remaining JS files ---
    def inline_js_tag(m):
        src = m.group(1)
        js_path = os.path.join(BOOK_DIR, src)
        if not os.path.exists(js_path):
            return m.group(0)
        js_text = read_file(js_path)
        return f'<script>\n{js_text}\n</script>'

    shell = re.sub(r'<script src="([^"]+)"></script>', inline_js_tag, shell)

    # --- Remove noscript iframe (toc.html not needed) ---
    shell = re.sub(r'<noscript>\s*<iframe[^>]*src="toc\.html"[^>]*></iframe>\s*</noscript>', '', shell)

    # --- Build page sections ---
    page_titles = get_page_titles_from_toc(toc_js_raw)
    sections_html = []
    for i, page in enumerate(pages):
        page_path = os.path.join(BOOK_DIR, page)
        if not os.path.exists(page_path):
            print(f'Warning: {page_path} not found, skipping')
            continue

        page_html = read_file(page_path)
        main_content = extract_main(page_html)
        main_content = rewrite_content_links(main_content, page)
        main_content = inline_images(main_content, page)

        # Build prev/next nav links
        nav_parts = []
        if i > 0:
            prev_id = page_to_id(pages[i - 1])
            prev_title = page_titles.get(pages[i - 1], prev_id)
            nav_parts.append(f'<a href="#{prev_id}" class="page-nav-prev">Previous: {prev_title}</a>')
        if i < len(pages) - 1:
            next_id = page_to_id(pages[i + 1])
            next_title = page_titles.get(pages[i + 1], next_id)
            nav_parts.append(f'<a href="#{next_id}" class="page-nav-next">Next: {next_title}</a>')
        nav_html = f'<div class="page-nav">{"".join(nav_parts)}</div>' if nav_parts else ''

        section_id = page_to_id(page)
        display = 'block' if i == 0 else 'none'
        sections_html.append(
            f'<section id="{section_id}" class="spa-page" style="display:{display}">'
            f'{main_content}'
            f'{nav_html}'
            f'</section>'
        )

    combined_content = '\n'.join(sections_html)

    # Strip nav wrappers and replace main content
    shell = strip_nav(shell)
    shell = re.sub(
        r'<main>.*?</main>',
        f'<main>\n{combined_content}\n</main>',
        shell,
        flags=re.DOTALL
    )

    # Inject page-nav CSS and SPA scripts before </body>
    page_nav_css = '''<style>
.page-nav { display: flex; justify-content: space-between; margin-top: 2rem; padding-top: 1rem; border-top: 1px solid #404040; }
.page-nav a { color: #39AFD7; text-decoration: none; }
.page-nav a:hover { text-decoration: underline; }
.page-nav-next { margin-left: auto; }
</style>'''
    page_list_json = str([page_to_id(p) for p in pages])
    spa_script = SPA_SCRIPT.replace('PAGE_LIST_PLACEHOLDER', page_list_json)
    shell = shell.replace('</body>', f'{page_nav_css}\n{spa_script}\n</body>')

    # Update title  strip page-specific prefix (e.g. "Introduction - Vastrum Protocol" → "Vastrum Protocol")
    shell = re.sub(r'<title>.+? - (.+)</title>', r'<title>\1</title>', shell)

    # --- Post-processing ---
    shell = strip_comments(shell)

    # Write output
    os.makedirs(os.path.dirname(OUTPUT), exist_ok=True)
    with open(OUTPUT, 'w') as f:
        f.write(shell)

    print(f'SPA written to {OUTPUT}')
    print(f'Pages combined: {len(sections_html)}')


if __name__ == '__main__':
    build_spa()
