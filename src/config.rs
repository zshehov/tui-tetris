pub const END_PLAYING_SCREEN_X : usize = 74;
pub const END_SCREEN_Y : usize = 54;

pub const BLOCK_HEIGHT : usize = 2;
pub const BLOCK_WIDTH : usize = BLOCK_HEIGHT * 2;
pub const LEFT_THRESHOLD : usize = 0;
pub const RIGHT_THRESHOLD : usize = END_PLAYING_SCREEN_X / BLOCK_WIDTH;
pub const BOTTOM_THRESHOLD : usize = END_SCREEN_Y / BLOCK_HEIGHT;
pub const INITIAL_TICK_TIME_MS : usize = 1000;
