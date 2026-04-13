use crate::timer::Phase;

pub fn notify_phase_complete(phase: &Phase) {
    let (summary, body) = match phase {
        Phase::Work => ("Work Complete!", "Time for a break."),
        Phase::ShortBreak => ("Break Over!", "Ready to focus?"),
        Phase::LongBreak => ("Long Break Over!", "Starting a new cycle."),
    };
    let _ = notify_rust::Notification::new()
        .summary(summary)
        .body(body)
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show();
}
