use std::time::{Duration, Instant};

use sdl2::EventPump;
use tiny_sdl2_gui::widget::SDLEvent;

/// a helper for the examples. but could do done in a variety of ways
#[allow(dead_code)]
pub fn gui_loop<F>(max_delay: Duration, event_pump: &mut EventPump, mut handler: F)
where
    F: FnMut(&mut [SDLEvent]) -> bool // true iff leave
{
    // accumulate the events for this frame
    let mut events_accumulator: Vec<SDLEvent> = Vec::new();
    'running: loop {
        // wait forever since nothing has happened yet!
        let event = event_pump.wait_event();
        let oldest_event = Instant::now(); // immediately after event received
        if let sdl2::event::Event::Quit { .. } = event {
            break 'running;
        }
        events_accumulator.push(SDLEvent::new(event));

        // don't send off the event immediately! wait a bit and accumulate
        // several events to be processed together. max bound on waiting so that
        // the first event received isn't too stale
        loop {
            let max_time = oldest_event + max_delay;
            let now = Instant::now();
            if max_time <= now {
                break; // can't wait any longer
            }

            let time_to_wait = max_time - now;
            // cast ok since bounded max_delay
            let time_to_wait = time_to_wait.as_millis() as u32;
            let event = match event_pump.wait_event_timeout(time_to_wait) {
                None => break, // waited too long
                Some(v) => v,
            };
            if let sdl2::event::Event::Quit { .. } = event {
                break 'running;
            }
            events_accumulator.push(SDLEvent::new(event));
        }

        if handler(&mut events_accumulator) {
            break 'running;
        }
        events_accumulator.clear(); // clear after use
    }
}