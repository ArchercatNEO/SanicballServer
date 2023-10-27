use crate::{
    data::{Client, Clock, MatchConfig, Player, PlayerPosition},
    game::{GameHeader, MessageTypes},
    headers::Header,
    to_byte,
};

pub struct Buffer {
    pub payload: [u8; 1500],
    pub seq: usize,
    ptr: usize,
}

impl Buffer {
    ///Write a single byte into the buffer and shift the pointer to the right.
    pub fn write_byte(&mut self, byte: u8) {
        self.payload[self.ptr] = byte;
        self.ptr += 1;
    }

    pub fn write_string(&mut self, string: &str) {
        let bytes = string.as_bytes();

        let mut size = bytes.len();
        while size >= 0x80 {
            self.write_byte(to_byte(size | 0x80));
            size >>= 7;
        }
        self.write_byte(to_byte(size));

        for byte in bytes.iter() {
            self.write_byte(*byte);
        }
    }

    pub fn write_i32(&mut self, num: &i32) {
        for byte in num.to_le_bytes().iter() {
            self.write_byte(*byte);
        }
    }

    pub fn write_f32(&mut self, num: &f32) {
        for byte in num.to_le_bytes().iter() {
            self.write_byte(*byte);
        }
    }

    pub fn write_bool(&mut self, statement: bool) {
        self.write_byte(statement.into());
    }

    pub fn write_guid(&mut self, guid: Vec<u8>) {
        let size: i32 = guid.len().try_into().unwrap();

        self.write_i32(&size);
        for byte in guid {
            self.write_byte(byte);
        }
    }

    pub fn write_time(&mut self, clock: &mut Clock) {
        self.write_f32(&clock.now());
    }

    pub fn write_game_header(&mut self, header: GameHeader) {
        match header {
            GameHeader::MatchMessage => self.write_byte(0),
            GameHeader::InitMessage => self.write_byte(1),
            GameHeader::PlayerMovementMessage => self.write_byte(2),
        }
    }

    pub fn write_vec3(&mut self, vector: &[f32; 3]) {
        self.write_f32(&vector[0]);
        self.write_f32(&vector[1]);
        self.write_f32(&vector[2]);
    }

    pub fn write_vec4(&mut self, vector: &[f32; 4]) {
        self.write_f32(&vector[0]);
        self.write_f32(&vector[1]);
        self.write_f32(&vector[2]);
        self.write_f32(&vector[3]);
    }

    pub fn write_clients(&mut self, clients: &Vec<Client>) {
        self.write_i32(&clients.len().try_into().unwrap());
        for client in clients.iter() {
            self.write_guid(client.guid.clone());
            self.write_string(&client.name);
        }
    }

    pub fn write_players(&mut self, players: &Vec<Player>) {
        self.write_i32(&players.len().try_into().unwrap());
        for player in players.iter() {
            self.write_guid(player.guid.clone());
            self.write_i32(&player.ctrl_type);
            self.write_bool(player.ready_to_race);
            self.write_i32(&player.char_id);
        }
    }

    pub fn write_settings(&mut self, setting: &MatchConfig) {
        //Match settings properties, written in the order they appear in code
        self.write_i32(&setting.stage_id); //Int32
        self.write_i32(&setting.laps); //Int32
        self.write_i32(&setting.ai_count); //Int32
        self.write_i32(&setting.ai_skill); //Int32 (Cast to AISkillLevel)
        self.write_i32(&setting.auto_start_time); //Int32
        self.write_i32(&setting.auto_start_min_players); //Int32
        self.write_i32(&setting.auto_return_time); //Int32
        self.write_f32(&setting.vote_ratio); //Float
        self.write_i32(&setting.stage_rotation_mode); //Int32 (Cast to StageRotationMode)
    }

    pub fn write_player_position(&mut self, player: &PlayerPosition) {
        self.write_guid(player.guid.clone());
        //self.write_byte(player.ctrl_type as u8);
        self.write_vec3(&player.position);
        self.write_vec4(&player.rotation);
        self.write_vec3(&player.velocity);
        self.write_vec3(&player.angular_velocity);
        self.write_vec3(&player.direction);
    }

    pub fn write_json(&mut self, email: MessageTypes) {
        let json = serde_json::to_string(&email).unwrap();
        let substring = email.as_string();
        let json = sharpize(json, substring);
        self.write_string(&json);
    }

    /// Write the header, sequence and full size into the buffer. You cannot write more after this.
    pub fn write_header(&mut self, header: Header) {
        // Special number that tells Unity how to respond
        self.payload[0] = header as u8;

        // Sequence, used for multi-part convos and split in two for int support
        self.payload[1] = to_byte(self.seq << 1);
        self.payload[2] = to_byte(self.seq >> 7);

        // Remove the assumed 5 bytes then convert to bits
        let bits = (self.ptr - 5) * 8;

        // Message size in bits, split in two for int support
        self.payload[3] = to_byte(bits); // Low bits
        self.payload[4] = to_byte(bits >> 8); // High bits
    }

    pub fn seq(&mut self, counter: usize) {
        self.payload[1] = to_byte(counter << 1);
        self.payload[2] = to_byte(counter >> 7);
    }

    /// Return the slice into the message we should send (a 1500 length array isn't good to send)
    pub fn message(&self) -> Vec<u8> {
        self.payload[..self.ptr].to_owned()
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            payload: [0; 1500],
            ptr: 5,
            seq: 0,
        }
    }
}

///Turn a Rust JSON into a C# JSON
pub fn sharpize(json: String, substring: &'static str) -> String {
    json.replace(
        substring,
        &format!("SanicballCore.MatchMessages.{substring}, SanicballCore"),
    )
}
