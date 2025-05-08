use crate::{SystemInfo};
use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::{ObjectReadWrite, P9SongPref, load_object, save_object};
use thiserror::Error as ThisError;
use std::error::Error;

#[derive(Debug, ThisError)]
pub enum P9SongPrefReadError {
    #[error("P9SongPref version of {version} not supported")]
    P9SongPrefVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
        20 | 25 => true,
        _ => false
    }
}

impl ObjectReadWrite for P9SongPref {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;

        // If not valid, return unsupported error
        if !is_version_supported(version) {
            return Err(Box::new(P9SongPrefReadError::P9SongPrefVersionNotSupported {
                version
            }));
        }

        load_object(self, &mut reader, info)?;

        // Reset collections
        self.minivenues.clear();
        self.scenes.clear();
        self.instruments.iter_mut().for_each(|ins| ins.clear());

        self.venue = reader.read_prefixed_string()?;

        let minivenues_count = reader.read_uint32()?;
        for _ in 0..minivenues_count {
            let minivenue = reader.read_prefixed_string()?;
            self.minivenues.push(minivenue);
        }

        let scenes_count = reader.read_uint32()?;
        for _ in 0..scenes_count {
            let scene = reader.read_prefixed_string()?;
            self.scenes.push(scene);
        }

        // TODO: Support reading scene groups?
        let _scene_group_count = reader
            .read_uint32()
            .expect("scene_group count is non-zero in P9SongPref");

        self.dreamscape_outfit = reader.read_prefixed_string()?;
        self.studio_outfit = reader.read_prefixed_string()?;

        for char_instruments in self.instruments.iter_mut() {
            let instrument_count = reader.read_uint32()?;

            for _ in 0..instrument_count {
                let instrument = reader.read_prefixed_string()?;
                char_instruments.push(instrument);
            }
        }

        self.tempo = reader.read_prefixed_string()?;
        self.song_clips = reader.read_prefixed_string()?;
        self.dreamscape_font = reader.read_prefixed_string()?;

        // TBRB
        if version <= 20 {
            self.george_amp = reader.read_prefixed_string()?;
            self.john_amp = reader.read_prefixed_string()?;
            self.paul_amp = reader.read_prefixed_string()?;
            self.mixer = reader.read_prefixed_string()?;
        }

        self.dreamscape_camera = reader.read_prefixed_string()?; // Usually empty for GDRB
        self.lyric_part = reader.read_prefixed_string()?;

        // GDRB
        if version >= 25 {
            self.normal_outfit = reader.read_prefixed_string()?;
            self.bonus_outfit = reader.read_prefixed_string()?;
            self.drum_set = reader.read_prefixed_string()?;

            self.era = reader.read_prefixed_string()?;

            self.cam_directory = reader.read_prefixed_string()?;
            self.media_directory = reader.read_prefixed_string()?;

            self.song_intro_cam = reader.read_prefixed_string()?;
            self.win_cam = reader.read_prefixed_string()?;
        }

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut writer = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = if self.dreamscape_camera.is_empty() { 25 } else { 20 }; // GDRB doesn't have dreamscapes :(
        writer.write_uint32(version)?;

        save_object(self, &mut writer, info)?;

        writer.write_prefixed_string(&self.venue)?;

        writer.write_uint32(self.minivenues.len() as u32)?;
        for mv in self.minivenues.iter() {
            writer.write_prefixed_string(mv)?;
        }

        writer.write_uint32(self.scenes.len() as u32)?;
        for scene in self.scenes.iter() {
            writer.write_prefixed_string(scene)?;
        }

        writer.write_int32(0)?;

        writer.write_prefixed_string(&self.dreamscape_outfit)?;
        writer.write_prefixed_string(&self.studio_outfit)?;

        for char_instruments in self.instruments.iter() {
            writer.write_uint32(char_instruments.len() as u32)?;

            for instrument in char_instruments.iter() {
                writer.write_prefixed_string(instrument)?;
            }
        }

        writer.write_prefixed_string(&self.tempo)?;
        writer.write_prefixed_string(&self.song_clips)?;
        writer.write_prefixed_string(&self.dreamscape_font)?;

        // TBRB
        if version <= 20 {
            writer.write_prefixed_string(&self.george_amp)?;
            writer.write_prefixed_string(&self.john_amp)?;
            writer.write_prefixed_string(&self.paul_amp)?;
            writer.write_prefixed_string(&self.mixer)?;
        }

        writer.write_prefixed_string(&self.dreamscape_camera)?;
        writer.write_prefixed_string(&self.lyric_part)?;

        // GDRB
        if version >= 25 {
            writer.write_prefixed_string(&self.normal_outfit)?;
            writer.write_prefixed_string(&self.bonus_outfit)?;
            writer.write_prefixed_string(&self.drum_set)?;

            writer.write_prefixed_string(&self.era)?;

            writer.write_prefixed_string(&self.cam_directory)?;
            writer.write_prefixed_string(&self.media_directory)?;

            writer.write_prefixed_string(&self.song_intro_cam)?;
            writer.write_prefixed_string(&self.win_cam)?;
        }

        Ok(())
    }
}