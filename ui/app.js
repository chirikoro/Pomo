// Wait for Tauri to be ready
window.addEventListener('DOMContentLoaded', () => {
  const { invoke } = window.__TAURI__.core;
  const { listen } = window.__TAURI__.event;

  // --- State ---
  let currentBgm = 'off';
  let currentPhase = 'work';
  let timerInterval = null;

  // --- Phase Colors ---
  const PHASE_COLORS = {
    work:        { color: '#FF6B6B', track: '#FFE0E0' },
    short_break: { color: '#51D88A', track: '#D4F5E0' },
    long_break:  { color: '#9B8EC4', track: '#E8E0F0' },
  };

  // --- DOM refs ---
  const phaseLabel     = document.getElementById('phase-label');
  const timeDisplay    = document.getElementById('time-display');
  const sessionDisplay = document.getElementById('session-display');
  const startBtn       = document.getElementById('start-btn');
  const ringProgress   = document.getElementById('ring-progress');
  const cycleDots      = document.getElementById('cycle-dots');
  const audio1         = document.getElementById('audio-1');
  const audio2         = document.getElementById('audio-2');

  const CIRCUMFERENCE = 2 * Math.PI * 84;

  // --- Timer polling ---
  function startPolling() {
    if (timerInterval) return;
    timerInterval = setInterval(async () => {
      try {
        const status = await invoke('tick');
        updateUI(status);
        if (status.phase_finished) {
          onPhaseFinished(status);
        }
      } catch (e) { console.error('tick error:', e); }
    }, 200);
  }

  function stopPolling() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  function updateUI(status) {
    const colors = PHASE_COLORS[status.phase] || PHASE_COLORS.work;
    document.documentElement.style.setProperty('--phase-color', colors.color);
    document.documentElement.style.setProperty('--phase-track', colors.track);

    const labels = { work: 'WORK', short_break: 'SHORT BREAK', long_break: 'LONG BREAK' };
    phaseLabel.textContent = labels[status.phase] || 'WORK';

    const mins = Math.floor(status.remaining_secs / 60);
    const secs = status.remaining_secs % 60;
    timeDisplay.textContent =
      `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;

    const offset = CIRCUMFERENCE * (1 - status.progress);
    ringProgress.style.strokeDashoffset = offset;

    sessionDisplay.textContent = `Session ${status.cycle}/${status.num_sets}`;

    if (status.is_running) {
      startBtn.textContent = 'Pause';
    } else if (status.state === 'finished') {
      startBtn.textContent = 'Next';
    } else {
      startBtn.textContent = 'Start';
    }

    renderCycleDots(status.cycle, status.num_sets);

    if (status.phase !== currentPhase) {
      currentPhase = status.phase;
      if (currentBgm !== 'off') {
        playBgm(currentBgm, currentPhase);
      }
      updateBgmButtons();
    }
  }

  function renderCycleDots(cycle, numSets) {
    if (cycleDots.children.length !== numSets) {
      cycleDots.innerHTML = '';
      for (let i = 1; i <= numSets; i++) {
        const dot = document.createElement('div');
        dot.className = 'dot' + (i <= cycle ? ' filled' : '');
        cycleDots.appendChild(dot);
      }
    } else {
      for (let i = 0; i < numSets; i++) {
        cycleDots.children[i].classList.toggle('filled', i < cycle);
      }
    }
  }

  async function onPhaseFinished(status) {
    try {
      const newStatus = await invoke('skip');
      updateUI(newStatus);
    } catch (e) { console.error('skip error:', e); }

    try {
      const msgs = {
        work: ['Work Complete!', 'Time for a break.'],
        short_break: ['Break Over!', 'Ready to focus?'],
        long_break: ['Long Break Over!', 'Starting a new cycle.'],
      };
      const [title, body] = msgs[status.phase] || ['Phase Complete', ''];
      new Notification(title, { body });
    } catch (e) { /* ignore */ }
  }

  // --- Controls (exposed globally for onclick) ---
  window.toggleStartPause = async function () {
    try {
      const status = await invoke('tick');
      let newStatus;
      if (status.is_running) {
        newStatus = await invoke('pause');
        stopPolling();
      } else {
        newStatus = await invoke('start');
        startPolling();
      }
      updateUI(newStatus);
      if (newStatus.is_running && currentBgm !== 'off') {
        playBgm(currentBgm, currentPhase);
      }
    } catch (e) { console.error('toggleStartPause error:', e); }
  };

  window.doReset = async function () {
    try {
      stopPolling();
      const status = await invoke('reset');
      updateUI(status);
      stopAllAudio();
    } catch (e) { console.error('reset error:', e); }
  };

  window.doSkip = async function () {
    try {
      const status = await invoke('skip');
      updateUI(status);
      if (currentBgm !== 'off') {
        playBgm(currentBgm, status.phase);
      }
    } catch (e) { console.error('skip error:', e); }
  };

  // --- BGM ---
  window.setBgm = function (mode) {
    currentBgm = mode;
    updateBgmButtons();
    if (mode === 'off') {
      stopAllAudio();
      return;
    }
    playBgm(mode, currentPhase);
  };

  function updateBgmButtons() {
    document.querySelectorAll('.bgm-btn').forEach(btn => {
      btn.classList.toggle('active', btn.dataset.mode === currentBgm);
    });
  }

  function playBgm(mode, phase) {
    stopAllAudio();
    const isWork = (phase === 'work');

    if (mode === 'nature') {
      audio1.src = isWork ? 'assets/bgm/nature_work.mp3' : 'assets/bgm/nature_break.mp3';
      audio1.volume = 0.5;
      audio1.play().catch(() => {});
    } else if (mode === 'cafe') {
      if (isWork) {
        audio1.src = 'assets/bgm/cafe_work_1.mp3';
        audio2.src = 'assets/bgm/cafe_work_2.mp3';
        audio1.volume = 0.5;
        audio2.volume = 0.5;
        audio1.play().catch(() => {});
        audio2.play().catch(() => {});
      } else {
        audio1.src = 'assets/bgm/cafe_break.mp3';
        audio1.volume = 0.5;
        audio1.play().catch(() => {});
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
      sv.style.display = 'block';
      tv.style.display = 'none';
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
      } catch (e) { console.error('loadSettings error:', e); }
    } else {
      sv.style.display = 'none';
      tv.style.display = 'block';
    }
  };

  window.saveSettings = async function () {
    try {
      const newSettings = {
        work_minutes: parseInt(document.getElementById('s-work').value),
        short_break_minutes: parseInt(document.getElementById('s-break').value),
        long_break_minutes: parseInt(document.getElementById('s-long').value),
        num_sets: parseInt(document.getElementById('s-sets').value),
      };
      const status = await invoke('save_settings', { newSettings });
      updateUI(status);
      window.toggleSettings();
    } catch (e) { console.error('saveSettings error:', e); }
  };

  // Slider live values
  ['s-work', 's-break', 's-long', 's-sets'].forEach(id => {
    document.getElementById(id).addEventListener('input', function () {
      document.getElementById(id + '-val').textContent = this.value;
    });
  });

  // --- Listen for tray events ---
  listen('timer-update', (event) => {
    updateUI(event.payload);
    if (event.payload.is_running) startPolling();
    else stopPolling();
  });

  // --- Init ---
  (async () => {
    try {
      const status = await invoke('tick');
      updateUI(status);
      if (status.is_running) startPolling();
    } catch (e) { console.error('init error:', e); }
  })();
});
