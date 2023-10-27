use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use std::usize;
use std::vec;

use crate::data::{Clock, MatchConfig, Motd, ServerConfig, Stopwatch};
use crate::game::{ChatMessageType, GameHeader, MessageTypes};
use crate::oxidize;
use crate::{
    buffer::Buffer,
    data::{Client, Player},
    headers::Header,
    stream::Stream,
};

pub struct Server {
    app_id: &'static str,

    listener: UdpSocket,

    config: ServerConfig,
    match_settings: MatchConfig,
    motd: Motd,

    clock: Clock,

    clients: Vec<Client>,
    players: Vec<Player>,

    buffer: Buffer,
    stream: Stream,
}

impl Server {
    // ? Helper functions unrelated to most logic handling

    ///Send the current buffer to a specified adress
    fn send_to(&mut self, header: Header, addr: SocketAddr) {
        //Add the header
        self.buffer.write_header(header);

        //Return a small vector that send
        let message = self.buffer.message();
        self.listener.send_to(&message, addr).unwrap();
    }

    //Send the buffer to every specified adress
    fn send_all(&mut self, header: Header) {
        //Add the header
        self.buffer.write_header(header);

        //Return a small vector that send
        for client in self.clients.iter_mut() {
            self.buffer.seq(client.counter);
            client.counter += 1;
            let message = self.buffer.message();
            self.listener.send_to(&message, client.connection).unwrap();
        }
    }

    fn send_new(&mut self, header: MessageTypes) {
        let mut buffer = Buffer::default();

        buffer.write_game_header(GameHeader::MatchMessage);
        buffer.write_time(&mut self.clock);
        buffer.write_json(header);
        buffer.write_header(Header::UserReliableOrdered1);

        for client in self.clients.iter_mut() {
            buffer.seq(client.counter);
            client.counter += 1;
            let message = buffer.message();
            self.listener.send_to(&message, client.connection).unwrap();
        }
    }

    ///Create a new message and send it to an address' chat
    fn chat_to(&mut self, message: &str, socket: SocketAddr) {
        let mut buffer = Buffer::default();
        let email = MessageTypes::ChatMessage {
            from: "Server".to_owned(),
            r#type: ChatMessageType::System,
            text: message.to_owned(),
        };

        buffer.write_game_header(GameHeader::MatchMessage);
        buffer.write_time(&mut self.clock);

        buffer.write_json(email);
        buffer.write_header(Header::UserReliableOrdered1);

        self.listener.send_to(&buffer.message(), socket).unwrap();
    }

    ///Create a new message and send it to every chat
    fn chat_all(&mut self, message: &str) {
        let mut buffer = Buffer::default();
        let email = MessageTypes::ChatMessage {
            from: "Server".to_owned(),
            r#type: ChatMessageType::System,
            text: message.to_owned(),
        };

        buffer.write_game_header(GameHeader::MatchMessage);
        buffer.write_time(&mut self.clock);

        buffer.write_json(email);
        buffer.write_header(Header::UserReliableOrdered1);

        for client in self.clients.iter() {
            self.listener
                .send_to(&buffer.message(), client.connection)
                .unwrap();
        }
    }

    fn ack(&mut self) {
        let mut buffer = Buffer::default();

        buffer.write_byte(self.stream.header_byte);
        buffer.write_byte((self.stream.sequence & 0xFF).try_into().unwrap());
        buffer.write_byte((self.stream.sequence >> 8).try_into().unwrap());
        buffer.write_header(Header::Acknowledge);

        self.listener
            .send_to(&buffer.message(), self.stream.origin)
            .unwrap();
    }

    fn sending_client(&mut self) -> Option<usize> {
        self.clients
            .iter()
            .position(|e| e.connection == self.stream.origin)
    }

    fn current_client(&mut self, guid: Vec<u8>) -> Option<usize> {
        self.clients.iter().position(|e| e.guid == guid)
    }

    fn current_player(&mut self, guid: Vec<u8>, control: &i32) -> Option<usize> {
        self.players
            .iter()
            .position(|e| e.guid == guid && &e.ctrl_type == control)
    }

    fn verify_player(&mut self, _player: Player) -> bool {
        true
    }

    // ? Start of Logic handling
    // ? If you read this from top to bottom you should get a pretty good grasp of what's going on

    ///Start a new Server
    pub fn new(config: ServerConfig, matches: MatchConfig, motd: Motd) -> Server {
        println!("Hello there welcome your stay");
        println!("Starting server on ip: [129.0.0.1], port: [7878]");

        Server {
            app_id: "Sanicball",
            listener: UdpSocket::bind("0.0.0.0:7878").unwrap(),
            config,
            match_settings: matches,
            motd,
            clock: Clock::default(),
            clients: vec![],
            players: vec![],
            buffer: Buffer::default(),
            stream: Stream::default(),
        }
    }

    ///Update the Server's incoming requests
    pub fn update(&mut self) {
        self.timers();
        let mut buffer = [0; 1500];
        let raw = self.listener.recv_from(&mut buffer);

        if let Ok((size, addr)) = raw {
            self.stream = Stream::new(&buffer, size, addr);
            self.buffer = Buffer::default();

            let header: Header = match self.stream.header {
                Ok(header) => match header {
                    // ! This is not correct
                    Header::Acknowledge => Header::Unconnected,
                    //* Correct as far as I can tell
                    Header::Ping => {
                        let ping_number = self.stream.read_byte();

                        self.buffer.write_byte(ping_number);
                        self.buffer.write_time(&mut self.clock);

                        //Add the header
                        self.buffer.write_header(Header::Pong);
                        let message = self.buffer.message();
                        self.listener.send_to(&message, self.stream.origin).unwrap();

                        Header::Unconnected
                    }
                    //* Correct as far as I can tell
                    Header::Connect => {
                        //Initialize App ID on connect
                        let _app_id = self.stream.read_string();
                        self.buffer.write_string(self.app_id);

                        //TODO Figure out what this does
                        // Don't ask because I don't know
                        self.stream.read_f32();
                        self.stream.read_f32();
                        self.stream.read_f32();

                        //TODO Check version is valid

                        Header::ConnectResponse
                    }
                    //* Correct as far as I can tell
                    Header::ConnectionEstablished => {
                        //TODO Figure out what this does
                        self.stream.read_f32();

                        self.buffer.write_game_header(GameHeader::InitMessage);

                        self.buffer.write_clients(&self.clients);
                        self.buffer.write_players(&self.players);
                        self.buffer.write_settings(&self.match_settings);

                        //In race
                        self.buffer.write_bool(false);
                        //Cur auto start time
                        self.buffer.write_i32(&67);

                        self.send_to(Header::UserReliableOrdered1, addr);

                        Header::Unconnected
                    }
                    Header::UserReliableOrdered1 | Header::UserUnreliable => self.relay_data(),
                    _ => panic!(
                        "Tried to respond to a header that is not implamented {} (disable it)",
                        self.stream
                    ),
                },
                Err(_) => {
                    println!("Ignoring {}", self.stream);
                    Header::Unconnected
                }
            };

            match header {
                Header::Unconnected => {}
                Header::ConnectResponse => self.send_to(header, addr),
                _ => self.send_all(header),
            }
        }
    }

    ///Relay and react to a change in the game's state
    fn relay_data(&mut self) -> Header {
        self.ack();

        match self.stream.read_game_header() {
            GameHeader::MatchMessage => {
                let _time = self.stream.read_f32();
                let json = self.stream.read_string();

                self.buffer.write_game_header(GameHeader::MatchMessage);
                self.buffer.write_time(&mut self.clock);
                self.buffer.write_string(&json);

                let json = oxidize(json);
                self.update_server_state(&json);

                match self.sending_client() {
                    Some(index) => {
                        self.clients[index].counter = usize::from(self.stream.sequence) + 1
                    }
                    None => println!("Client is joining, sequence ignored"),
                }

                Header::UserReliableOrdered1
            }
            GameHeader::InitMessage => panic!(), // The game itself never sends this
            GameHeader::PlayerMovementMessage => {
                let _time = self.stream.read_f32();

                self.buffer
                    .write_game_header(GameHeader::PlayerMovementMessage);
                self.buffer.write_time(&mut self.clock);

                let data = self.stream.data.to_vec();
                for byte in data[5..].iter() {
                    self.buffer.write_byte(*byte);
                }

                //Add the header
                self.buffer.write_header(Header::UserUnreliable);

                //Return a small vector that send
                let guid = match self.sending_client() {
                    Some(index) => self.clients[index].guid.clone(),
                    None => vec![],
                };

                for client in self.clients.iter_mut() {
                    if client.guid == guid {
                        continue;
                    }
                    let message = self.buffer.message();
                    self.listener.send_to(&message, client.connection).unwrap();
                }

                Header::Unconnected
            }
        }
    }

    ///I don't think I need to explain why this isn't inlined
    fn update_server_state(&mut self, json: &str) {
        let message: MessageTypes = serde_json::from_str(json).unwrap();
        match message {
            MessageTypes::AutoStartTimerMessage { enabled } => todo!(),
            MessageTypes::ChangedReadyMessage {
                client_guid,
                ctrl_type,
                ready,
            } => {
                self.clock.lobby.start();
                let vecter = guid_to_vec(client_guid);
                match self.current_player(vecter, &ctrl_type) {
                    Some(index) => {
                        self.players[index].ready_to_race = ready;
                    }
                    None => println!("No such player"),
                }
            }
            MessageTypes::CharacterChangedMessage {
                client_guid,
                ctrl_type,
                new_character,
            } => {
                let vecter = guid_to_vec(client_guid);
                match self.current_player(vecter, &ctrl_type) {
                    Some(index) => {
                        // ! validate player
                        self.players[index].char_id = new_character;
                    }
                    None => println!("Player does not exist"),
                }
            }
            // TODO meme everyone into shrek
            MessageTypes::ChatMessage { from, r#type, text } => {}
            MessageTypes::CheckpointPassedMessage {
                client_guid,
                ctrl_type,
                lap_time,
            } => {}
            MessageTypes::ClientJoinedMessage {
                client_guid,
                client_name,
            } => {
                let socket = self.stream.origin;
                let vecter = guid_to_vec(client_guid);

                self.chat_all(&format!("{:?}, Has Joined The Match", client_name));
                self.chat_to("Welcome", socket);
                // ! valid characters
                self.chat_to(
                    &format!("Our Message of the day is {}", self.motd.text),
                    socket,
                );
                // ! Add support to send what characters are allowed

                self.clients.push(Client {
                    guid: vecter,
                    name: client_name,
                    connection: self.stream.origin,
                    is_loading: false,
                    wants_lobby: false,
                    counter: 0,
                });
            }
            MessageTypes::ClientLeftMessage { client_guid } => {}
            MessageTypes::DoneRacingMessage {
                client_guid,
                ctrl_type,
                race_time,
                disqualified,
            } => {
                let vecter = guid_to_vec(client_guid);
                match self.current_player(vecter, &ctrl_type) {
                    Some(index) => {
                        self.players[index].is_racing = false;
                        // ! reset race timeout
                        // ! kick everyone
                    }
                    None => println!("Who are you talking about"),
                }
            }
            MessageTypes::LoadLobbyMessage {} => {}
            MessageTypes::LoadRaceMessage {} => {}
            MessageTypes::PlayerJoinedMessage {
                client_guid,
                ctrl_type,
                initial_character,
            } => {
                let socket = self.stream.origin;

                let vecter = guid_to_vec(client_guid);
                match self.current_client(vecter.clone()) {
                    None => println!("A Player that is not a Client attempted to join"),
                    Some(_) => {
                        println!("New player");
                        // ! Verify player is valid
                        //self.chat_to("You can't join", socket);
                        self.players.push(Player {
                            guid: vecter,
                            ctrl_type,
                            char_id: initial_character,
                            ready_to_race: false,
                            is_racing: false,
                            race_timeout: Stopwatch {},
                            has_timed_out: false,
                        })
                    }
                }
            }
            MessageTypes::PlayerLeftMessage {
                client_guid,
                ctrl_type,
            } => {
                let vecter = guid_to_vec(client_guid);
                match self.current_player(vecter, &ctrl_type) {
                    Some(index) => {
                        self.players.remove(index);
                    }
                    None => println!("This guy didn't exist anyway"),
                }
            }
            MessageTypes::RaceFinishedMessage {
                client_guid,
                ctrl_type,
                race_time,
                race_position,
            } => {}
            MessageTypes::RaceTimeoutMessage {
                client_guid,
                ctrl_type,
                time,
            } => {}
            MessageTypes::SettingsChanged { new_match_settings } => {
                self.match_settings = new_match_settings;
            }
            MessageTypes::StartRaceMessage {} => {
                match self.sending_client() {
                    Some(index) => {
                        // ! smth smth timers
                        for player in &mut self.players {
                            player.is_racing = true;
                            //player.race_timeout
                        }
                    }
                    None => println!("What?"),
                }
            }
        }
    }

    fn timers(&mut self) {
        if self.clock.lobby.timeout(Duration::from_secs(3)) {
            self.clock.lobby.reset();
            self.clock.stage_load_timeout.start();

            self.send_new(MessageTypes::LoadRaceMessage {})
        }

        if self.clock.stage_load_timeout.timeout(Duration::from_secs(20)) {
            self.clock.stage_load_timeout.reset();

            self.send_new(MessageTypes::StartRaceMessage {  })
        }
    }
}

fn guid_to_vec(guid: String) -> Vec<u8> {
    let mut sum: Vec<u8> = Vec::with_capacity(16);

    let mut first: Vec<u8> = (0..8)
        .step_by(2)
        .map(|i| u8::from_str_radix(&guid[i..i + 2], 16).unwrap())
        .collect();
    first.reverse();

    let mut second: Vec<u8> = (9..13)
        .step_by(2)
        .map(|i| u8::from_str_radix(&guid[i..i + 2], 16).unwrap())
        .collect();
    second.reverse();

    let mut third: Vec<u8> = (14..18)
        .step_by(2)
        .map(|i| u8::from_str_radix(&guid[i..i + 2], 16).unwrap())
        .collect();
    third.reverse();

    let mut forth: Vec<u8> = (19..23)
        .step_by(2)
        .map(|i| u8::from_str_radix(&guid[i..i + 2], 16).unwrap())
        .collect();

    let mut fifth: Vec<u8> = (24..36)
        .step_by(2)
        .map(|i| u8::from_str_radix(&guid[i..i + 2], 16).unwrap())
        .collect();

    sum.append(&mut first);
    sum.append(&mut second);
    sum.append(&mut third);
    sum.append(&mut forth);
    sum.append(&mut fifth);

    println!("{guid}");
    sum
}
