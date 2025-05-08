mod io;
mod reader;

pub use self::io::*;
pub use self::reader::*;
use std::cmp::Ordering;
use std::collections::HashMap;

const MPQ_120BPM: u32 = 60_000_000 / 120;

pub struct MidiFile {
    pub format: u16,
    pub ticks_per_quarter: u16,
    pub tracks: Vec<MidiTrack>,
    pub tempo: Vec<MidiTempo>
}

impl Default for MidiFile {
    fn default() -> Self {
        Self {
            format: 1,
            ticks_per_quarter: 480,
            tracks: Vec::new(),
            tempo: Vec::new()
        }
    }
}

#[derive(Clone, Debug)]
pub struct MidiTempo {
    pub pos: u64,
    pub pos_realtime: Option<f64>, // Milliseconds
    pub mpq: u32,
}

impl MidiTempo {
    pub fn get_bpm(&self) -> f64 {
        60_000_000.0 / (self.mpq as f64)
    }
}

pub struct MidiTrack {
    pub name: Option<String>, // Matches meta track name
    pub events: Vec<MidiEvent>,
}

impl MidiTrack {
    pub fn is_sorted(&self) -> bool {
        let events = &self.events;

        if events.len() <= 1 {
            // Congrats, nothing to sort
            return true;
        }

        let mut prev_pos = events
            .first()
            .map(|ev| ev.get_pos())
            .unwrap(); // Shouldn't panic

        for ev in events.iter().skip(1) {
            let ev_pos = ev.get_pos();

            if prev_pos > ev_pos {
                return false;
            }

            prev_pos = ev_pos;
        }

        true
    }

    pub fn sort(&mut self) {
        self.events.sort_by(|a, b| {
            let a_pos = a.get_pos();
            let b_pos = b.get_pos();

            if a_pos != b_pos {
                a_pos.cmp(&b_pos)
            } else {
                // Positions are equal, compare by other means
                a.partial_cmp(&b).unwrap_or(Ordering::Equal)
            }
        });
    }
}

pub struct MidiInfo {
    pub format: u16,
    pub ticks_per_quarter: u16, // Usually 480
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum MidiEvent {
    Note(MidiNote),
    Meta(MidiText),
    SysEx(MidiSysex)
}

impl MidiEvent {
    pub fn get_pos(&self) -> u64 {
        match self {
            MidiEvent::Note(note) => note.pos,
            MidiEvent::Meta(meta) => meta.pos,
            MidiEvent::SysEx(sys) => sys.pos
        }
    }

    pub fn get_pos_realtime(&self) -> Option<f64> {
        match self {
            MidiEvent::Note(note) => note.pos_realtime,
            MidiEvent::Meta(meta) => meta.pos_realtime,
            MidiEvent::SysEx(sys) => sys.pos_realtime
        }
    }

    pub fn get_length(&self) -> u64 {
        match self {
            MidiEvent::Note(note) => note.length,
            MidiEvent::Meta(_) => 0,
            MidiEvent::SysEx(_) => 0
        }
    }

    pub fn get_length_realtime(&self) -> Option<f64> {
        match self {
            MidiEvent::Note(note) => note.length_realtime,
            MidiEvent::Meta(_) => None,
            MidiEvent::SysEx(_) => None
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct MidiNote {
    pub pos: u64,
    pub pos_realtime: Option<f64>, // Milliseconds
    pub length: u64,
    pub length_realtime: Option<f64>, // Milliseconds
    pub pitch: u8,
    pub channel: u8,
    pub velocity: u8
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct MidiText {
    pub pos: u64,
    pub pos_realtime: Option<f64>, // Milliseconds
    pub text: MidiTextType,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MidiTextType {
    Event(Box<[u8]>),
    Lyric(Box<[u8]>)
}

impl Default for MidiTextType {
    fn default() -> MidiTextType {
        MidiTextType::Event(Default::default())
    }
}

impl MidiText {
    pub fn is_lyric(&self) -> bool {
        match self.text {
            MidiTextType::Lyric(_) => true,
            _ => false,
        }
    }

    pub fn is_text(&self) -> bool {
        match self.text {
            MidiTextType::Event(_) => true,
            _ => false,
        }
    }

    pub fn as_str<'a>(&'a self) -> Option<&'a str> {
        match &self.text {
            MidiTextType::Lyric(text) => std::str::from_utf8(text).ok(),
            MidiTextType::Event(text) => std::str::from_utf8(text).ok(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct MidiSysex {
    pub pos: u64,
    pub pos_realtime: Option<f64>, // Milliseconds
    pub data: Box<[u8]>
}

impl MidiFile {
    pub fn calculate_realtime_pos(&mut self) {
        self.calculate_tempo_realtime();
        self.calculate_tracks_realtime();
    }

    fn calculate_tempo_realtime(&mut self) {
        let tpq = self.ticks_per_quarter;

        let mut current_pos = 0;
        let mut current_pos_realtime = 0.0;
        let mut current_mpq = MPQ_120BPM;

        for tempo in self.tempo.iter_mut() {
            let delta_ticks = tempo.pos - current_pos;
            let delta_realtime = (current_mpq as u64 * delta_ticks) as f64 / (1_000 * tpq as u32) as f64;

            current_pos_realtime = current_pos_realtime + delta_realtime;
            tempo.pos_realtime = Some(current_pos_realtime);

            // Update current
            current_pos = tempo.pos;
            current_mpq = tempo.mpq;
        }
    }

    fn calculate_tracks_realtime(&mut self) {
        let mut tempo_nav = TempoNavigator::new(&self.tempo, self.ticks_per_quarter);

        for ev in self.tracks.iter_mut().flat_map(|t| &mut t.events) {
            match ev {
                MidiEvent::Note(MidiNote { pos, pos_realtime, length, length_realtime, pitch: _, channel: _, velocity: _ }) => {
                    let pos_start_realtime = tempo_nav.get_realtime_position(*pos);

                    // Calculate length
                    let pos_end = *pos + *length;
                    let pos_end_realtime = tempo_nav.get_realtime_position(pos_end);
                    let realtime_length = pos_end_realtime - pos_start_realtime;

                    *pos_realtime = Some(pos_start_realtime);
                    *length_realtime = Some(realtime_length);
                },
                MidiEvent::Meta(MidiText { pos, pos_realtime, text: _ }) => {
                    *pos_realtime = Some(tempo_nav.get_realtime_position(*pos));
                },
                MidiEvent::SysEx(MidiSysex { pos, pos_realtime, data: _ }) => {
                    *pos_realtime = Some(tempo_nav.get_realtime_position(*pos));
                }
            }
        }
    }

    pub fn add_tracks_with_realtime_positions(&mut self, mut tracks: Vec<MidiTrack>, merge_same_tracks: bool) {
        self.calculate_tempo_realtime(); // Just in case

        let mut tempo_nav = TempoNavigator::new(&self.tempo, self.ticks_per_quarter);

        // Calculate tick position for every note
        for ev in tracks.iter_mut().flat_map(|t| &mut t.events) {
            match ev {
                MidiEvent::Note(MidiNote { pos, pos_realtime, length, length_realtime, pitch: _, channel: _, velocity: _ }) => {
                    let pos_start_ticks = pos_realtime.map(|p| tempo_nav.get_tick_position(p)).unwrap_or_default();

                    // Calculate length
                    let pos_end = pos_realtime.unwrap_or_default() + length_realtime.unwrap_or_default();
                    let pos_end_ticks = tempo_nav.get_tick_position(pos_end);
                    let tick_length = pos_end_ticks - pos_start_ticks;

                    *pos = pos_start_ticks;
                    *length = tick_length;
                },
                MidiEvent::Meta(MidiText { pos, pos_realtime, text: _ }) => {
                    *pos = pos_realtime.map(|p| tempo_nav.get_tick_position(p)).unwrap_or_default();
                },
                MidiEvent::SysEx(MidiSysex { pos, pos_realtime, data: _ }) => {
                    *pos = pos_realtime.map(|p| tempo_nav.get_tick_position(p)).unwrap_or_default();
                }
            }
        }

        // Map track name to track collection index
        let track_name_map = self
            .tracks
            .iter()
            .enumerate()
            .fold(HashMap::new(), |mut acc, (i, track)| {
                if let Some(track_name) = track.name.as_ref() {
                    acc.insert(track_name.to_owned(), i);
                }

                acc
            });

        for mut track in tracks {
            // Merge track events if existing found and option enabled
            if merge_same_tracks && track.name.is_some() {
                let existing_track = track_name_map
                    .get(track.name.as_ref().unwrap())
                    .map(|i| &mut self.tracks[*i]);

                if let Some(existing_track) = existing_track {
                    existing_track.events.append(&mut track.events);
                    existing_track.sort();
                    continue;
                }
            }

            // Just add track
            track.sort();
            self.tracks.push(track);
        }
    }

    pub fn get_track_with_name<'a>(&'a self, name: &str) -> Option<&'a MidiTrack> {
        self.tracks
            .iter()
            .find(|t| t
                .name
                .as_ref()
                .map(|n| n.eq(name))
                .unwrap_or_default())
    }

    pub fn get_track_with_name_mut<'a>(&'a mut self, name: &str) -> Option<&'a mut MidiTrack> {
        self.tracks
            .iter_mut()
            .find(|t| t
                .name
                .as_ref()
                .map(|n| n.eq(name))
                .unwrap_or_default())
    }
}

struct TempoNavigator<'a> {
    index: usize,
    tpq: u16,
    tempo: &'a [MidiTempo]
}

impl<'a> TempoNavigator<'a> {
    fn new(tempo: &'a [MidiTempo], tpq: u16) -> Self {
        TempoNavigator {
            index: 0,
            tpq,
            tempo
        }
    }

    fn get_tempo_at_pos(&mut self, pos: u64) -> Option<&'a MidiTempo> {
        if self.tempo.is_empty() {
            return None;
        }

        let mut current_tempo = self.get_current_tempo().unwrap();

        if current_tempo.pos > pos {
            // Find first tempo pos less than or equal to input pos
            while let Some(prev_tempo) = self.get_prev_tempo() {
                self.index -= 1;
                current_tempo = prev_tempo;

                if current_tempo.pos <= pos {
                    return Some(current_tempo);
                }
            }

            // Not found
            return None;
        }

        // Stop when next tempo pos is greater than input pos or end reached
        while let Some(next_tempo) = self.get_next_tempo() {
            if current_tempo.pos == pos || next_tempo.pos > pos {
                break;
            }

            self.index += 1;
            current_tempo = next_tempo;
        }

        return Some(current_tempo);
    }

    fn get_tempo_at_pos_realtime(&mut self, pos: f64) -> Option<&'a MidiTempo> {
        if self.tempo.is_empty() {
            return None;
        }

        let mut current_tempo = self.get_current_tempo().unwrap();
        let mut curr_pos_realtime = current_tempo.pos_realtime.unwrap();

        if curr_pos_realtime > pos {
            // Find first tempo pos less than or equal to input pos
            while let Some(prev_tempo) = self.get_prev_tempo() {
                self.index -= 1;
                current_tempo = prev_tempo;
                curr_pos_realtime = prev_tempo.pos_realtime.unwrap();

                if curr_pos_realtime <= pos {
                    return Some(current_tempo);
                }
            }

            // Not found
            return None;
        }

        // Stop when next tempo pos is greater than input pos or end reached
        while let Some(next_tempo) = self.get_next_tempo() {
            let next_pos_realtime = next_tempo.pos_realtime.unwrap();

            if curr_pos_realtime == pos || next_pos_realtime > pos {
                break;
            }

            self.index += 1;
            current_tempo = next_tempo;
            curr_pos_realtime = next_pos_realtime;
        }

        return Some(current_tempo);
    }

    fn get_current_tempo(&self) -> Option<&'a MidiTempo> {
        if self.tempo.is_empty() {
            return None;
        }

        Some(&self.tempo[self.index])
    }

    fn get_prev_tempo(&self) -> Option<&'a MidiTempo> {
        if self.tempo.is_empty() || self.index == 0 {
            return None;
        }

        let prev_index = self.index - 1;
        Some(&self.tempo[prev_index])
    }

    fn get_next_tempo(&self) -> Option<&'a MidiTempo> {
        if self.tempo.is_empty() {
            return None;
        }

        let next_index = self.index + 1;
        if next_index >= self.tempo.len() {
            return None;
        }

        Some(&self.tempo[next_index])
    }

    fn get_realtime_position(&mut self, pos: u64) -> f64 {
        let (tempo_pos, tempo_pos_realtime, mpq) = self
            .get_tempo_at_pos(pos)
            .map(|t| (t.pos, t.pos_realtime.unwrap(), t.mpq))
            .unwrap_or((0, 0., MPQ_120BPM));


        let delta_ticks = pos - tempo_pos;
        let delta_realtime = (mpq as u64 * delta_ticks) as f64 / (1_000 * self.tpq as u32) as f64;

        tempo_pos_realtime + delta_realtime
    }

    fn get_tick_position(&mut self, pos: f64) -> u64 {
        let (tempo_pos, tempo_pos_realtime, mpq) = self
            .get_tempo_at_pos_realtime(pos)
            .map(|t| (t.pos, t.pos_realtime.unwrap(), t.mpq))
            .unwrap_or((0, 0., MPQ_120BPM));

        let delta_realtime = pos - tempo_pos_realtime;
        let delta_ticks = (1000. * self.tpq as f64 * delta_realtime) as u64 / mpq as u64;

        tempo_pos + delta_ticks
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use super::*;

    const TICKS_PER_QUARTER: u16 = 480;

    #[rstest]
    #[case([(0, 120.), (2, 120.), (4, 120.)], [0., 1000., 2000.])]
    #[case([(4, 160.)], [2000.])]
    #[case([(4, 160.), (8, 200.)], [2000., 3500.])]
    fn calc_realtime_tempo_positions<const N: usize>(#[case] input_tempo: [(u64, f64); N], #[case] expected_pos: [f64; N]) {
        let mut mid = MidiFile {
            ticks_per_quarter: TICKS_PER_QUARTER,
            tempo: input_tempo.iter().map(|(beat, bpm)| MidiTempo {
                pos: beat * (TICKS_PER_QUARTER as u64),
                pos_realtime: None,
                mpq: (60_000_000. / bpm).ceil() as u32
            }).collect(),
            ..Default::default()
        };

        mid.calculate_tempo_realtime();

        for (i, tempo) in mid.tempo.iter().enumerate() {
            assert_eq!(Some(expected_pos[i]), tempo.pos_realtime);
        }
    }

    #[rstest]
    #[case([], [(2, 1)], [(1000., 500.)])]
    #[case([(4, 200.)], [(2, 4)], [(1000., 1600.)])] // Note overlaps tempo change
    #[case([(4, 200.)], [(0, 8), (2, 4), (2,1)], [(0., 3200.), (1000., 1600.), (1000., 500.)])]
    #[case([(4, 240.)], [(8, 1)], [(3000., 250.)])]
    fn calc_realtime_note_positions<const N: usize, const M: usize>(#[case] input_tempo: [(u64, f64); N], #[case] input_notes: [(u64, u64); M], #[case] expected: [(f64, f64); M]) {
        let mut mid = MidiFile {
            ticks_per_quarter: TICKS_PER_QUARTER,
            tempo: input_tempo.iter().map(|(beat, bpm)| MidiTempo {
                pos: beat * (TICKS_PER_QUARTER as u64),
                pos_realtime: None,
                mpq: (60_000_000. / bpm).ceil() as u32
            }).collect(),
            tracks: vec![
                MidiTrack {
                    name: None,
                    events: input_notes.iter().map(|(beat_pos, beat_len)| MidiEvent::Note(MidiNote {
                        pos: beat_pos * (TICKS_PER_QUARTER as u64),
                        length: beat_len * (TICKS_PER_QUARTER as u64),
                        ..Default::default()
                    })).collect()
                }
            ],
            ..Default::default()
        };

        mid.calculate_tempo_realtime();
        mid.calculate_tracks_realtime();

        for (i, ev) in mid.tracks.iter().flat_map(|t| &t.events).enumerate() {
            let (expected_pos, expected_length) = expected[i];

            let actual_pos = ev.get_pos_realtime();
            let actual_length = ev.get_length_realtime();

            assert_eq!(Some(expected_pos), actual_pos);
            assert_eq!(Some(expected_length), actual_length);
        }
    }

    #[rstest]
    #[case([], [0.], [0])]
    #[case([], [500.], [1 * TICKS_PER_QUARTER as u64])]
    #[case([(0, 240.)], [250.], [1 * TICKS_PER_QUARTER as u64])]
    #[case([(4, 240.)], [500., 2000., 3000.], [1 * TICKS_PER_QUARTER as u64, 4 * TICKS_PER_QUARTER as u64, 8 * TICKS_PER_QUARTER as u64])]
    #[case([(4, 240.)], [500., 2000., 1000.], [1 * TICKS_PER_QUARTER as u64, 2 * TICKS_PER_QUARTER as u64, 4 * TICKS_PER_QUARTER as u64])] // Events are re-sorted by position
    fn calc_realtime_to_ticks<const N: usize, const M: usize>(#[case] input_tempo: [(u64, f64); N], #[case] input_notes: [f64; M], #[case] expected: [u64; M]) {
        let mut mid = MidiFile {
            ticks_per_quarter: TICKS_PER_QUARTER,
            tempo: input_tempo.iter().map(|(beat, bpm)| MidiTempo {
                pos: beat * (TICKS_PER_QUARTER as u64),
                pos_realtime: None,
                mpq: (60_000_000. / bpm).ceil() as u32
            }).collect(),
            tracks: Vec::new(),
            ..Default::default()
        };
        mid.calculate_tempo_realtime();

        let realtime_tracks = vec![
            MidiTrack {
                name: None,
                events: input_notes.iter().map(|real_pos| MidiEvent::Meta(MidiText {
                    pos_realtime: Some(*real_pos),
                    ..Default::default()
                })).collect()
            }
        ];

        // Add notes
        mid.add_tracks_with_realtime_positions(realtime_tracks, false);

        for (i, ev) in mid.tracks.iter().flat_map(|t| &t.events).enumerate() {
            let expected_pos = expected[i];
            let actual_pos = ev.get_pos();

            assert_eq!(expected_pos, actual_pos);
        }
    }

    #[rstest]
    #[case([], [0], [None])]
    #[case([(4, 120.)], [0], [None])]
    #[case([(4, 120.)], [0, 8], [None, Some(4)])]
    #[case([(4, 120.)], [0, 8, 0], [None, Some(4), None])]
    #[case([(2, 120.), (6, 120.)], [0, 4, 1, 8, 3], [None, Some(2), None, Some(6), Some(2)])]
    fn tempo_navigation_ticks<const N: usize, const M: usize>(#[case] input_tempo: [(u64, f64); N], #[case] input_pos: [u64; M], #[case] expected: [Option<u64>; M]) {
        let tempos = input_tempo
            .iter()
            .map(|(beat, bpm)| MidiTempo {
                pos: beat * (TICKS_PER_QUARTER as u64),
                pos_realtime: None,
                mpq: (60_000_000. / bpm).ceil() as u32
            })
            .collect::<Vec<_>>();

        let mut tempo_nav = TempoNavigator::new(tempos.as_slice(), TICKS_PER_QUARTER);

        let results = input_pos
            .iter()
            .map(|p| tempo_nav.get_tempo_at_pos(*p * (TICKS_PER_QUARTER as u64)))
            .collect::<Vec<_>>();

        for (i, tempo) in results.iter().enumerate() {
            let expected_tempo_pos = expected[i];
            let actual_tempo_pos = tempo.map(|t| t.pos / (TICKS_PER_QUARTER as u64));

            assert_eq!(expected_tempo_pos, actual_tempo_pos);
        }
    }

    #[rstest]
    #[case([], [0.], [None])]
    #[case([(2000., 120.)], [0.], [None])]
    #[case([(2000., 120.)], [0., 4000., 1000.], [None, Some(2000.), None])]
    #[case([(2000., 200.), (6000., 120.)], [0., 4000., 1000., 8000., 3000.], [None, Some(2000.), None, Some(6000.), Some(2000.)])]
    fn tempo_navigation_realtime<const N: usize, const M: usize>(#[case] input_tempo: [(f64, f64); N], #[case] input_pos: [f64; M], #[case] expected: [Option<f64>; M]) {
        let tempos = input_tempo
            .iter()
            .map(|(real_pos, bpm)| MidiTempo {
                pos: 0, // Doesn't matter for test
                pos_realtime: Some(*real_pos),
                mpq: (60_000_000. / bpm).ceil() as u32
            })
            .collect::<Vec<_>>();

        let mut tempo_nav = TempoNavigator::new(tempos.as_slice(), TICKS_PER_QUARTER);

        let results = input_pos
            .iter()
            .map(|p| tempo_nav.get_tempo_at_pos_realtime(*p))
            .collect::<Vec<_>>();

        for (i, tempo) in results.iter().enumerate() {
            let expected_tempo_pos = expected[i];
            let actual_tempo_pos = tempo.and_then(|t| t.pos_realtime);

            assert_eq!(expected_tempo_pos, actual_tempo_pos);
        }
    }
}