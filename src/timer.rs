use std::time::{Duration, Instant};

use crate::settings::PomoSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn label(&self) -> &'static str {
        match self {
            Phase::Work => "WORK",
            Phase::ShortBreak => "SHORT BREAK",
            Phase::LongBreak => "LONG BREAK",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimerState {
    Idle,
    Running {
        started_at: Instant,
        remaining_at_start: Duration,
    },
    Paused {
        remaining: Duration,
    },
    Finished,
}

pub struct PomodoroTimer {
    pub phase: Phase,
    pub state: TimerState,
    pub cycle: u32,
    pub num_sets: u32,
    durations: PhaseDurations,
}

struct PhaseDurations {
    work: Duration,
    short_break: Duration,
    long_break: Duration,
}

impl PomodoroTimer {
    pub fn new(settings: &PomoSettings) -> Self {
        Self {
            phase: Phase::Work,
            state: TimerState::Idle,
            cycle: 1,
            num_sets: settings.num_sets,
            durations: PhaseDurations {
                work: Duration::from_secs(settings.work_minutes as u64 * 60),
                short_break: Duration::from_secs(settings.short_break_minutes as u64 * 60),
                long_break: Duration::from_secs(settings.long_break_minutes as u64 * 60),
            },
        }
    }

    pub fn apply_settings(&mut self, settings: &PomoSettings) {
        self.num_sets = settings.num_sets;
        self.durations = PhaseDurations {
            work: Duration::from_secs(settings.work_minutes as u64 * 60),
            short_break: Duration::from_secs(settings.short_break_minutes as u64 * 60),
            long_break: Duration::from_secs(settings.long_break_minutes as u64 * 60),
        };
    }

    pub fn start(&mut self) {
        match self.state {
            TimerState::Idle => {
                let dur = self.phase_duration();
                self.state = TimerState::Running {
                    started_at: Instant::now(),
                    remaining_at_start: dur,
                };
            }
            TimerState::Paused { remaining } => {
                self.state = TimerState::Running {
                    started_at: Instant::now(),
                    remaining_at_start: remaining,
                };
            }
            TimerState::Finished => {
                self.advance_phase();
                self.start();
            }
            TimerState::Running { .. } => {}
        }
    }

    pub fn pause(&mut self) {
        if let TimerState::Running {
            started_at,
            remaining_at_start,
        } = self.state
        {
            let elapsed = started_at.elapsed();
            let remaining = remaining_at_start.saturating_sub(elapsed);
            self.state = TimerState::Paused { remaining };
        }
    }

    pub fn reset(&mut self) {
        self.state = TimerState::Idle;
    }

    pub fn full_reset(&mut self) {
        self.phase = Phase::Work;
        self.state = TimerState::Idle;
        self.cycle = 1;
    }

    pub fn skip(&mut self) {
        self.advance_phase();
        self.state = TimerState::Idle;
    }

    /// Called each frame. Returns true if the phase just finished.
    pub fn tick(&mut self) -> bool {
        if let TimerState::Running {
            started_at,
            remaining_at_start,
        } = self.state
        {
            let elapsed = started_at.elapsed();
            if elapsed >= remaining_at_start {
                self.state = TimerState::Finished;
                return true;
            }
        }
        false
    }

    pub fn remaining(&self) -> Duration {
        match self.state {
            TimerState::Idle => self.phase_duration(),
            TimerState::Running {
                started_at,
                remaining_at_start,
            } => remaining_at_start.saturating_sub(started_at.elapsed()),
            TimerState::Paused { remaining } => remaining,
            TimerState::Finished => Duration::ZERO,
        }
    }

    /// Progress from 0.0 (just started) to 1.0 (finished)
    pub fn progress(&self) -> f32 {
        let total = self.phase_duration().as_secs_f32();
        if total == 0.0 {
            return 1.0;
        }
        let remaining = self.remaining().as_secs_f32();
        1.0 - (remaining / total)
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, TimerState::Running { .. })
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.state, TimerState::Idle)
    }

    fn phase_duration(&self) -> Duration {
        match self.phase {
            Phase::Work => self.durations.work,
            Phase::ShortBreak => self.durations.short_break,
            Phase::LongBreak => self.durations.long_break,
        }
    }

    fn advance_phase(&mut self) {
        match self.phase {
            Phase::Work => {
                if self.cycle >= self.num_sets {
                    self.phase = Phase::LongBreak;
                } else {
                    self.phase = Phase::ShortBreak;
                }
            }
            Phase::ShortBreak => {
                self.cycle += 1;
                self.phase = Phase::Work;
            }
            Phase::LongBreak => {
                self.cycle = 1;
                self.phase = Phase::Work;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_settings() -> PomoSettings {
        PomoSettings {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            num_sets: 4,
        }
    }

    #[test]
    fn test_initial_state() {
        let timer = PomodoroTimer::new(&default_settings());
        assert_eq!(timer.phase, Phase::Work);
        assert!(timer.is_idle());
        assert_eq!(timer.cycle, 1);
    }

    #[test]
    fn test_start_pause_resume() {
        let mut timer = PomodoroTimer::new(&default_settings());
        timer.start();
        assert!(timer.is_running());

        timer.pause();
        assert!(matches!(timer.state, TimerState::Paused { .. }));

        timer.start();
        assert!(timer.is_running());
    }

    #[test]
    fn test_reset() {
        let mut timer = PomodoroTimer::new(&default_settings());
        timer.start();
        timer.reset();
        assert!(timer.is_idle());
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60));
    }

    #[test]
    fn test_progress() {
        let timer = PomodoroTimer::new(&default_settings());
        assert!((timer.progress() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_phase_advancement() {
        let mut timer = PomodoroTimer::new(&default_settings());
        // Work -> ShortBreak
        timer.skip();
        assert_eq!(timer.phase, Phase::ShortBreak);
        assert_eq!(timer.cycle, 1);

        // ShortBreak -> Work (cycle 2)
        timer.skip();
        assert_eq!(timer.phase, Phase::Work);
        assert_eq!(timer.cycle, 2);
    }

    #[test]
    fn test_long_break_after_n_sets() {
        let mut timer = PomodoroTimer::new(&default_settings());
        // Complete 4 work sessions
        for i in 1..=4 {
            assert_eq!(timer.phase, Phase::Work);
            assert_eq!(timer.cycle, i);
            timer.skip(); // Work -> Break
            if i < 4 {
                assert_eq!(timer.phase, Phase::ShortBreak);
                timer.skip(); // ShortBreak -> Work
            }
        }
        assert_eq!(timer.phase, Phase::LongBreak);

        // LongBreak -> Work (cycle resets to 1)
        timer.skip();
        assert_eq!(timer.phase, Phase::Work);
        assert_eq!(timer.cycle, 1);
    }

    #[test]
    fn test_custom_sets() {
        let settings = PomoSettings {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            num_sets: 2,
        };
        let mut timer = PomodoroTimer::new(&settings);

        // Work 1 -> ShortBreak
        timer.skip();
        assert_eq!(timer.phase, Phase::ShortBreak);

        // ShortBreak -> Work 2
        timer.skip();
        assert_eq!(timer.phase, Phase::Work);
        assert_eq!(timer.cycle, 2);

        // Work 2 -> LongBreak (after 2 sets)
        timer.skip();
        assert_eq!(timer.phase, Phase::LongBreak);
    }
}
