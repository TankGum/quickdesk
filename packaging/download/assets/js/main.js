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

  /* ---------- Language ---------- */
  // English lives in the markup; Vietnamese for every [data-i18n] element is
  // in #i18n-vi (inside index.html, so release placeholders get filled in).
  const VI = JSON.parse($('#i18n-vi')?.textContent || '{}');
  const EN_META = { title: document.title, description: $('meta[name="description"]').content };
  // Strings this script writes itself.
  const STR = {
    en: {
      refreshed: 'Usage refreshed', paused: 'Clipboard history paused', resumed: 'Clipboard history resumed',
      quit: 'Quit — in the real app, QuickDesk would close now',
      encrypting: 'Encrypting…', synced: 'Encrypted & synced',
      copied: 'Copied', pressCopy: 'Press Ctrl+C', copy: 'Copy',
      note: 'Standup — Tuesday\n• Shipped clipboard images\n• Reviewing the release flow\n• No blockers',
    },
    vi: {
      refreshed: 'Đã làm mới usage', paused: 'Đã tạm dừng lưu clipboard', resumed: 'Đã bật lại lưu clipboard',
      quit: 'Thoát — trong app thật, QuickDesk sẽ đóng lại ngay',
      encrypting: 'Đang mã hoá…', synced: 'Đã mã hoá & đồng bộ',
      copied: 'Đã copy', pressCopy: 'Bấm Ctrl+C', copy: 'Copy',
      note: 'Họp đầu ngày — Thứ Ba\n• Đã xong ảnh trong clipboard\n• Đang rà quy trình phát hành\n• Không vướng gì',
    },
  };
  let lang = 'en';
  const tr = (key) => STR[lang][key];
  const langListeners = [];

  function setLang(next, remember) {
    lang = next === 'vi' ? 'vi' : 'en';
    document.documentElement.lang = lang;
    $$('[data-i18n]').forEach((el) => {
      if (el.dataset.en === undefined) el.dataset.en = el.innerHTML;
      const html = lang === 'vi' ? VI[el.dataset.i18n] : el.dataset.en;
      if (html !== undefined) el.innerHTML = html;
    });
    document.title = lang === 'vi' ? VI.title : EN_META.title;
    $('meta[name="description"]').content = lang === 'vi' ? VI.description : EN_META.description;
    $$('[data-lang]').forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.lang === lang)));
    // The footer's year span is re-created with its sentence.
    $('#year').textContent = new Date().getFullYear();
    // The demo windows are the real app: switch its language too.
    $$('iframe').forEach((f) => f.contentWindow?.postMessage({ source: 'quickdesk-page', type: 'lang', lang }, '*'));
    langListeners.forEach((fn) => fn());
    if (remember) {
      try { localStorage.setItem('qd-lang', lang); } catch { /* private mode */ }
    }
  }

  let saved = null;
  try { saved = localStorage.getItem('qd-lang'); } catch { /* private mode */ }
  setLang(saved || (navigator.language.toLowerCase().startsWith('vi') ? 'vi' : 'en'), false);
  $$('[data-lang]').forEach((b) => b.addEventListener('click', () => setLang(b.dataset.lang, true)));
  // Iframes that finish loading later still need to hear the language.
  $$('iframe').forEach((f) => f.addEventListener('load', () => {
    f.contentWindow?.postMessage({ source: 'quickdesk-page', type: 'lang', lang }, '*');
  }));

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
  const menus = Object.fromEntries($$('[data-menu]', demo).map((m) => [m.dataset.menu, m]));
  const trayButtons = { ai: $('[data-ring]', desktop), app: $('[data-app-tray]', desktop) };
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

  // Drop a tray menu down under its icon, like GNOME Shell.
  function openMenu(name) {
    Object.entries(menus).forEach(([n, m]) => m.classList.toggle('is-open', n === name));
    Object.entries(trayButtons).forEach(([n, b]) => b.classList.toggle('is-active', n === name));
    const menu = menus[name];
    if (!menu) return;
    const icon = trayButtons[name].getBoundingClientRect();
    const box = stage.getBoundingClientRect();
    const x = icon.left + icon.width / 2 - box.left - menu.offsetWidth / 2;
    menu.style.left = `${Math.max(8, Math.min(x, box.width - menu.offsetWidth - 8))}px`;
  }
  const menuOpen = () => Object.values(menus).some((m) => m.classList.contains('is-open'));

  function showWindow(win, tab) {
    Object.entries(wins).forEach(([name, w]) => w.classList.toggle('is-open', name === win));
    openMenu(null);
    desktop.classList.add('has-panel');
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
      openMenu('ai');
    } else {
      showWindow(m.win, m.tab);
    }
    pressCombo(mode);
  }

  function closePanel() {
    current = null;
    Object.values(wins).forEach((w) => w.classList.remove('is-open'));
    openMenu(null);
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
  trayButtons.ai.addEventListener('click', () => {
    takeOver();
    if (menus.ai.classList.contains('is-open')) closePanel();
    else openPanel('ai');
  });

  // Like the real tray: a usage line opens the AI tab; Refresh refreshes.
  $('[data-tray]', demo).addEventListener('click', (e) => {
    const li = e.target.closest('li');
    if (!li || !('limit' in li.dataset || 'refresh' in li.dataset)) return;
    takeOver();
    if ('refresh' in li.dataset) {
      openMenu(null);
      if (!Object.values(wins).some((w) => w.classList.contains('is-open'))) closePanel();
      showToast(tr('refreshed'));
      return;
    }
    showWindow('main', 'ai');
  });

  // QuickDesk's own tray icon and menu.
  trayButtons.app.addEventListener('click', () => {
    takeOver();
    if (menus.app.classList.contains('is-open')) {
      openMenu(null);
      if (!Object.values(wins).some((w) => w.classList.contains('is-open'))) closePanel();
    } else {
      desktop.classList.add('has-panel');
      openMenu('app');
    }
  });
  let paused = false;
  menus.app.addEventListener('click', (e) => {
    const li = e.target.closest('li');
    if (!li || li.classList.contains('sep')) return;
    takeOver();
    const open = li.dataset.open;
    if (open && MODES[open]) openPanel(open);
    else if (open === 'note-popup') { tabs.forEach((t) => t.setAttribute('aria-selected', 'false')); showWindow('note-popup'); }
    else if (open === 'main') showWindow('main', null);
    else if ('pause' in li.dataset) {
      paused = !paused;
      li.classList.toggle('is-on', !paused);
      Object.keys(wins).forEach((w) => post(w, { type: 'pause', paused }));
      openMenu(null);
      if (!Object.values(wins).some((w) => w.classList.contains('is-open'))) closePanel();
      showToast(tr(paused ? 'paused' : 'resumed'));
    } else if ('quit' in li.dataset) {
      closePanel();
      showToast(tr('quit'));
    }
  });

  $$('[data-close]', demo).forEach((b) => b.addEventListener('click', () => {
    takeOver();
    closePanel();
  }));

  // Click on the wallpaper (outside any window) closes, like a real overlay.
  desktop.addEventListener('click', (e) => {
    if ((current || menuOpen()) && !e.target.closest('.shell-menu, .appwin, .desktop__tray')) {
      takeOver();
      closePanel();
    }
  });

  // Messages from the app windows: a popup hid itself, or wants a toast.
  window.addEventListener('message', ({ data }) => {
    if (!data || data.source !== 'quickdesk-demo') return;
    if (data.type === 'hide') {
      wins[data.label]?.classList.remove('is-open');
      if (!Object.values(wins).some((w) => w.classList.contains('is-open')) && !menuOpen()) closePanel();
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

    if (e.key === 'Escape' && (current || menuOpen())) {
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
  const clockFormat = () => new Intl.DateTimeFormat(lang === 'vi' ? 'vi-VN' : 'en-US', {
    weekday: 'short', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', hour12: false,
  });
  let fmt = clockFormat();
  const tick = () => { clock.textContent = fmt.format(new Date()).replace(/,/g, ''); };
  tick();
  langListeners.push(() => {
    fmt = clockFormat();
    tick();
    $('.desktop__ime').textContent = lang;
  });
  $('.desktop__ime').textContent = lang;
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
    const NOTE = () => tr('note');

    if (reduceMotion) {
      const show = () => { src.textContent = NOTE(); dst.textContent = NOTE(); badge.textContent = tr('synced'); };
      show();
      langListeners.push(show);
    } else {
      let started = false;
      const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
      let mirrorTimer = null;

      const mirror = () => {
        clearTimeout(mirrorTimer);
        sync.classList.add('is-syncing');
        badge.textContent = tr('encrypting');
        mirrorTimer = setTimeout(() => {
          dst.textContent = src.textContent;
          sync.classList.remove('is-syncing');
          badge.textContent = tr('synced');
        }, 450);
      };

      const loop = async () => {
        for (;;) {
          src.textContent = '';
          dst.textContent = '';
          src.classList.add('is-typing');
          await sleep(600);
          for (const ch of NOTE()) {
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
        btn.textContent = tr('copied');
      } catch {
        btn.textContent = tr('pressCopy');
      }
      setTimeout(() => { btn.textContent = tr('copy'); }, 1800);
    });
  });
})();
