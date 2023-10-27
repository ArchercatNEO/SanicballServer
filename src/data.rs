use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Instant};

use crate::game::CtrlType;

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub public: bool,
    pub servers: Vec<String>,
    pub ip: String,
    pub port: i32,
    pub max_players: u8,
    pub enabled_connections: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MatchConfig {
    pub stage_id: i32,
    pub laps: i32,
    pub ai_count: i32,
    pub ai_skill: i32,
    pub auto_start_time: i32,
    pub auto_start_min_players: i32,
    pub auto_return_time: i32,
    pub vote_ratio: f32,
    pub stage_rotation_mode: i32,
}

#[derive(Deserialize, Debug)]
pub struct Motd {
    pub text: String,
}

pub struct Clock {
    pub start_time: Instant,
    pub running: Timer,
}

impl Clock {
    pub fn now(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }
}

impl Default for Clock {
    fn default() -> Self {
        Clock {
            start_time: Instant::now(),
            running: Timer::AutoStartTimer,
        }
    }
}
pub enum Timer {
    ServerPing,
    Lobby,
    StageLoadTimeout,
    AutoStartTimer,
    BackToLobby,
}

pub struct Settings {
    pub stage_id: i32,
    pub laps: i32,
    pub ai_count: i32,
    pub ai_skill: i32,
    pub auto_start_time: i32,
    pub auto_start_min_players: i32,
    pub auto_return_time: i32,
    pub vote_ratio: f32,
    pub stage_rotation_mode: i32,
}

#[derive(Clone)]
pub struct Stopwatch {}

#[derive(Clone)]
pub struct Client {
    pub guid: Vec<u8>,
    pub name: String,
    pub connection: SocketAddr,
    pub is_loading: bool,
    pub wants_lobby: bool,
    pub counter: usize
}

#[derive(Clone)]
pub struct Player {
    pub guid: Vec<u8>,
    pub ctrl_type: i32,
    pub char_id: i32,
    pub ready_to_race: bool,
    pub is_racing: bool,
    pub race_timeout: Stopwatch,
    pub has_timed_out: bool,
}

pub struct PlayerPosition {
    pub guid: Vec<u8>,
    pub ctrl_type: CtrlType,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub direction: [f32; 3],
}

pub enum CharacterTier {
    Normal = 0,
    Odd = 1,
    Hypersonic = 2,
}
