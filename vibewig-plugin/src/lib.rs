use nih_plug::prelude::*;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use crossbeam_channel::{bounded, Receiver, Sender};
use tungstenite::accept;

/// A single note in the pattern
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub pitch: u8,       // MIDI note number
    pub duration: f64,   // Duration in beats
    pub velocity: u8,    // 0-127
}

/// Messages from WebSocket client to plugin
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

/// Messages from plugin to WebSocket client
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

/// The looping pattern state
#[derive(Default)]
struct PatternState {
    notes: Vec<Note>,
    current_index: usize,
    note_start_beat: f64,
    note_active: bool,
    active_pitch: u8,
}

pub struct VibewigPlugin {
    params: Arc<VibewigParams>,
    pattern: Arc<Mutex<PatternState>>,
    message_rx: Receiver<ClientMessage>,
    _message_tx: Sender<ClientMessage>, // Keep sender alive
    last_playing: bool,
}

#[derive(Params)]
struct VibewigParams {
    #[id = "port"]
    pub port: IntParam,
}

impl Default for VibewigPlugin {
    fn default() -> Self {
        let (tx, rx) = bounded(64);
        Self {
            params: Arc::new(VibewigParams::default()),
            pattern: Arc::new(Mutex::new(PatternState::default())),
            message_rx: rx,
            _message_tx: tx,
            last_playing: false,
        }
    }
}

impl Default for VibewigParams {
    fn default() -> Self {
        Self {
            port: IntParam::new("WebSocket Port", 9001, IntRange::Linear { min: 9001, max: 9010 }),
        }
    }
}

impl Plugin for VibewigPlugin {
    const NAME: &'static str = "Vibewig";
    const VENDOR: &'static str = "vibewig";
    const URL: &'static str = "https://github.com/vibewig/vibewig";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::Basic;

    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Start WebSocket server thread
        let port = self.params.port.value() as u16;
        let (tx, rx) = bounded(64);
        self.message_rx = rx;

        // Clone for the thread
        let tx_clone = tx.clone();
        self._message_tx = tx;

        thread::spawn(move || {
            let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Vibewig: Failed to bind to port {}: {}", port, e);
                    return;
                }
            };

            eprintln!("Vibewig: WebSocket server listening on port {}", port);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let tx = tx_clone.clone();
                        thread::spawn(move || {
                            if let Ok(mut websocket) = accept(stream) {
                                eprintln!("Vibewig: Client connected");
                                while let Ok(msg) = websocket.read() {
                                    if msg.is_text() {
                                        let text = msg.to_text().unwrap_or("");
                                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(text) {
                                            let _ = tx.try_send(client_msg);
                                        }
                                    }
                                }
                                eprintln!("Vibewig: Client disconnected");
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Vibewig: Connection error: {}", e);
                    }
                }
            }
        });

        true
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Process incoming messages
        while let Ok(msg) = self.message_rx.try_recv() {
            let mut pattern = self.pattern.lock();
            match msg {
                ClientMessage::SetPattern { notes, durations, velocities } => {
                    pattern.notes = notes.iter()
                        .zip(durations.iter())
                        .zip(velocities.iter())
                        .map(|((&pitch, &duration), &velocity)| Note { pitch, duration, velocity })
                        .collect();
                    pattern.current_index = 0;
                    pattern.note_start_beat = 0.0;
                    pattern.note_active = false;
                }
                ClientMessage::Clear => {
                    pattern.notes.clear();
                    pattern.current_index = 0;
                    pattern.note_active = false;
                }
            }
        }

        let transport = context.transport();
        let playing = transport.playing;
        let pos_beats = transport.pos_beats().unwrap_or(0.0);

        // Handle play state changes
        if playing && !self.last_playing {
            // Just started playing - reset pattern
            let mut pattern = self.pattern.lock();
            pattern.current_index = 0;
            pattern.note_start_beat = pos_beats;
            pattern.note_active = false;
        }
        self.last_playing = playing;

        if !playing {
            // Send note off if we have an active note
            let mut pattern = self.pattern.lock();
            if pattern.note_active {
                context.send_event(NoteEvent::NoteOff {
                    timing: 0,
                    voice_id: None,
                    channel: 0,
                    note: pattern.active_pitch,
                    velocity: 0.0,
                });
                pattern.note_active = false;
            }
            return ProcessStatus::Normal;
        }

        // Process pattern
        let mut pattern = self.pattern.lock();
        if pattern.notes.is_empty() {
            return ProcessStatus::Normal;
        }

        let current_note = &pattern.notes[pattern.current_index];
        let note_end_beat = pattern.note_start_beat + current_note.duration;

        // Check if current note should end
        if pos_beats >= note_end_beat {
            // Note off
            if pattern.note_active {
                context.send_event(NoteEvent::NoteOff {
                    timing: 0,
                    voice_id: None,
                    channel: 0,
                    note: pattern.active_pitch,
                    velocity: 0.0,
                });
                pattern.note_active = false;
            }

            // Move to next note (loop)
            pattern.current_index = (pattern.current_index + 1) % pattern.notes.len();
            pattern.note_start_beat = note_end_beat;
        }

        // Check if we need to start a new note
        if !pattern.note_active && pos_beats >= pattern.note_start_beat {
            let note = &pattern.notes[pattern.current_index];
            context.send_event(NoteEvent::NoteOn {
                timing: 0,
                voice_id: None,
                channel: 0,
                note: note.pitch,
                velocity: note.velocity as f32 / 127.0,
            });
            pattern.active_pitch = note.pitch;
            pattern.note_active = true;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for VibewigPlugin {
    const CLAP_ID: &'static str = "com.vibewig.vibewig";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Claude Code controlled MIDI looper");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::NoteEffect,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for VibewigPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"VibewigPlugin___";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Tools,
    ];
}

nih_export_clap!(VibewigPlugin);
nih_export_vst3!(VibewigPlugin);
