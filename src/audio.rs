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
        self.stop_playback();
        self.current_mode = mode;
        if mode != BgmMode::Off {
            self.start_playback();
        }
    }

    pub fn on_timer_start(&mut self) {
        if self.current_mode != BgmMode::Off && !self.is_playing {
            self.start_playback();
        }
    }

    pub fn on_timer_pause(&mut self) {
        // Keep BGM playing during pause
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
                    BgmMode::Cafe => Box::new(CafeAmbience::new()),
                    BgmMode::Nature => Box::new(RainAmbience::new()),
                    BgmMode::Off => return,
                };
                sink.set_volume(0.45);
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

// ============================================================
// Shared PRNG
// ============================================================

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Returns a uniform random f32 in -1.0..1.0
    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        let bits = (self.0 >> 40) as u32;
        (bits as f32 / 0x00FF_FFFF as f32) * 2.0 - 1.0
    }

    /// Returns a uniform random f32 in 0.0..1.0
    fn next_unit(&mut self) -> f32 {
        (self.next_f32() + 1.0) * 0.5
    }
}

// ============================================================
// Simple biquad low-pass filter
// ============================================================

struct LowPass {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl LowPass {
    fn new(sample_rate: f32, cutoff: f32, q: f32) -> Self {
        let w0 = std::f32::consts::TAU * cutoff / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let b0 = (1.0 - cos_w0) / 2.0;
        let b1 = 1.0 - cos_w0;
        let b2 = (1.0 - cos_w0) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

// ============================================================
// Cafe Ambience
//   - Deep room rumble (very low-pass filtered noise)
//   - Muffled conversation murmur (bandpass-ish filtered noise,
//     with slow volume swell)
//   - Occasional soft cup/tap sounds (short noise bursts)
// ============================================================

struct CafeAmbience {
    sample_rate: u32,
    rng: Rng,
    // Room rumble
    rumble_filter: LowPass,
    // Murmur layer
    murmur_filter: LowPass,
    murmur_envelope: f32,
    murmur_target: f32,
    murmur_speed: f32,
    // Cup clink / tap accents
    tap_countdown: u32,
    tap_envelope: f32,
    tap_filter: LowPass,
    // Sample counter
    sample_idx: u64,
}

impl CafeAmbience {
    fn new() -> Self {
        let sr = 44100.0;
        Self {
            sample_rate: 44100,
            rng: Rng::new(0xDEAD_BEEF_CAFE_1234),
            // Very low rumble: 80 Hz cutoff
            rumble_filter: LowPass::new(sr, 80.0, 0.5),
            // Murmur: 600 Hz cutoff gives muffled voice-like quality
            murmur_filter: LowPass::new(sr, 600.0, 0.8),
            murmur_envelope: 0.0,
            murmur_target: 0.3,
            murmur_speed: 0.00001,
            // Taps every ~1-3 seconds
            tap_countdown: 44100,
            tap_envelope: 0.0,
            tap_filter: LowPass::new(sr, 2500.0, 1.2),
            sample_idx: 0,
        }
    }
}

impl Iterator for CafeAmbience {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.sample_idx += 1;
        let white = self.rng.next_f32();

        // --- Layer 1: Room rumble ---
        let rumble = self.rumble_filter.process(white) * 0.4;

        // --- Layer 2: Murmur (voice-like filtered noise with volume swell) ---
        // Slowly drift envelope toward target, then pick a new target
        if (self.murmur_envelope - self.murmur_target).abs() < 0.01 {
            self.murmur_target = self.rng.next_unit() * 0.5 + 0.1; // 0.1 to 0.6
            self.murmur_speed = self.rng.next_unit() * 0.00003 + 0.000005;
        }
        if self.murmur_envelope < self.murmur_target {
            self.murmur_envelope += self.murmur_speed;
        } else {
            self.murmur_envelope -= self.murmur_speed;
        }
        let murmur_raw = self.murmur_filter.process(self.rng.next_f32());
        let murmur = murmur_raw * self.murmur_envelope;

        // --- Layer 3: Occasional taps/clinks ---
        if self.tap_countdown == 0 {
            self.tap_envelope = 1.0;
            // Next tap in 0.8 to 3 seconds
            let delay = (self.rng.next_unit() * 2.2 + 0.8) * self.sample_rate as f32;
            self.tap_countdown = delay as u32;
        } else {
            self.tap_countdown -= 1;
        }
        // Fast exponential decay for tap
        self.tap_envelope *= 0.9993;
        let tap_noise = self.rng.next_f32();
        let tap = self.tap_filter.process(tap_noise) * self.tap_envelope * 0.6;

        // Mix
        let sample = rumble + murmur + tap;
        Some(sample.clamp(-1.0, 1.0))
    }
}

impl Source for CafeAmbience {
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

// ============================================================
// Rain Ambience
//   - Steady rain bed (filtered noise with gentle modulation)
//   - Individual raindrop impacts (very short high-pitched pings
//     at random intervals)
//   - Slow wind swell (very low frequency modulation)
// ============================================================

struct RainAmbience {
    sample_rate: u32,
    rng: Rng,
    // Rain bed
    rain_filter: LowPass,
    // Wind modulation
    wind_phase: f32,
    wind_speed: f32,
    // Raindrop impacts (pool of 8 concurrent drops)
    drops: [Raindrop; 8],
    drop_spawn_counter: u32,
    // Sample counter
    sample_idx: u64,
}

#[derive(Clone, Copy)]
struct Raindrop {
    active: bool,
    envelope: f32,
    decay: f32,
    pitch: f32,  // frequency of the "ping"
    phase: f32,
}

impl Default for Raindrop {
    fn default() -> Self {
        Self {
            active: false,
            envelope: 0.0,
            decay: 0.999,
            pitch: 3000.0,
            phase: 0.0,
        }
    }
}

impl RainAmbience {
    fn new() -> Self {
        let sr = 44100.0;
        Self {
            sample_rate: 44100,
            rng: Rng::new(0xCAFE_BABE_0000_5678),
            // Rain bed: 3kHz cutoff for "sssh" character
            rain_filter: LowPass::new(sr, 3000.0, 0.6),
            wind_phase: 0.0,
            wind_speed: 0.08, // very slow oscillation
            drops: [Raindrop::default(); 8],
            drop_spawn_counter: 200,
            sample_idx: 0,
        }
    }

    fn spawn_drop(&mut self) {
        // Find an inactive slot
        for drop in &mut self.drops {
            if !drop.active {
                drop.active = true;
                drop.envelope = 0.6 + self.rng.next_unit() * 0.4; // 0.6 to 1.0
                // Faster decay = shorter, sharper drip
                drop.decay = 0.9990 + self.rng.next_unit() * 0.0008; // 0.9990 to 0.9998
                // Random pitch between 2kHz and 6kHz for variety
                drop.pitch = 2000.0 + self.rng.next_unit() * 4000.0;
                drop.phase = 0.0;
                return;
            }
        }
    }
}

impl Iterator for RainAmbience {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.sample_idx += 1;
        let sr = self.sample_rate as f32;

        // --- Layer 1: Rain bed (filtered noise + wind modulation) ---
        let white = self.rng.next_f32();
        let rain_raw = self.rain_filter.process(white);

        // Wind: slow sine modulation of rain volume
        self.wind_phase += self.wind_speed / sr;
        if self.wind_phase > 1.0 {
            self.wind_phase -= 1.0;
            // Slightly vary wind speed each cycle
            self.wind_speed = 0.05 + self.rng.next_unit() * 0.1;
        }
        let wind_mod = 0.55 + (self.wind_phase * std::f32::consts::TAU).sin() * 0.35;
        let rain = rain_raw * wind_mod * 0.45;

        // --- Layer 2: Individual raindrop impacts ---
        if self.drop_spawn_counter == 0 {
            self.spawn_drop();
            // Spawn every 30ms to 150ms (very frequent, like real rain)
            let interval = (self.rng.next_unit() * 0.12 + 0.03) * sr;
            self.drop_spawn_counter = interval as u32;
        } else {
            self.drop_spawn_counter -= 1;
        }

        let mut drops_sum = 0.0f32;
        for drop in &mut self.drops {
            if !drop.active {
                continue;
            }
            // Sine ping with exponential decay
            drop.phase += drop.pitch / sr;
            let ping = (drop.phase * std::f32::consts::TAU).sin();
            drops_sum += ping * drop.envelope * 0.15;
            drop.envelope *= drop.decay;
            if drop.envelope < 0.001 {
                drop.active = false;
            }
        }

        // Mix
        let sample = rain + drops_sum;
        Some(sample.clamp(-1.0, 1.0))
    }
}

impl Source for RainAmbience {
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
