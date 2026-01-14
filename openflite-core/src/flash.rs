use anyhow::{anyhow, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

/// Supported board types for flashing
#[derive(Debug, Clone, PartialEq)]
pub enum BoardType {
    ArduinoMega,
    ArduinoProMicro,
    ArduinoNano,
}

impl BoardType {
    pub fn avrdude_part(&self) -> &str {
        match self {
            BoardType::ArduinoMega => "atmega2560",
            BoardType::ArduinoProMicro => "atmega32u4",
            BoardType::ArduinoNano => "atmega328p",
        }
    }

    pub fn avrdude_programmer(&self) -> &str {
        match self {
            BoardType::ArduinoMega => "wiring",
            BoardType::ArduinoProMicro => "avr109",
            BoardType::ArduinoNano => "arduino",
        }
    }

    pub fn baud_rate(&self) -> u32 {
        match self {
            BoardType::ArduinoMega => 115200,
            BoardType::ArduinoProMicro => 57600,
            BoardType::ArduinoNano => 57600,
        }
    }

    pub fn firmware_name(&self) -> &str {
        match self {
            BoardType::ArduinoMega => "mobiflight_mega.hex",
            BoardType::ArduinoProMicro => "mobiflight_promicro.hex",
            BoardType::ArduinoNano => "mobiflight_nano.hex",
        }
    }
}

/// Flash firmware to an Arduino board using avrdude
pub fn flash_firmware(
    port: &str,
    board: BoardType,
    firmware_path: &str,
    progress_tx: Option<mpsc::Sender<u8>>,
) -> Result<()> {
    let args = vec![
        "-v".to_string(),
        "-p".to_string(),
        board.avrdude_part().to_string(),
        "-c".to_string(),
        board.avrdude_programmer().to_string(),
        "-P".to_string(),
        port.to_string(),
        "-b".to_string(),
        board.baud_rate().to_string(),
        "-D".to_string(),
        "-U".to_string(),
        format!("flash:w:{}:i", firmware_path),
    ];

    log::info!("Running avrdude with args: {:?}", args);

    let mut child = Command::new("avrdude")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to start avrdude: {}. Is avrdude installed?", e))?;

    // Parse stderr for progress (avrdude outputs progress there)
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let tx = progress_tx.clone();

        thread::spawn(move || {
            for line in reader.lines().map_while(Result::ok) {
                // Parse progress from avrdude output
                // Lines like "Writing | ################################################## | 100%"
                if line.contains('%') {
                    if let Some(pct_pos) = line.rfind('%') {
                        let start = line[..pct_pos]
                            .rfind(|c: char| !c.is_ascii_digit())
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        if let Ok(pct) = line[start..pct_pos].trim().parse::<u8>() {
                            if let Some(ref tx) = tx {
                                let _ = tx.send(pct);
                            }
                        }
                    }
                }
                log::debug!("avrdude: {}", line);
            }
        });
    }

    let status = child.wait()?;
    if status.success() {
        if let Some(tx) = progress_tx {
            let _ = tx.send(100);
        }
        Ok(())
    } else {
        Err(anyhow!("avrdude exited with status: {}", status))
    }
}

/// Check if avrdude is available on the system
pub fn check_avrdude() -> bool {
    Command::new("avrdude")
        .arg("-?")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}
