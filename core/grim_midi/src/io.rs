use midly::{Format as MidiFormat, Header as MidiHeader, MetaMessage, MidiMessage, Smf, Timing as MidiTiming, Track, TrackEvent, TrackEventKind};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs;
use std::path::Path;
use super::*;

const MAX_DELTA: u64 = 1 << 27;

impl MidiFile {
    pub fn from_path<T: AsRef<Path>>(path: T) -> Option<MidiFile> {
        let mid_file = fs::File::open(path).ok()?;
        Self::from_stream(mid_file)
    }

    pub fn from_stream<T: std::io::Read>(mut stream: T) -> Option<MidiFile> {
        // TODO: Use result w/ custom error
        let mut mid_bytes = Vec::new();
        stream.read_to_end(&mut mid_bytes).ok()?;
        let mut mid = MidiFile::default();

        // Load midi file using lib
        let smf = Smf::parse(&mid_bytes).ok()?;

        // TODO: Don't bother re-mapping and just re-export enum
        mid.format = match &smf.header.format {
            MidiFormat::SingleTrack => 0,
            MidiFormat::Parallel => 1,
            MidiFormat::Sequential => 2
        };

        mid.ticks_per_quarter = match &smf.header.timing {
            MidiTiming::Metrical(time) => time.as_int(),
            MidiTiming::Timecode(_, _) => panic!("\"Timecode\" not supported for reading in midi file"),
        };

        // Parse tempo track
        if let Some(tempo_track) = smf.tracks.get(0) {
            let mut abs_pos = 0;

            for ev in tempo_track.iter() {
                abs_pos += ev.delta.as_int() as u64;

                match ev.kind {
                    TrackEventKind::Meta(MetaMessage::Tempo(tempo)) => {
                        mid.tempo.push(MidiTempo {
                            pos: abs_pos,
                            pos_realtime: None,
                            mpq: tempo.as_int()
                        });
                    },
                    /*TrackEventKind::Meta(MetaMessage::TimeSignature(num, dem, clocks_per_click, notes_per_quarter_32)) => {
                        // TODO: Save time sig changes
                    },*/
                    _ => continue
                }
            }
        }

        let mut pending_notes: [Option<MidiNote>; 0x80] = [(); 0x80].map(|_| None);

        // TODO: Parse tempo track individually. Maybe use accumulator too.
        for track in smf.tracks.iter().skip(1) {
            let mut abs_pos = 0;
            let mut mid_track_events = Vec::new();
            let mut track_name = None;

            // TODO: Parse track name event
            for ev in track.iter() {
                abs_pos += ev.delta.as_int() as u64;

                match &ev.kind {
                    TrackEventKind::Meta(meta) => match meta {
                        MetaMessage::TrackName(raw_track_name) => {
                            // Parse track name and assign
                            if let Ok(name) = String::from_utf8(raw_track_name.to_vec()) {
                                track_name = Some(name);
                            }
                        },
                        MetaMessage::Lyric(lyric) => {
                            mid_track_events.push(MidiEvent::Meta(MidiText {
                                pos: abs_pos,
                                pos_realtime: None,
                                text: MidiTextType::Lyric(lyric.to_vec().into_boxed_slice())
                            }));
                        },
                        MetaMessage::Text(text) => {
                            mid_track_events.push(MidiEvent::Meta(MidiText {
                                pos: abs_pos,
                                pos_realtime: None,
                                text: MidiTextType::Event(text.to_vec().into_boxed_slice())
                            }));
                        },
                        _ => {}
                    },
                    TrackEventKind::Midi { channel, message: MidiMessage::NoteOn { key, vel } } => {
                        let index = key.as_int() as usize;

                        // If note exists, ignore
                        if let Some(mut note) = pending_notes[index].take() {
                            if vel.as_int() == 0 {
                                // Treat as off note
                                let length = abs_pos - note.pos;
                                if length == 0 {
                                    continue;
                                }

                                // Edit length and add note
                                note.length = length;
                                mid_track_events.push(MidiEvent::Note(note));
                                continue;
                            } else if note.velocity != vel.as_int() || note.channel != channel.as_int() {
                                // End existing note and create new

                                let length = abs_pos - note.pos;
                                if length > 0 {
                                    // Add existing note
                                    // Edit length and add note
                                    note.length = length;
                                    mid_track_events.push(MidiEvent::Note(note));
                                }
                            } else {
                                // Restore
                                pending_notes[index] = Some(note);
                                continue;
                            }
                        } else if vel.as_int() == 0 {
                            // Orphaned note off, ignore
                            continue;
                        }

                        // Otherwise add note
                        pending_notes[index] = Some(MidiNote {
                            pos: abs_pos,
                            length: 0,
                            pitch: key.as_int(),
                            channel: channel.as_int(),
                            velocity: vel.as_int(),
                            ..Default::default()
                        });
                    },
                    TrackEventKind::Midi { channel: _, message: MidiMessage::NoteOff { key, vel: _ } } => {
                        let index = key.as_int() as usize;

                        if let Some(mut note) = pending_notes[index].take() {
                            let length = abs_pos - note.pos;
                            if length == 0 {
                                continue;
                            }

                            // Edit length and add note
                            note.length = length;
                            mid_track_events.push(MidiEvent::Note(note));
                        }
                    },
                    TrackEventKind::SysEx(sysex) => {
                        mid_track_events.push(MidiEvent::SysEx(MidiSysex {
                            pos: abs_pos,
                            pos_realtime: None,
                            data: sysex.to_vec().into_boxed_slice()
                        }));
                    },
                    _ => {}
                }
            }

            // Add remaining notes
            for note in pending_notes.iter_mut() {
                if let Some(mut note) = note.take() {
                    let length = abs_pos - note.pos;
                    if length == 0 {
                        continue;
                    }

                    // Edit length and add note
                    note.length = length;
                    mid_track_events.push(MidiEvent::Note(note));
                }
            }

            // Sort events and add track
            let mut mid_track = MidiTrack {
                name: track_name,
                events: mid_track_events
            };

            mid_track.sort();
            mid.tracks.push(mid_track);
        }

        // Update realtime offsets
        mid.calculate_realtime_pos();

        Some(mid)
    }

    fn generate_tempo_track<'a>(&'a self) -> Vec<TrackEvent<'a>> {
        let mut tempo_track = vec![
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(midly::MetaMessage::TrackName(b"tempo"))
            }
        ];

        // Add change events
        let mut current_pos: u64 = 0;
        for tempo_ev in self.tempo.iter() {
            let delta = tempo_ev.pos - current_pos;

            // Super unlikely to occur. It comes out to about 279k beats at 480 resolution
            if delta > MAX_DELTA {
                panic!("Unsupported: Delta distance of {delta} between {} and {} is larger than max {MAX_DELTA}",
                    current_pos,
                    tempo_ev.pos
                );
            }

            tempo_track.push(TrackEvent {
                delta: (delta as u32).into(),
                kind: TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo_ev.mpq.into()))
            });

            current_pos = tempo_ev.pos;
        }

        // Add end of track event
        tempo_track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)
        });

        tempo_track
    }

    fn generate_track<'a>(&'a self, track_index: usize) -> Vec<TrackEvent<'a>> {
        let mut track = Vec::new();
        let input_track = &self.tracks[track_index];

        // TODO: Verify track is sorted before generating?

        // Add track name event
        if let Some(track_name) = input_track.name.as_ref() {
            track.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(midly::MetaMessage::TrackName(track_name.as_bytes()))
            });
        };

        let mut pending_off_notes: BinaryHeap<Reverse<(u64, u8, u8, u8)>> = BinaryHeap::new();

        // Add track events
        let mut prev_note_pos: u64 = 0;
        for ev in input_track.events.iter() {
            let ev_pos = ev.get_pos();

            // Process note off events
            while let Some(&Reverse((off_pos, off_pitch, off_channel, off_velocity))) = pending_off_notes.peek() {
                if off_pos > ev_pos {
                    break;
                }

                let off_delta = off_pos - prev_note_pos;
                prev_note_pos = off_pos;

                // Add note off event
                track.push(TrackEvent {
                    delta: (off_delta as u32).into(),
                    kind: TrackEventKind::Midi {
                        channel: off_channel.into(),
                        message: MidiMessage::NoteOff {
                            key: off_pitch.into(),
                            vel: off_velocity.into()
                        }
                    }
                });

                pending_off_notes.pop();
            }

            // Super unlikely to occur. It comes out to about 279k beats at 480 resolution
            let delta = ev_pos - prev_note_pos;
            if delta > MAX_DELTA {
                panic!("Unsupported: Delta distance of {delta} between {} and {} is larger than max {MAX_DELTA}",
                    prev_note_pos,
                    ev_pos
                );
            }

            let ev_kind = match ev {
                MidiEvent::Note(note) => {
                    // Track note off position
                    let note_off_pos = ev_pos + note.length;
                    pending_off_notes.push(Reverse((note_off_pos, note.pitch, note.channel, note.velocity)));

                    TrackEventKind::Midi {
                        channel: note.channel.into(),
                        message: MidiMessage::NoteOn {
                            key: note.pitch.into(),
                            vel: note.velocity.into()
                        }
                    }
                },
                MidiEvent::Meta(meta) => {
                    let meta_message = match &meta.text {
                        MidiTextType::Event(ev) => midly::MetaMessage::Text(ev.as_ref()),
                        MidiTextType::Lyric(lyric) => midly::MetaMessage::Lyric(lyric.as_ref()),
                    };

                    TrackEventKind::Meta(meta_message)
                },
                MidiEvent::SysEx(sysex) => {
                    TrackEventKind::SysEx(sysex.data.as_ref())
                }
            };

            track.push(TrackEvent {
                delta: (delta as u32).into(),
                kind: ev_kind
            });

            prev_note_pos = ev_pos;
        }

        // Process remaining note off events
        while let Some(Reverse((off_pos, off_pitch, off_channel, off_velocity))) = pending_off_notes.pop() {
            let off_delta = off_pos - prev_note_pos;
            prev_note_pos = off_pos;

            // Add note off event
            track.push(TrackEvent {
                delta: (off_delta as u32).into(),
                kind: TrackEventKind::Midi {
                    channel: off_channel.into(),
                    message: MidiMessage::NoteOff {
                        key: off_pitch.into(),
                        vel: off_velocity.into()
                    }
                }
            });
        }

        // Add end of track event
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)
        });

        track
    }

    pub fn write_to_file<T: AsRef<Path>>(&self, path: T) {
        let path = path.as_ref();

        let format = match self.format {
            0 => MidiFormat::SingleTrack,
            1 => MidiFormat::Parallel,
            _ => MidiFormat::Sequential
        };

        let header = MidiHeader::new(format, MidiTiming::Metrical(self.ticks_per_quarter.into()));
        let mut smf = Smf::new(header);

        // Add tempo track
        let tempo_track = self.generate_tempo_track();
        smf.tracks.push(tempo_track);

        // Add tracks
        for track_index in 0..self.tracks.len() {
            let track_events = self.generate_track(track_index);
            smf.tracks.push(track_events);
        }

        // Write midi file
        let file = std::fs::File::create(path).unwrap();
        midly::write_std(&smf.header, &smf.tracks, file).unwrap();
    }
}