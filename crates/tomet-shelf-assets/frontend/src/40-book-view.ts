(() => {
  const T = (window.TMT ??= {} as any);
  const t = (key: string, fallback = "") => T.t(key, fallback);

  // ==================== BOOK VIEW INTERACTIONS ====================

  /** Heading indicated by the requested URL that exists on this page. */
  function requestedHeadingId() {
    const id = T.readHash();
    return id && document.getElementById(id) ? id : null;
  }

  function setupBookViewInteraction() {
    const tocLinks = document.querySelectorAll('.book-toc-link');
    const scrollContainer = document.getElementById('book-content-scroll');

    // Mobile navigation drawer toggle
    const toggleNavBtn = document.getElementById('btn-toggle-nav');
    const paneNav = document.getElementById('pane-nav');
    const navBackdrop = document.getElementById('pane-nav-backdrop');

    if (toggleNavBtn && paneNav) {
      toggleNavBtn.onclick = () => {
        paneNav.classList.toggle('is-open');
        navBackdrop?.classList.toggle('is-open');
      };
      if (navBackdrop) {
        navBackdrop.onclick = () => {
          paneNav.classList.remove('is-open');
          navBackdrop.classList.remove('is-open');
        };
      }
    }

    if (scrollContainer && tocLinks.length > 0) {
      let headingTargets: any[] = Array.from(tocLinks)
        .map((link) => {
          const id = link.getAttribute('data-target');
          const el = id ? document.getElementById(id) : null;
          return el ? { id, el, link } : null;
        })
        .filter(Boolean);

      if (headingTargets.length === 0) {
        const headingEls = scrollContainer.querySelectorAll('h1[id], h2[id], h3[id], h4[id]');
        headingTargets = Array.from(headingEls).map((el) => ({
          id: el.id,
          el,
          link: null,
        }));
      }

      // ScrollSpy: track headings as reader scrolls
      const spy = T.createScrollSpy({
        container: scrollContainer,
        headingTargets,
        tocLinks,
        headingLine: 80,
        isActiveView: () => true,
        onActiveChange: (currentId) => {
          T.syncHeadingHash(currentId);
        },
      });

      if ((window as any).__tmtBookSpy) {
        (window as any).__tmtBookSpy.destroy();
      }
      (window as any).__tmtBookSpy = spy;

      tocLinks.forEach((link) => {
        link.addEventListener('click', (e) => {
          e.preventDefault();
          const targetId = link.getAttribute('data-target');
          if (!targetId) return;
          const targetEl = document.getElementById(targetId);
          if (targetEl && scrollContainer) {
            spy.markClickScrolling(800);
            const top = targetEl.offsetTop - scrollContainer.offsetTop - 16;
            scrollContainer.scrollTo({ top, behavior: 'smooth' });
            tocLinks.forEach((l) => l.classList.remove('is-active'));
            link.classList.add('is-active');
            T.syncHeadingHash(targetId);
          }
        });
      });

      // Jump to requested hash section on load
      const requested = requestedHeadingId();
      const settle = () => {
        if (requested) {
          const target = document.getElementById(requested);
          if (target) {
            scrollContainer.scrollTo({
              top: target.offsetTop - scrollContainer.offsetTop - 16,
              behavior: 'auto',
            });
          }
        }
        spy.update();
      };
      settle();
      setTimeout(settle, 80);
    }
  }

  Object.assign(T, {
    setupBookViewInteraction,
    requestedHeadingId,
  });
})();
