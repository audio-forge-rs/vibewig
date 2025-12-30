use anyhow::{Context, Result};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// Messages sent to plugins
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "set_pattern")]
    SetPattern {
        notes: Vec<u8>,
        durations: Vec<f64>,
        velocities: Vec<u8>,
    },
    #[serde(rename = "clear")]
    Clear,
}

/// Messages received from plugins
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PluginMessage {
    #[serde(rename = "transport")]
    Transport {
        playing: bool,
        tempo: f64,
        time_sig_num: u32,
        time_sig_denom: u32,
        position_beats: f64,
    },
}

#[derive(Parser, Debug)]
#[command(name = "vibewig")]
#[command(about = "Control Vibewig plugins from Claude Code")]
struct Args {
    /// Port for track 1 plugin
    #[arg(long, default_value = "9001")]
    port1: u16,

    /// Port for track 2 plugin
    #[arg(long, default_value = "9002")]
    port2: u16,

    /// JSON command to send (if not provided, enters interactive mode)
    #[arg(short, long)]
    command: Option<String>,

    /// Target track (1 or 2, default sends to both)
    #[arg(short, long)]
    track: Option<u8>,
}

struct PluginConnection {
    write: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message,
    >,
}

impl PluginConnection {
    async fn send(&mut self, msg: &ClientMessage) -> Result<()> {
        let json = serde_json::to_string(msg)?;
        self.write.send(Message::Text(json)).await?;
        Ok(())
    }
}

async fn connect_to_plugin(port: u16) -> Result<PluginConnection> {
    let url = format!("ws://127.0.0.1:{}", port);
    let (ws_stream, _) = connect_async(&url)
        .await
        .with_context(|| format!("Failed to connect to plugin on port {}", port))?;

    let (write, _read) = ws_stream.split();

    Ok(PluginConnection { write })
}

async fn send_to_tracks(
    track1: &mut Option<PluginConnection>,
    track2: &mut Option<PluginConnection>,
    msg: &ClientMessage,
    target: Option<u8>,
) -> Result<()> {
    match target {
        Some(1) => {
            if let Some(ref mut conn) = track1 {
                conn.send(msg).await?;
                println!("Sent to track 1");
            } else {
                println!("Track 1 not connected");
            }
        }
        Some(2) => {
            if let Some(ref mut conn) = track2 {
                conn.send(msg).await?;
                println!("Sent to track 2");
            } else {
                println!("Track 2 not connected");
            }
        }
        _ => {
            // Send to both
            let mut sent = false;
            if let Some(ref mut conn) = track1 {
                conn.send(msg).await?;
                sent = true;
            }
            if let Some(ref mut conn) = track2 {
                conn.send(msg).await?;
                sent = true;
            }
            if sent {
                println!("Sent to all connected tracks");
            } else {
                println!("No tracks connected");
            }
        }
    }
    Ok(())
}

fn parse_command(input: &str) -> Result<(ClientMessage, Option<u8>)> {
    let input = input.trim();

    // Check for track prefix like "1:" or "2:"
    let (target, cmd) = if let Some(rest) = input.strip_prefix("1:") {
        (Some(1), rest.trim())
    } else if let Some(rest) = input.strip_prefix("2:") {
        (Some(2), rest.trim())
    } else {
        (None, input)
    };

    // Try parsing as JSON first
    if cmd.starts_with('{') {
        let msg: ClientMessage = serde_json::from_str(cmd)?;
        return Ok((msg, target));
    }

    // Simple command parsing
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    match parts.first().copied() {
        Some("clear") | Some("stop") => Ok((ClientMessage::Clear, target)),
        Some("play") | Some("notes") => {
            // Parse: play 60 62 64 65 [dur:0.25] [vel:100]
            let mut notes = Vec::new();
            let mut duration = 0.25;
            let mut velocity = 100u8;

            for part in &parts[1..] {
                if let Some(d) = part.strip_prefix("dur:") {
                    duration = d.parse()?;
                } else if let Some(v) = part.strip_prefix("vel:") {
                    velocity = v.parse()?;
                } else if let Ok(note) = part.parse::<u8>() {
                    notes.push(note);
                }
            }

            if notes.is_empty() {
                anyhow::bail!("No notes provided");
            }

            let durations = vec![duration; notes.len()];
            let velocities = vec![velocity; notes.len()];

            Ok((ClientMessage::SetPattern { notes, durations, velocities }, target))
        }
        _ => anyhow::bail!("Unknown command: {}", cmd),
    }
}

fn print_help() {
    println!("Vibewig Client Commands:");
    println!("  play <notes> [dur:X] [vel:X]  - Set pattern (e.g., 'play 60 62 64 dur:0.5')");
    println!("  clear / stop                   - Clear the pattern");
    println!("  {{\"type\":\"set_pattern\",...}}    - Send raw JSON");
    println!();
    println!("Prefix with '1:' or '2:' to target specific track:");
    println!("  1:play 60 64 67               - Send to track 1 only");
    println!("  2:clear                        - Clear track 2 only");
    println!();
    println!("  help                           - Show this help");
    println!("  quit / exit                    - Exit");
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Try to connect to plugins
    println!("Connecting to Vibewig plugins...");

    let mut track1 = match connect_to_plugin(args.port1).await {
        Ok(conn) => {
            println!("Connected to track 1 (port {})", args.port1);
            Some(conn)
        }
        Err(e) => {
            println!("Track 1 (port {}): {}", args.port1, e);
            None
        }
    };

    let mut track2 = match connect_to_plugin(args.port2).await {
        Ok(conn) => {
            println!("Connected to track 2 (port {})", args.port2);
            Some(conn)
        }
        Err(e) => {
            println!("Track 2 (port {}): {}", args.port2, e);
            None
        }
    };

    if track1.is_none() && track2.is_none() {
        anyhow::bail!("Could not connect to any plugins. Make sure Vibewig is loaded in Bitwig.");
    }

    // Single command mode
    if let Some(cmd) = args.command {
        let (msg, target) = parse_command(&cmd)?;
        send_to_tracks(&mut track1, &mut track2, &msg, target.or(args.track)).await?;
        return Ok(());
    }

    // Interactive mode
    println!();
    print_help();
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush()?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match line {
            "quit" | "exit" => break,
            "help" => print_help(),
            _ => {
                match parse_command(line) {
                    Ok((msg, target)) => {
                        if let Err(e) = send_to_tracks(&mut track1, &mut track2, &msg, target).await {
                            println!("Error: {}", e);
                        }
                    }
                    Err(e) => println!("Parse error: {}", e),
                }
            }
        }
    }

    println!("Goodbye!");
    Ok(())
}
