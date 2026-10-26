(() => {
  const T = (window.TMT ??= {} as any);
  const syncHeadingHash = (...a) => T.syncHeadingHash(...a);

  // ==================== CLASSIC VIEW INTERACTIONS ====================
  function setupClassicViewInteraction() {
    const classicView = document.getElementById('wiki-classic-view');
    if (!classicView) return;

    const classicTocLinks = Array.from(classicView.querySelectorAll('.classic-toc-link, .toc a'));
    const linkById = new Map();
    classicTocLinks.forEach((link: HTMLElement) => {
      const href = link.getAttribute('href');
      if (href && href.startsWith('#')) {
        linkById.set(href.slice(1), link);
      }
    });

    const headingEls = Array.from(
      classicView.querySelectorAll('article h1[id], article h2[id], article h3[id], article h4[id]')
    );

    if (headingEls.length === 0) {
      if ((window as any).__tmtClassicSpy) {
        (window as any).__tmtClassicSpy.destroy();
        (window as any).__tmtClassicSpy = null;
      }
      return;
    }

    const headingTargets = headingEls.map((el: HTMLElement) => ({
      id: el.id,
      el,
      link: linkById.get(el.id) || null,
    }));

    const classicSpy = T.createScrollSpy({
      container: window,
      headingTargets,
      tocLinks: classicTocLinks,
      headingLine: 80,
      isActiveView: () => document.documentElement.getAttribute('data-view') === 'classic',
      onActiveChange: (currentId) => {
        syncHeadingHash(currentId);
      },
    });

    if ((window as any).__tmtClassicSpy) {
      (window as any).__tmtClassicSpy.destroy();
    }
    (window as any).__tmtClassicSpy = classicSpy;

    classicTocLinks.forEach((link: HTMLElement) => {
      link.addEventListener('click', (e) => {
        const href = link.getAttribute('href');
        if (href && href.startsWith('#')) {
          e.preventDefault();
          const targetId = href.slice(1);
          const targetEl = document.getElementById(targetId);
          if (targetEl) {
            classicSpy.markClickScrolling(800);
            targetEl.scrollIntoView({ behavior: 'smooth', block: 'start' });
            syncHeadingHash(targetId);
          }
        }
      });
    });
  }

  Object.assign(T, {
    setupClassicViewInteraction,
  });
})();
