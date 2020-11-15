use std::time::Duration;
use std::time;
use crate::config;

pub struct TimeManager {
    pub tick_time: usize,
    last: std::time::SystemTime,
    sticky_timeout: usize,
    offset_tick: usize,
}

impl TimeManager {
    pub fn tick(&mut self) {
        self.last = time::SystemTime::now();
        self.sticky_timeout = config::INITIAL_TICK_TIME_MS;
        self.offset_tick = 0;
    }

    pub fn get_timeout(&self) -> usize {
        let elapsed = time::SystemTime::now().duration_since(self.last)
            .unwrap_or(Duration::from_millis(0));

        let with_offset = self.offset_tick + elapsed.as_millis() as usize;
        if self.tick_time >= with_offset {
            self.tick_time - with_offset
        } else {
            0
        }
    }

    pub fn should_finish_turn(&self) -> bool {
        self.sticky_timeout <= self.tick_time
    }

    pub fn update_tick_speed(&mut self, cleaned_up: usize) {
        const QUICKENING_COEF: usize = 20;
        const SPEED_CAP: usize = 20;
        let tick_quickening = QUICKENING_COEF * cleaned_up;

        if self.tick_time >= tick_quickening + SPEED_CAP {
            self.tick_time -= tick_quickening;
        } else {
            // cap speed at 20ms per tick
            self.tick_time = SPEED_CAP;
        }
    }

    // for when the piece is layed down it's better if we wait for a timeout
    // instead of just finishing the piece with the tick speed
    pub fn advance_stuck(&mut self) {
        self.sticky_timeout -= self.tick_time;

        if self.sticky_timeout < self.tick_time {
            self.offset_tick = self.tick_time - self.sticky_timeout;
        }

        self.last = time::SystemTime::now();
    }

    pub fn new() -> Self {
        TimeManager {tick_time: config::INITIAL_TICK_TIME_MS, last: time::SystemTime::now(),
                     sticky_timeout: config::INITIAL_TICK_TIME_MS, offset_tick: 0}
    }
}
