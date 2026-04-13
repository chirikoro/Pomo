use std::sync::Arc;
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
    _stream_handle: Arc<OutputStreamHandle>,
    sink: Option<Sink>,
    current_mode: BgmMode,
}

impl AudioManager {
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        Some(Self {
            _stream: stream,
            _stream_handle: Arc::new(handle),
            sink: None,
            current_mode: BgmMode::Off,
        })
    }

    pub fn current_mode(&self) -> BgmMode {
        self.current_mode
    }

    pub fn play(&mut self, mode: BgmMode) {
        self.stop();
        self.current_mode = mode;
        if mode == BgmMode::Off {
            return;
        }

        if let Ok(sink) = Sink::try_new(&self._stream_handle) {
            let source: Box<dyn Source<Item = f32> + Send> = match mode {
                BgmMode::Cafe => Box::new(CafeNoise::new()),
                BgmMode::Nature => Box::new(NatureNoise::new()),
                BgmMode::Off => return,
            };
            sink.set_volume(0.15);
            sink.append(source);
            self.sink = Some(sink);
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
    }

    pub fn set_mode(&mut self, mode: BgmMode, is_running: bool) {
        self.current_mode = mode;
        if is_running && mode != BgmMode::Off {
            self.play(mode);
        } else {
            self.stop();
        }
    }

    pub fn on_timer_start(&mut self) {
        if self.current_mode != BgmMode::Off {
            self.play(self.current_mode);
        }
    }

    pub fn on_timer_pause(&mut self) {
        self.stop();
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
            rng_state: 12345,
        }
    }

    fn next_random(&mut self) -> f32 {
        // Simple xorshift64 PRNG
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        // Convert to -1.0..1.0
        (self.rng_state as f32 / u64::MAX as f32) * 2.0 - 1.0
    }
}

impl Iterator for CafeNoise {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let white = self.next_random();
        // Brown noise: integrate white noise with leak
        self.state = self.state * 0.998 + white * 0.02;
        // Soft clip to prevent overflow
        let sample = self.state.tanh() * 0.6;
        Some(sample)
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
            rng_state: 67890,
        }
    }

    fn next_random(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        (self.rng_state as f32 / u64::MAX as f32) * 2.0 - 1.0
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
        let wind = (t * 0.3 * std::f32::consts::TAU).sin() * 0.1;

        let sample = (pink + wind) * 0.5;
        Some(sample.clamp(-1.0, 1.0))
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
