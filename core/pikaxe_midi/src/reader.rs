use super::*;

#[derive(Clone, Copy, Debug)]
struct PendingMidiNote {
    pos: u64,
    channel: u8,
    velocity: u8
}

/*pub(crate) struct MidiReader {
    info: Option<MidiInfo>,
    current_track_index: i32,
    current_pos: u64,
    pending_notes: [Option<PendingMidiNote>; 0x80],
    current_track: Option<MidiTrack>,
    tracks: Vec<MidiTrack>,
    tempo_track: Vec<MidiTempo>,
}*/