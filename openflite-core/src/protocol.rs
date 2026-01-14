#[derive(Debug, Clone)]
pub enum Command {
    Init,
    GetInfo,
    GetName,
    SetName(String),
    GetVersion,
    ResetBoard,
    SetPin(u8, u8),              // pin, value
    Set7Segment(u8, u8, String), // module, index, value
    SetLCD(u8, u8, String),      // display_id, line, text
    SetStepper(u8, i32),         // motor_id, steps (negative = reverse)
    SetRGB(u8, u8, u8, u8),      // led_id, r, g, b
}

impl Command {
    pub fn id(&self) -> u8 {
        match self {
            Command::Init => 1,
            Command::ResetBoard => 5,
            Command::GetInfo => 7,
            Command::GetName => 8,
            Command::SetName(_) => 9,
            Command::GetVersion => 10,
            Command::SetPin(_, _) => 3,
            Command::Set7Segment(_, _, _) => 15,
            Command::SetLCD(_, _, _) => 16,
            Command::SetStepper(_, _) => 17,
            Command::SetRGB(_, _, _, _) => 18,
        }
    }

    pub fn serialize(&self) -> String {
        let id = self.id();
        match self {
            Command::SetName(name) => format!("{},{};", id, name),
            Command::SetPin(pin, val) => format!("{},{},{};", id, pin, val),
            Command::Set7Segment(module, index, val) => {
                format!("{},{},{},{};", id, module, index, val)
            }
            Command::SetLCD(display_id, line, text) => {
                format!("{},{},{},{};", id, display_id, line, text)
            }
            Command::SetStepper(motor_id, steps) => {
                format!("{},{},{};", id, motor_id, steps)
            }
            Command::SetRGB(led_id, r, g, b) => {
                format!("{},{},{},{},{};", id, led_id, r, g, b)
            }
            _ => format!("{};", id),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Response {
    Info {
        name: String,
        board_type: String,
        serial: String,
        version: String,
    },
    Data(Vec<String>),
    InputEvent {
        name: String,
        value: String,
    },
    Unknown(u8, Vec<String>),
}

impl Response {
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim_end_matches(';').trim();
        let parts: Vec<&str> = input.split(',').collect();
        if parts.is_empty() {
            return None;
        }

        let id: u8 = parts[0].parse().ok()?;
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        match id {
            7 if args.len() >= 4 => Some(Response::Info {
                name: args[0].clone(),
                board_type: args[1].clone(),
                serial: args[2].clone(),
                version: args[3].clone(),
            }),
            11 if args.len() >= 2 => Some(Response::InputEvent {
                name: args[0].clone(),
                value: args[1].clone(),
            }),
            _ => Some(Response::Unknown(id, args)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization() {
        assert_eq!(Command::GetInfo.serialize(), "7;");
        assert_eq!(Command::SetName("Test".to_string()).serialize(), "9,Test;");
        assert_eq!(Command::SetPin(13, 1).serialize(), "3,13,1;");
    }

    #[test]
    fn test_response_parsing() {
        let input = "7,MyBoard,Mega,12345,1.0.0;";
        if let Some(Response::Info {
            name,
            board_type,
            serial,
            version,
        }) = Response::parse(input)
        {
            assert_eq!(name, "MyBoard");
            assert_eq!(board_type, "Mega");
            assert_eq!(serial, "12345");
            assert_eq!(version, "1.0.0");
        } else {
            panic!("Failed to parse info response");
        }
    }
}
