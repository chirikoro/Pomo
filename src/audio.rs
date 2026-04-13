use std::time::Duration;

use rodio::source::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink};

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
    sink: Option<Sink>,
    current_mode: BgmMode,
    is_playing: bool,
}

impl AudioManager {
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Some(Self {
                _stream: stream,
                stream_handle: handle,
                sink: None,
                current_mode: BgmMode::Off,
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

    pub fn set_mode(&mut self, mode: BgmMode) {
        let was_playing = self.is_playing;
        self.stop_playback();
        self.current_mode = mode;
        if mode != BgmMode::Off {
            // Always start playing when a BGM mode is selected
            self.start_playback();
        } else if was_playing {
            // Switching to Off: already stopped above
        }
    }

    pub fn on_timer_start(&mut self) {
        // Resume BGM if a mode is selected but not currently playing
        if self.current_mode != BgmMode::Off && !self.is_playing {
            self.start_playback();
        }
    }

    pub fn on_timer_pause(&mut self) {
        // Keep BGM playing during pause - it's ambient background music
        // Only stop if user explicitly switches to Off
    }

    pub fn on_timer_reset(&mut self) {
        self.stop_playback();
    }

    fn start_playback(&mut self) {
        self.stop_playback();

        let mode = self.current_mode;
        if mode == BgmMode::Off {
            return;
        }

        match Sink::try_new(&self.stream_handle) {
            Ok(sink) => {
                let source: Box<dyn Source<Item = f32> + Send> = match mode {
                    BgmMode::Cafe => Box::new(CafeNoise::new()),
                    BgmMode::Nature => Box::new(NatureNoise::new()),
                    BgmMode::Off => return,
                };
                sink.set_volume(0.5);
                sink.append(source);
                self.sink = Some(sink);
                self.is_playing = true;
            }
            Err(e) => {
                eprintln!("[Pomo] Failed to create audio sink: {}", e);
            }
        }
    }

    fn stop_playback(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.is_playing = false;
    }
}

// --- Procedural noise sources ---

/// Brown noise for cafe ambience - warm, low-frequency rumble
struct CafeNoise {
    sample_rate: u32,
    state: f32,
    rng_state: u64,
}

impl CafeNoise {
    fn new() -> Self {
        Self {
            sample_rate: 44100,
            state: 0.0,
            // Use a well-distributed initial seed
            rng_state: 0xDEAD_BEEF_CAFE_1234,
        }
    }

    fn next_random(&mut self) -> f32 {
        // xorshift64
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        // Map to -1.0..1.0 using upper bits for better distribution
        let bits = (self.rng_state >> 40) as u32; // top 24 bits
        (bits as f32 / 0x00FF_FFFF as f32) * 2.0 - 1.0
    }
}

impl Iterator for CafeNoise {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let white = self.next_random();
        // Brown noise: integrate white noise with leak
        self.state = self.state * 0.997 + white * 0.04;
        // Soft clip
        Some(self.state.tanh())
    }
}

impl Source for CafeNoise {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Pink noise for nature/rain sounds
struct NatureNoise {
    sample_rate: u32,
    // Voss-McCartney pink noise algorithm
    rows: [f32; 16],
    running_sum: f32,
    index: u32,
    rng_state: u64,
}

impl NatureNoise {
    fn new() -> Self {
        Self {
            sample_rate: 44100,
            rows: [0.0; 16],
            running_sum: 0.0,
            index: 0,
            // Use a well-distributed initial seed
            rng_state: 0xCAFE_BABE_1337_5678,
        }
    }

    fn next_random(&mut self) -> f32 {
        // xorshift64
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        // Map to -1.0..1.0 using upper bits
        let bits = (self.rng_state >> 40) as u32;
        (bits as f32 / 0x00FF_FFFF as f32) * 2.0 - 1.0
    }
}

impl Iterator for NatureNoise {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.index = self.index.wrapping_add(1);

        // Voss-McCartney: update one row based on trailing zeros
        let tz = self.index.trailing_zeros() as usize;
        if tz < self.rows.len() {
            self.running_sum -= self.rows[tz];
            let new_val = self.next_random();
            self.rows[tz] = new_val;
            self.running_sum += new_val;
        }

        let white = self.next_random();
        let pink = (self.running_sum + white) / (self.rows.len() as f32 + 1.0);

        // Add subtle low-frequency modulation for "wind" effect
        let t = self.index as f32 / self.sample_rate as f32;
        let wind = (t * 0.3 * std::f32::consts::TAU).sin() * 0.15;

        let sample = (pink * 1.5 + wind).clamp(-1.0, 1.0);
        Some(sample)
    }
}

impl Source for NatureNoise {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
