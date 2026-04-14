// =========================================================
// Pomo - Frontend
// =========================================================

// Show error on screen (for debugging without devtools)
function showError(msg) {
  let el = document.getElementById('debug-log');
  if (!el) {
    el = document.createElement('div');
    el.id = 'debug-log';
    el.style.cssText = 'position:fixed;bottom:0;left:0;right:0;background:red;color:white;font-size:11px;padding:4px 8px;z-index:9999;max-height:120px;overflow:auto;';
    document.body.appendChild(el);
  }
  el.textContent += msg + '\n';
}

// Wait for Tauri API to become available
function waitForTauri(callback) {
  let attempts = 0;
  const check = () => {
    attempts++;
    if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
      callback();
    } else if (attempts > 50) {
      showError('ERROR: window.__TAURI__ not found after 5s. withGlobalTauri may not be set.');
    } else {
      setTimeout(check, 100);
    }
  };
  check();
}

document.addEventListener('DOMContentLoaded', () => {
  waitForTauri(initApp);
});

function initApp() {
  const invoke = window.__TAURI__.core.invoke;
  const listen = window.__TAURI__.event.listen;

  // --- State ---
  let currentBgm = 'off';
  let currentPhase = 'work';
  let timerInterval = null;

  const PHASE_COLORS = {
    work:        { color: '#FF6B6B', track: '#FFE0E0' },
    short_break: { color: '#51D88A', track: '#D4F5E0' },
    long_break:  { color: '#9B8EC4', track: '#E8E0F0' },
  };

  const phaseLabel     = document.getElementById('phase-label');
  const timeDisplay    = document.getElementById('time-display');
  const sessionDisplay = document.getElementById('session-display');
  const startBtn       = document.getElementById('start-btn');
  const ringProgress   = document.getElementById('ring-progress');
  const cycleDots      = document.getElementById('cycle-dots');
  const audio1         = document.getElementById('audio-1');
  const audio2         = document.getElementById('audio-2');

  const CIRCUMFERENCE = 2 * Math.PI * 84;

  // --- Polling ---
  function startPolling() {
    if (timerInterval) return;
    timerInterval = setInterval(async () => {
      try {
        const s = await invoke('tick');
        updateUI(s);
        if (s.phase_finished) onPhaseFinished(s);
      } catch (e) { showError('tick: ' + e); }
    }, 200);
  }

  function stopPolling() {
    if (timerInterval) { clearInterval(timerInterval); timerInterval = null; }
  }

  // --- UI Update ---
  function updateUI(s) {
    const c = PHASE_COLORS[s.phase] || PHASE_COLORS.work;
    document.documentElement.style.setProperty('--phase-color', c.color);
    document.documentElement.style.setProperty('--phase-track', c.track);

    const labels = { work: 'WORK', short_break: 'SHORT BREAK', long_break: 'LONG BREAK' };
    phaseLabel.textContent = labels[s.phase] || 'WORK';

    const m = Math.floor(s.remaining_secs / 60);
    const sec = s.remaining_secs % 60;
    timeDisplay.textContent = String(m).padStart(2, '0') + ':' + String(sec).padStart(2, '0');

    ringProgress.style.strokeDashoffset = CIRCUMFERENCE * (1 - s.progress);
    sessionDisplay.textContent = 'Session ' + s.cycle + '/' + s.num_sets;

    startBtn.textContent = s.is_running ? 'Pause' : (s.state === 'finished' ? 'Next' : 'Start');

    renderDots(s.cycle, s.num_sets);

    if (s.phase !== currentPhase) {
      currentPhase = s.phase;
      if (currentBgm !== 'off') playBgm(currentBgm, currentPhase);
      updateBgmButtons();
    }
  }

  function renderDots(cycle, n) {
    if (cycleDots.children.length !== n) {
      cycleDots.innerHTML = '';
      for (let i = 1; i <= n; i++) {
        const d = document.createElement('div');
        d.className = 'dot' + (i <= cycle ? ' filled' : '');
        cycleDots.appendChild(d);
      }
    } else {
      for (let i = 0; i < n; i++)
        cycleDots.children[i].classList.toggle('filled', i < cycle);
    }
  }

  async function onPhaseFinished(s) {
    try { const ns = await invoke('skip'); updateUI(ns); } catch (e) { showError('skip: ' + e); }
    try {
      const msgs = { work: ['Work Complete!', 'Time for a break.'], short_break: ['Break Over!', 'Ready to focus?'], long_break: ['Long Break Over!', 'Starting a new cycle.'] };
      const [t, b] = msgs[s.phase] || ['Done', ''];
      new Notification(t, { body: b });
    } catch (e) {}
  }

  // --- Controls ---
  window.toggleStartPause = async function () {
    try {
      const s = await invoke('tick');
      let ns;
      if (s.is_running) { ns = await invoke('pause'); stopPolling(); }
      else { ns = await invoke('start'); startPolling(); }
      updateUI(ns);
      if (ns.is_running && currentBgm !== 'off') playBgm(currentBgm, currentPhase);
    } catch (e) { showError('startPause: ' + e); }
  };

  window.doReset = async function () {
    try { stopPolling(); updateUI(await invoke('reset')); stopAllAudio(); }
    catch (e) { showError('reset: ' + e); }
  };

  window.doSkip = async function () {
    try {
      const s = await invoke('skip');
      updateUI(s);
      if (currentBgm !== 'off') playBgm(currentBgm, s.phase);
    } catch (e) { showError('skip: ' + e); }
  };

  // --- BGM ---
  window.setBgm = function (mode) {
    currentBgm = mode;
    updateBgmButtons();
    if (mode === 'off') { stopAllAudio(); return; }
    playBgm(mode, currentPhase);
  };

  function updateBgmButtons() {
    document.querySelectorAll('.bgm-btn').forEach(b =>
      b.classList.toggle('active', b.dataset.mode === currentBgm));
  }

  function playBgm(mode, phase) {
    stopAllAudio();
    const isWork = phase === 'work';
    if (mode === 'nature') {
      audio1.src = isWork ? 'assets/bgm/nature_work.mp3' : 'assets/bgm/nature_break.mp3';
      audio1.volume = 0.5; audio1.play().catch(() => {});
    } else if (mode === 'cafe') {
      if (isWork) {
        audio1.src = 'assets/bgm/cafe_work_1.mp3';
        audio2.src = 'assets/bgm/cafe_work_2.mp3';
        audio1.volume = 0.5; audio2.volume = 0.5;
        audio1.play().catch(() => {}); audio2.play().catch(() => {});
      } else {
        audio1.src = 'assets/bgm/cafe_break.mp3';
        audio1.volume = 0.5; audio1.play().catch(() => {});
      }
    }
  }

  function stopAllAudio() {
    audio1.pause(); audio1.currentTime = 0; audio1.removeAttribute('src');
    audio2.pause(); audio2.currentTime = 0; audio2.removeAttribute('src');
  }

  // --- Settings ---
  window.toggleSettings = async function () {
    const sv = document.getElementById('settings-view');
    const tv = document.getElementById('timer-view');
    if (sv.style.display === 'none') {
      sv.style.display = 'block'; tv.style.display = 'none';
      try {
        const s = await invoke('get_settings');
        document.getElementById('s-work').value = s.work_minutes;
        document.getElementById('s-work-val').textContent = s.work_minutes;
        document.getElementById('s-break').value = s.short_break_minutes;
        document.getElementById('s-break-val').textContent = s.short_break_minutes;
        document.getElementById('s-long').value = s.long_break_minutes;
        document.getElementById('s-long-val').textContent = s.long_break_minutes;
        document.getElementById('s-sets').value = s.num_sets;
        document.getElementById('s-sets-val').textContent = s.num_sets;
      } catch (e) { showError('settings: ' + e); }
    } else { sv.style.display = 'none'; tv.style.display = 'block'; }
  };

  window.saveSettings = async function () {
    try {
      const ns = {
        work_minutes: parseInt(document.getElementById('s-work').value),
        short_break_minutes: parseInt(document.getElementById('s-break').value),
        long_break_minutes: parseInt(document.getElementById('s-long').value),
        num_sets: parseInt(document.getElementById('s-sets').value),
      };
      updateUI(await invoke('save_settings', { newSettings: ns }));
      window.toggleSettings();
    } catch (e) { showError('save: ' + e); }
  };

  // Slider live values
  ['s-work', 's-break', 's-long', 's-sets'].forEach(id => {
    document.getElementById(id).addEventListener('input', function () {
      document.getElementById(id + '-val').textContent = this.value;
    });
  });

  // --- Tray events ---
  listen('timer-update', (ev) => {
    updateUI(ev.payload);
    if (ev.payload.is_running) startPolling(); else stopPolling();
  });

  // --- Init ---
  (async () => {
    try {
      const s = await invoke('tick');
      updateUI(s);
      if (s.is_running) startPolling();
    } catch (e) { showError('init: ' + e); }
  })();
}
