use std::{sync::RwLock, time::Instant};

use circular_buffer::CircularBuffer;
use ggez::glam::Vec2;
use once_cell::sync::Lazy;

static DEBUG_POINTS: Lazy<RwLock<CircularBuffer<20, (Instant, Vec2)>>> =
    Lazy::new(|| RwLock::new(CircularBuffer::new()));

static DEBUG_LOG: Lazy<RwLock<CircularBuffer<10, String>>> =
    Lazy::new(|| RwLock::new(CircularBuffer::new()));

pub fn debug_point(p: Vec2) {
    DEBUG_POINTS.write().unwrap().push_back((Instant::now(), p));
}

pub fn get_debug_points() -> Vec<Vec2> {
    DEBUG_POINTS
        .read()
        .unwrap()
        .iter()
        .copied()
        .filter(|(i, _)| i.elapsed().as_secs_f32() < 5.0)
        .map(|(_, p)| p)
        .collect()
}

pub fn clear_debug_points() {
    DEBUG_POINTS.write().unwrap().clear();
}

pub fn debug_string(s: String) {
    DEBUG_LOG.write().unwrap().push_back(s);
}

pub fn get_debug_strings() -> Vec<String> {
    DEBUG_LOG.read().unwrap().iter().cloned().collect()
}

pub fn clear_debug_strings() {
    DEBUG_LOG.write().unwrap().clear();
}
