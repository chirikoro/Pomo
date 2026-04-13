#[derive(Debug, Clone)]
pub struct PomoSettings {
    pub work_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub num_sets: u32,
}

impl Default for PomoSettings {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            num_sets: 4,
        }
    }
}

impl PomoSettings {
    pub fn clamp(&mut self) {
        self.work_minutes = self.work_minutes.clamp(1, 60);
        self.short_break_minutes = self.short_break_minutes.clamp(1, 30);
        self.long_break_minutes = self.long_break_minutes.clamp(1, 60);
        self.num_sets = self.num_sets.clamp(1, 10);
    }
}
