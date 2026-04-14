use std::io::Cursor;

use rodio::source::Source;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::timer::Phase;

// Embed MP3 files at compile time
const NATURE_WORK: &[u8] = include_bytes!("../assets/bgm/nature_work.mp3");
const NATURE_BREAK: &[u8] = include_bytes!("../assets/bgm/nature_break.mp3");
const CAFE_WORK_1: &[u8] = include_bytes!("../assets/bgm/cafe_work_1.mp3");
const CAFE_WORK_2: &[u8] = include_bytes!("../assets/bgm/cafe_work_2.mp3");
const CAFE_BREAK: &[u8] = include_bytes!("../assets/bgm/cafe_break.mp3");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgmMode {
    Off,
    Cafe,
    Nature,
}

impl BgmMode {
    pub fn label(&self) -> &'static str {
        match self {
            BgmMode::Off => "Off",
            BgmMode::Cafe => "Cafe",
            BgmMode::Nature => "Nature",
        }
    }

    pub fn all() -> &'static [BgmMode] {
        &[BgmMode::Cafe, BgmMode::Nature, BgmMode::Off]
    }
}

pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: Vec<Sink>,
    current_mode: BgmMode,
    current_phase: Phase,
    is_playing: bool,
}

impl AudioManager {
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Some(Self {
                _stream: stream,
                stream_handle: handle,
                sinks: Vec::new(),
                current_mode: BgmMode::Off,
                current_phase: Phase::Work,
                is_playing: false,
            }),
            Err(e) => {
                eprintln!("[Pomo] Audio init failed: {}. BGM disabled.", e);
                None
            }
        }
    }

    pub fn current_mode(&self) -> BgmMode {
        self.current_mode
    }

    /// Called when the user selects a BGM mode
    pub fn set_mode(&mut self, mode: BgmMode) {
        self.current_mode = mode;
        if mode == BgmMode::Off {
            self.stop_playback();
        } else {
            self.start_for_phase(self.current_phase);
        }
    }

    /// Called when the phase changes (Work <-> Break)
    pub fn on_phase_change(&mut self, phase: Phase) {
        self.current_phase = phase;
        if self.current_mode != BgmMode::Off && self.is_playing {
            self.start_for_phase(phase);
        }
    }

    pub fn on_timer_start(&mut self, phase: Phase) {
        self.current_phase = phase;
        if self.current_mode != BgmMode::Off && !self.is_playing {
            self.start_for_phase(phase);
        }
    }

    pub fn on_timer_pause(&mut self) {
        // Keep BGM playing during pause
    }

    pub fn on_timer_reset(&mut self) {
        self.stop_playback();
    }

    fn start_for_phase(&mut self, phase: Phase) {
        self.stop_playback();

        let is_work = matches!(phase, Phase::Work);

        match self.current_mode {
            BgmMode::Nature => {
                let data = if is_work { NATURE_WORK } else { NATURE_BREAK };
                self.play_looped(data);
            }
            BgmMode::Cafe => {
                if is_work {
                    // 2 tracks simultaneously, each looping
                    self.play_looped(CAFE_WORK_1);
                    self.play_looped(CAFE_WORK_2);
                } else {
                    self.play_looped(CAFE_BREAK);
                }
            }
            BgmMode::Off => {}
        }

        if !self.sinks.is_empty() {
            self.is_playing = true;
        }
    }

    fn play_looped(&mut self, mp3_data: &'static [u8]) {
        let Ok(sink) = Sink::try_new(&self.stream_handle) else {
            eprintln!("[Pomo] Failed to create audio sink");
            return;
        };

        match Decoder::new(Cursor::new(mp3_data)) {
            Ok(source) => {
                sink.set_volume(0.5);
                sink.append(source.repeat_infinite());
                self.sinks.push(sink);
            }
            Err(e) => {
                eprintln!("[Pomo] Failed to decode MP3: {}", e);
            }
        }
    }

    fn stop_playback(&mut self) {
        for sink in self.sinks.drain(..) {
            sink.stop();
        }
        self.is_playing = false;
    }
}
