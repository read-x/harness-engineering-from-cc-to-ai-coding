(() => {
    const rightButtons = document.querySelector('.right-buttons');
    if (!rightButtons) return;

    const segments = window.location.pathname.split('/').filter(Boolean);
    const isEnglish = segments[0] === 'en' || segments[1] === 'en';
    const trailingSlash = window.location.pathname.endsWith('/');
    const knownContentRoots = new Set([
        'appendix',
        'part1',
        'part2',
        'part3',
        'part4',
        'part5',
        'part6',
        'part7',
        'index.html',
        'preface.html',
        '404.html',
        'print.html',
        'toc.html',
    ]);

    let prefix = [];
    let rest = [];

    if (segments[0] === 'en') {
        rest = segments.slice(1);
    } else if (segments[1] === 'en') {
        prefix = [segments[0]];
        rest = segments.slice(2);
    } else {
        const first = segments[0];
        const looksLikeContentRoot = !first
            || knownContentRoots.has(first)
            || /^part\d+$/.test(first)
            || first.endsWith('.html');

        prefix = looksLikeContentRoot ? [] : [first];
        rest = looksLikeContentRoot ? segments : segments.slice(1);
    }

    const targetSegments = isEnglish
        ? prefix.concat(rest)
        : prefix.concat(['en'], rest);
    const targetPath = `/${targetSegments.join('/')}${trailingSlash ? '/' : ''}`;
    const targetUrl = `${targetPath}${window.location.search}${window.location.hash}`;

    const link = document.createElement('a');
    link.className = 'icon-button language-switcher-button';
    link.href = targetUrl;
    link.title = isEnglish ? 'Switch to Chinese' : 'Switch to English';
    link.setAttribute('aria-label', isEnglish ? 'Switch to Chinese' : 'Switch to English');

    const label = document.createElement('span');
    label.className = 'language-switcher-label';
    label.textContent = isEnglish ? '中文' : 'EN';
    link.appendChild(label);

    rightButtons.insertBefore(link, rightButtons.firstChild);
})();
