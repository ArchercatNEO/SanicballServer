use serde::{Deserialize, Serialize};

use crate::data::MatchConfig;

#[derive(Debug, Deserialize, Serialize)]
pub enum ChatMessageType {
    System,
    User,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CtrlType {
    None = -1,
    Keyboard = 0,
    Joystick1 = 1,
    Joystick2 = 2,
    Joystick3 = 3,
    Joystick4 = 4,
}

impl From<i32> for CtrlType {
    fn from(value: i32) -> Self {
        match value {
            0 => CtrlType::Keyboard,
            1 => CtrlType::Joystick1,
            2 => CtrlType::Joystick2,
            3 => CtrlType::Joystick3,
            4 => CtrlType::Joystick4,
            _ => CtrlType::None,
        }
    }
}

pub enum GameHeader {
    MatchMessage = 0,
    InitMessage = 1,
    PlayerMovementMessage = 2,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum MessageTypes {
    #[serde(rename_all = "PascalCase")]
    AutoStartTimerMessage { enabled: bool },
    #[serde(rename_all = "PascalCase")]
    ChangedReadyMessage {
        client_guid: String,
        ctrl_type: i32,
        ready: bool,
    },
    #[serde(rename_all = "PascalCase")]
    CharacterChangedMessage {
        client_guid: String,
        ctrl_type: i32,
        new_character: i32,
    },
    #[serde(rename_all = "PascalCase")]
    ChatMessage {
        from: String,
        r#type: ChatMessageType,
        text: String,
    },
    #[serde(rename_all = "PascalCase")]
    CheckPointPassedMessage {
        client_guid: String,
        ctrl_type: i32,
        lap_time: f32,
    },
    #[serde(rename_all = "PascalCase")]
    ClientJoinedMessage {
        client_guid: String,
        client_name: String,
    },
    #[serde(rename_all = "PascalCase")]
    ClientLeftMessage { client_guid: String },
    #[serde(rename_all = "PascalCase")]
    DoneRacingMessage {
        client_guid: String,
        ctrl_type: i32,
        race_time: f64,
        disqualified: bool,
    },
    #[serde(rename_all = "PascalCase")]
    LoadLobbyMessage {},
    #[serde(rename_all = "PascalCase")]
    LoadRaceMessage {},
    #[serde(rename_all = "PascalCase")]
    PlayerJoinedMessage {
        client_guid: String,
        ctrl_type: i32,
        initial_character: i32,
    },
    #[serde(rename_all = "PascalCase")]
    PlayerLeftMessage { client_guid: String, ctrl_type: i32 },
    #[serde(rename_all = "PascalCase")]
    RaceFinishedMessage {
        client_guid: String,
        ctrl_type: i32,
        race_time: f32,
        race_position: i32,
    },
    #[serde(rename_all = "PascalCase")]
    RaceTimeoutMessage {
        client_guid: String,
        ctrl_type: i32,
        time: f32,
    },
    #[serde(rename_all = "PascalCase")]
    SettingsChanged { new_match_settings: MatchConfig },
    #[serde(rename_all = "PascalCase")]
    StartRaceMessage {},
}

impl MessageTypes {
    pub fn as_string(&self) -> &'static str {
        match self {
            MessageTypes::AutoStartTimerMessage { enabled: _ } => "AutoStartTimerMessage",
            MessageTypes::ChangedReadyMessage {
                client_guid: _,
                ctrl_type: _,
                ready: _,
            } => "ChangedReadyMessage",
            MessageTypes::CharacterChangedMessage {
                client_guid: _,
                ctrl_type: _,
                new_character: _,
            } => "CharacterChangedMessage",
            MessageTypes::ChatMessage {
                from: _,
                r#type: _,
                text: _,
            } => "ChatMessage",
            MessageTypes::CheckPointPassedMessage {
                client_guid: _,
                ctrl_type: _,
                lap_time: _,
            } => "CheckPointPassedMessage",
            MessageTypes::ClientJoinedMessage {
                client_guid: _,
                client_name: _,
            } => "ClientJoinedMessage",
            MessageTypes::ClientLeftMessage { client_guid: _ } => "ClientLeftMessage",
            MessageTypes::DoneRacingMessage {
                client_guid: _,
                ctrl_type: _,
                race_time: _,
                disqualified: _,
            } => "DoneRacingMessage",
            MessageTypes::LoadLobbyMessage {} => "LoadLobbyMessage",
            MessageTypes::LoadRaceMessage {} => "LoadRaceMessage",
            MessageTypes::PlayerJoinedMessage {
                client_guid: _,
                ctrl_type: _,
                initial_character: _,
            } => "PlayerJoinedMessage",
            MessageTypes::PlayerLeftMessage {
                client_guid: _,
                ctrl_type: _,
            } => "PlayerLeftMessage",
            MessageTypes::RaceFinishedMessage {
                client_guid: _,
                ctrl_type: _,
                race_time: _,
                race_position: _,
            } => "RaceFinishedMessage",
            MessageTypes::RaceTimeoutMessage {
                client_guid: _,
                ctrl_type: _,
                time: _,
            } => "RaceTimeoutMessage",
            MessageTypes::SettingsChanged {
                new_match_settings: _,
            } => "SettingsChanged",
            MessageTypes::StartRaceMessage {} => "StartRaceMessage",
        }
    }
}
