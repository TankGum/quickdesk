(() => {
  'use strict';

  document.documentElement.classList.add('js');
  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const $ = (sel, root = document) => root.querySelector(sel);
  const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));

  /* ---------- Nav border on scroll ---------- */
  const nav = $('#nav');
  const onScroll = () => nav.classList.toggle('is-scrolled', window.scrollY > 8);
  onScroll();
  window.addEventListener('scroll', onScroll, { passive: true });

  /* ---------- Footer year ---------- */
  $('#year').textContent = new Date().getFullYear();

  /* ---------- Reveal on scroll (staggered per parent) ---------- */
  const revealIO = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (!entry.isIntersecting) return;
      const el = entry.target;
      const siblings = $$(':scope > .reveal', el.parentElement);
      el.style.transitionDelay = `${Math.min(siblings.indexOf(el), 4) * 80}ms`;
      el.classList.add('is-in');
      revealIO.unobserve(el);
    });
  }, { threshold: 0.12, rootMargin: '0px 0px -40px 0px' });
  $$('.reveal').forEach((el) => revealIO.observe(el));

  /* ---------- Interactive demo ---------- */
  // The windows are the real QuickDesk UI (demo/ = `npm run build:demo`),
  // running on sample data in iframes. This script only plays the desktop:
  // which window is up, the hotkey animation, the tray menu.
  const MODES = {
    notes:     { letter: 'N', code: 'KeyN', color: 'var(--notes)', win: 'main', tab: 'notes' },
    clipboard: { letter: 'V', code: 'KeyV', color: 'var(--clip)', win: 'clip-popup' },
    ports:     { letter: 'P', code: 'KeyP', color: 'var(--ports)', win: 'main', tab: 'ports' },
    // AI usage has no shortcut: it lives behind the ring in the top bar.
    ai:        { letter: 'A', code: 'KeyA', color: 'var(--ai)', tray: true },
  };
  const ORDER = Object.keys(MODES);
  const CODE_TO_MODE = Object.fromEntries(ORDER.map((m) => [MODES[m].code, m]));

  const demo = $('#demo');
  const desktop = $('#desktop');
  const stage = $('.desktop__stage', desktop);
  const letter = $('#comboLetter');
  const comboKeys = $$('.combo .key', demo);
  const tabs = $$('[data-mode]', demo);
  const wins = Object.fromEntries($$('[data-appwin]', demo).map((w) => [w.dataset.appwin, w]));
  const trayMenu = $('[data-panel="ai"]', demo);
  const toast = $('#toast');

  let current = null;
  let pressTimers = [];
  let autoTimer = null;
  let userTookOver = false;

  // Scale each window to fit the mock screen (and the inline shots to their column).
  function fit() {
    const pad = 0.94;
    Object.values(wins).forEach((w) => {
      const s = Math.min(1, (stage.clientWidth * pad) / parseFloat(w.style.getPropertyValue('--w')),
        (stage.clientHeight * pad) / parseFloat(w.style.getPropertyValue('--h')));
      w.style.setProperty('--s', s.toFixed(4));
    });
    $$('[data-shot]').forEach((shot) => {
      shot.firstElementChild.style.setProperty('--s', (shot.clientWidth / parseFloat(shot.style.getPropertyValue('--w'))).toFixed(4));
    });
  }
  fit();
  window.addEventListener('resize', fit);

  const post = (win, msg) => {
    wins[win]?.querySelector('iframe').contentWindow?.postMessage({ source: 'quickdesk-page', ...msg }, '*');
  };

  function pressCombo(mode) {
    pressTimers.forEach(clearTimeout);
    pressTimers = [];
    comboKeys.forEach((k) => k.classList.remove('is-down'));
    demo.classList.toggle('is-tray', Boolean(MODES[mode].tray));
    if (MODES[mode].tray) return;
    letter.textContent = MODES[mode].letter;
    comboKeys.forEach((k, i) => pressTimers.push(setTimeout(() => k.classList.add('is-down'), i * 90)));
    pressTimers.push(setTimeout(() => comboKeys.forEach((k) => k.classList.remove('is-down')), 520));
  }

  function showWindow(win, tab) {
    Object.entries(wins).forEach(([name, w]) => w.classList.toggle('is-open', name === win));
    trayMenu.classList.remove('is-open');
    // What Rust sends when a hotkey fires: the window switches to that tab.
    post(win, { type: 'show', tab: tab ?? null });
  }

  function openPanel(mode) {
    current = mode;
    const m = MODES[mode];
    demo.style.setProperty('--mode', m.color);
    tabs.forEach((t) => t.setAttribute('aria-selected', String(t.dataset.mode === mode)));
    desktop.classList.add('has-panel');
    if (m.tray) {
      Object.values(wins).forEach((w) => w.classList.remove('is-open'));
      trayMenu.classList.add('is-open');
    } else {
      showWindow(m.win, m.tab);
    }
    pressCombo(mode);
  }

  function closePanel() {
    current = null;
    Object.values(wins).forEach((w) => w.classList.remove('is-open'));
    trayMenu.classList.remove('is-open');
    tabs.forEach((t) => t.setAttribute('aria-selected', 'false'));
    desktop.classList.remove('has-panel');
    demo.classList.remove('is-tray');
  }

  function takeOver() {
    userTookOver = true;
    clearInterval(autoTimer);
  }

  tabs.forEach((tab) => tab.addEventListener('click', () => {
    takeOver();
    openPanel(tab.dataset.mode);
  }));

  // The ring in the top bar toggles the AI usage menu, like the real tray icon.
  $('[data-ring]', desktop).addEventListener('click', () => {
    takeOver();
    if (current === 'ai' && trayMenu.classList.contains('is-open')) closePanel();
    else openPanel('ai');
  });

  // Like the real tray: clicking a usage line opens the AI tab.
  $('[data-tray]', demo).addEventListener('click', () => {
    takeOver();
    showWindow('main', 'ai');
  });

  $$('[data-close]', demo).forEach((b) => b.addEventListener('click', () => {
    takeOver();
    closePanel();
  }));

  // Click on the wallpaper (outside any window) closes, like a real overlay.
  desktop.addEventListener('click', (e) => {
    if (current && !e.target.closest('.panel, .appwin, [data-ring]')) {
      takeOver();
      closePanel();
    }
  });

  // Messages from the app windows: a popup hid itself, or wants a toast.
  window.addEventListener('message', ({ data }) => {
    if (!data || data.source !== 'quickdesk-demo') return;
    if (data.type === 'hide') {
      wins[data.label]?.classList.remove('is-open');
      if (!Object.values(wins).some((w) => w.classList.contains('is-open')) && !trayMenu.classList.contains('is-open')) closePanel();
    }
    if (data.type === 'toast') showToast(data.text);
    if (data.type === 'interact') takeOver();
  });

  function isInView(el) {
    const r = el.getBoundingClientRect();
    return r.top < window.innerHeight * 0.85 && r.bottom > window.innerHeight * 0.15;
  }

  document.addEventListener('keydown', (e) => {
    if (e.target.closest('input, textarea, select, [contenteditable="true"]')) return;

    if (e.key === 'Escape' && current) {
      takeOver();
      closePanel();
      return;
    }

    const mode = CODE_TO_MODE[e.code];
    // Plain letters (Shift/Alt allowed) so no browser shortcut gets in the way.
    if (!mode || e.ctrlKey || e.metaKey) return;
    e.preventDefault();
    takeOver();

    // Same shortcut again toggles the window, just like the app.
    if (current === mode) closePanel();
    else openPanel(mode);

    if (!isInView(desktop)) {
      desktop.scrollIntoView({ behavior: reduceMotion ? 'auto' : 'smooth', block: 'center' });
    }
  });

  // Auto-cycle through the windows until the visitor interacts.
  const demoIO = new IntersectionObserver(([entry]) => {
    clearInterval(autoTimer);
    if (!entry.isIntersecting || userTookOver) return;
    if (!current) openPanel(ORDER[0]);
    if (reduceMotion) return;
    autoTimer = setInterval(() => {
      const next = ORDER[(ORDER.indexOf(current) + 1) % ORDER.length];
      openPanel(next);
    }, 4200);
  }, { threshold: 0.4 });
  demoIO.observe(desktop);

  // Live clock in the top bar.
  const clock = $('#clock');
  const fmt = new Intl.DateTimeFormat('en-US', {
    weekday: 'short', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', hour12: false,
  });
  const tick = () => { clock.textContent = fmt.format(new Date()).replace(/,/g, ''); };
  tick();
  setInterval(tick, 15000);

  let toastTimer = null;
  function showToast(msg) {
    toast.textContent = msg;
    toast.classList.add('is-on');
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => toast.classList.remove('is-on'), 2600);
  }

  /* ---------- Notes: live sync typing ---------- */
  const sync = $('[data-sync]');
  if (sync) {
    const src = $('[data-sync-src]', sync);
    const dst = $('[data-sync-dst]', sync);
    const badge = $('[data-sync-badge]', sync);
    const NOTE = 'Standup — Tuesday\n• Shipped clipboard images\n• Reviewing the release flow\n• No blockers';

    if (reduceMotion) {
      src.textContent = NOTE;
      dst.textContent = NOTE;
    } else {
      let started = false;
      const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
      let mirrorTimer = null;

      const mirror = () => {
        clearTimeout(mirrorTimer);
        sync.classList.add('is-syncing');
        badge.textContent = 'Encrypting…';
        mirrorTimer = setTimeout(() => {
          dst.textContent = src.textContent;
          sync.classList.remove('is-syncing');
          badge.textContent = 'Encrypted & synced';
        }, 450);
      };

      const loop = async () => {
        for (;;) {
          src.textContent = '';
          dst.textContent = '';
          src.classList.add('is-typing');
          await sleep(600);
          for (const ch of NOTE) {
            src.textContent += ch;
            if (ch === '\n' || ch === ' ') mirror();
            await sleep(ch === '\n' ? 260 : 38 + Math.random() * 50);
          }
          mirror();
          await sleep(900);
          src.classList.remove('is-typing');
          await sleep(4200);
        }
      };

      new IntersectionObserver(([entry], io) => {
        if (entry.isIntersecting && !started) {
          started = true;
          io.disconnect();
          loop();
        }
      }, { threshold: 0.4 }).observe(sync);
    }
  }

  /* ---------- Ports: EADDRINUSE story ---------- */
  const story = $('[data-story]');
  if (story) {
    const term = $('[data-story-term]', story);
    const row = $('[data-story-row]', story);
    const killBtn = $('[data-story-kill]', story);
    const replay = $('[data-story-replay]', story);

    const BEFORE =
      '<span class="p">$</span> npm run dev\n' +
      '<span class="dim">&gt; web@1.0.0 dev\n&gt; vite --port 3000 --strictPort</span>\n\n' +
      '<span class="err">error when starting dev server:\nError: Port 3000 is already in use</span>\n' +
      '<span class="p">$</span> <span class="dim">█</span>';
    const AFTER =
      '<span class="p">$</span> npm run dev\n' +
      '<span class="dim">&gt; web@1.0.0 dev\n&gt; vite --port 3000 --strictPort</span>\n\n' +
      '<span class="line-in">  <span class="ok">VITE v6.0.0</span>  ready in 312 ms\n\n' +
      '  <span class="ok">➜</span>  Local:   http://localhost:3000/\n' +
      '  <span class="dim">➜  press h + enter to show help</span></span>';

    const reset = () => {
      term.innerHTML = BEFORE;
      row.classList.remove('is-killing');
      row.hidden = false;
      story.classList.remove('is-fixed');
    };
    reset();

    killBtn.addEventListener('click', () => {
      row.classList.add('is-killing');
      setTimeout(() => {
        row.hidden = true;
        term.innerHTML = AFTER;
        story.classList.add('is-fixed');
      }, 350);
    });
    replay.addEventListener('click', reset);
  }

  /* ---------- Copy install command ---------- */
  $$('[data-copy]').forEach((btn) => {
    btn.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(btn.dataset.copy);
        btn.textContent = 'Copied';
      } catch {
        btn.textContent = 'Press Ctrl+C';
      }
      setTimeout(() => { btn.textContent = 'Copy'; }, 1800);
    });
  });
})();
