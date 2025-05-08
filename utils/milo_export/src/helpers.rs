use pikaxe::dta::DataArray;
use pikaxe::{Platform, SystemInfo};
use pikaxe::io::*;
use pikaxe::scene::{Object, ObjectDir, ObjectDirBase, PackedObject, PropAnim, PropKeysEvents, Tex, AnimRate};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::{PathBuf, Path};

const PLATFORMS: [&str; 3] = ["ps3", "wii", "xbox"];

const GDRB_CHARACTERS: [(&str, &str); 3] = [
    ("BILLIE", "billiejoe"),
    ("MIKE", "mikedirnt"),
    ("TRE", "trecool")
];

const GDRB_VENUE_NAMES: [&str; 3] = [
    "americanidiot",
    "dookie",
    "twentyfirst"
];

#[derive(Default)]
pub struct GameAnalyzer {
    pub game_dir: PathBuf,
    pub song_ids: Vec<String>,
    pub cams: Cams,
    pub post_procs: Vec<String>,
    pub char_clips: CharClips,
    pub light_presets: Vec<ValueCollection<String>>,
    pub trigger_groups: Vec<ValueCollection<String>>,
    pub prop_anims: Vec<ValueCollection<PropAnimInfo>>
}

impl GameAnalyzer {
    pub fn new(game_path: PathBuf) -> GameAnalyzer {
        let song_ids = find_song_ids(game_path.as_path());

        GameAnalyzer {
            game_dir: game_path,
            song_ids,
            ..Default::default()
         }
    }

    pub fn process(&mut self) {
        self.process_venues();
        self.process_songs();
        self.process_post_procs();
        self.process_prop_anims();
    }

    fn process_venues(&mut self) {
        for venue_name in GDRB_VENUE_NAMES.iter() {
            let mut entries = Vec::new();

            // Open venue milo
            let venue_file_name = format!("{venue_name}.milo");
            let venue_path = self.game_dir.join("world").join(venue_name).join(&venue_file_name);
            let venue_milo_dir = try_open_milo(venue_path.as_path());
            if venue_milo_dir.is_err() {
                continue;
            }

            if let Ok((_, mut venue_milo_dir)) = venue_milo_dir {
                entries.append(venue_milo_dir.get_entries_mut());
            }

            // Open venue lighting milo
            let venue_lighting_file_name = format!("{venue_name}_lighting.milo");
            let mut venue_lighting_path = self.game_dir.join("world").join(venue_name).join(&venue_lighting_file_name);
            let mut venue_lighting_milo_dir = try_open_milo(venue_lighting_path.as_path());

            if venue_lighting_milo_dir.is_err() {
                // Try looking in platform specific directory
                for platform in ["ps3", "xbx"] {
                    venue_lighting_path = self.game_dir
                        .join("world")
                        .join(venue_name)
                        .join(platform)
                        .join(&venue_lighting_file_name);

                    venue_lighting_milo_dir = try_open_milo(venue_lighting_path.as_path());
                    if venue_lighting_milo_dir.is_ok() {
                        break;
                    }
                }
            }

            if let Ok((_, mut venue_lighting_milo_dir)) = venue_lighting_milo_dir {
                entries.append(venue_lighting_milo_dir.get_entries_mut());
            }

            // Get cams
            self.cams.venues.push(ValueCollection {
                id: venue_name.to_string(),
                values: get_names_for_type_from_dir(&entries, "BandCamShot")
            });

            // Get light presets
            self.light_presets.push(ValueCollection {
                id: venue_name.to_string(),
                values: get_names_for_type_from_dir(&entries, "LightPreset")
            });

            // Get trigger groups
            self.trigger_groups.push(ValueCollection {
                id: venue_name.to_string(),
                values: get_names_for_type_from_dir(&entries, "TriggerGroup")
            });
        }
    }

    fn process_songs(&mut self) {
        for song_name in self.song_ids.iter() {
            let song_dir = self.game_dir
                .join("songs")
                .join(song_name);

            // Get cams
            let milo_cams_file_name = format!("{song_name}_cams.milo");
            let cams_path = song_dir.join(&milo_cams_file_name);
            if let Ok((_, cams_milo_dir)) = try_open_milo(cams_path.as_path()) {
                let cams = get_names_for_type_from_dir(&cams_milo_dir.get_entries(), "BandCamShot");

                self.cams.songs.push(ValueCollection {
                    id: song_name.to_string(),
                    values: cams
                });
            }

            // Get char clips
            let mut song_clips = ValueCollection {
                id: song_name.to_string(),
                values: Vec::new()
            };

            let anims_file_name = format!("{song_name}.milo");
            for (_, long_char_name) in GDRB_CHARACTERS.iter() {
                let anims_path = self.game_dir
                    .join("char")
                    .join(long_char_name)
                    .join("song")
                    .join(&anims_file_name);

                if let Ok((_, anims_milo_dir)) = try_open_milo(anims_path.as_path()) {
                    let clips = get_names_for_type_from_dir(&anims_milo_dir.get_entries(), "CharClipGroup");

                    song_clips.values.push(ValueCollection {
                        id: long_char_name.to_string(),
                        values: clips
                    });
                }
            }

            self.char_clips.songs.push(song_clips);
        }
    }

    fn process_prop_anims(&mut self) {
        let mut tracked_prop_anims: HashMap<String, HashMap<String, (String, Option<String>, u32, Option<String>, u32, HashSet<String>)>> = HashMap::default();

        for (_, char_long_name) in GDRB_CHARACTERS.iter() {
            tracked_prop_anims.insert(char_long_name.to_string(), HashMap::new());
        }
        tracked_prop_anims.insert(String::from("venue"), HashMap::new());

        let mapped_cams = self
            .cams
            .songs
            .iter()
            .flat_map(|c| &c.values)
            .chain(self
                .cams
                .venues
                .iter()
                .flat_map(|c| &c.values)
            )
            .map(|s| s.as_str())
            .collect::<HashSet<_>>();

        let mapped_clips = self
            .char_clips
            .songs
            .iter()
            .flat_map(|c| &c.values)
            .flat_map(|c| &c.values)
            .map(|s| s.as_str())
            .collect::<HashSet<_>>();

        let mapped_post_procs = self
            .post_procs
            .iter()
            .map(|s| s.as_str())
            .collect::<HashSet<_>>();

        let mapped_light_presets = self
            .light_presets
            .iter()
            .flat_map(|lp| &lp.values)
            .map(|s| s.as_str())
            .collect::<HashSet<_>>();

        let mapped_trigger_groups = self
            .trigger_groups
            .iter()
            .flat_map(|lp| &lp.values)
            .map(|s| s.as_str())
            .collect::<HashSet<_>>();

        let is_mapped = |value: &str| -> bool {
            mapped_cams.contains(value)
                || mapped_clips.contains(value)
                || mapped_post_procs.contains(value)
                || mapped_light_presets.contains(value)
                || mapped_trigger_groups.contains(value)
        };

        mapped_cams.contains(&"test");

        for song_name in self.song_ids.iter() {
            let song_dir = self.game_dir
                .join("songs")
                .join(song_name);

            // Get prop anims
            let milo_file_name = format!("{song_name}_ap.milo");
            let milo_path = song_dir.join(&milo_file_name);
            let prop_anim_entry = try_open_milo(milo_path.as_path())
                .ok()
                .and_then(|(sys_info, milo_dir)| milo_dir
                    .get_entries()
                    .iter()
                    .find_map(|e| match e.get_name() {
                        "song.anim" => e.unpack(&sys_info)
                            .and_then(|e| match e {
                                Object::PropAnim(p) => Some(p),
                                _ => None
                            }),
                        _ => None
                    }));

            if let Some(prop_anim) = prop_anim_entry {
                for prop_keys in prop_anim.keys {
                    // Assume single symbol for now (most common for TBRB/GDRB song anims)
                    let property = prop_keys
                        .property
                        .first()
                        .and_then(|node| match node {
                            DataArray::Symbol(s) => s.as_utf8(),
                            _ => None,
                        })
                        .unwrap()
                        .to_owned();

                    let mut property_short = None;

                    let mut track = tracked_prop_anims.get_mut("venue").unwrap(); // Use venue group by default

                    for (_, char_long_name) in GDRB_CHARACTERS.iter() {
                        if property.contains(char_long_name) {
                            let key_slice = format!("_{char_long_name}");

                            // Update short property name and use dedicated character track
                            property_short = Some(property.replace(&key_slice, ""));
                            track = tracked_prop_anims.get_mut(*char_long_name).unwrap();
                            break;
                        }
                    }

                    let (ev_type, mut ev_values) = match prop_keys.events {
                        PropKeysEvents::Float(_) => ("float", None),
                        PropKeysEvents::Color(_) => ("color", None),
                        PropKeysEvents::Object(evs) => ("object", Some(evs
                            .into_iter()
                            .map(|e| e.text2) // Only text2 is used in songs
                            .filter(|s| !s.is_empty() && !is_mapped(&s))
                            .collect::<Vec<_>>()
                        )),
                        PropKeysEvents::Bool(_) => ("bool", None),
                        PropKeysEvents::Quat(_) => ("quat", None),
                        PropKeysEvents::Vector3(_) => ("vector3", None),
                        PropKeysEvents::Symbol(evs) => ("symbol", Some(evs
                            .into_iter()
                            .map(|e| e.text)
                            .filter(|s| !s.is_empty() && !is_mapped(&s))
                            .collect::<Vec<_>>()
                        )),
                    };

                    let (_, _, _, _, _, values) = track
                        .entry(property)
                        .or_insert_with(|| (
                            ev_type.to_owned(),
                            property_short,
                            prop_keys.interpolation,
                            if !prop_keys.interp_handler.is_empty() { Some(prop_keys.interp_handler.to_owned()) } else { None },
                            prop_keys.unknown_enum,
                            HashSet::new()
                        ));

                    if let Some(ev_values) = ev_values.take() {
                        values.extend(ev_values);
                    }
                }
            }
        }

        // Map to prop anim values and sort
        self.prop_anims = tracked_prop_anims
            .into_iter()
            .map(|(key, value)| ValueCollection {
                id: key,
                values: {
                    let mut vals = value
                        .into_iter()
                        .map(|(key, (anim_type, key_short, interpolation, interp_handler, unknown_enum, values))| PropAnimInfo {
                            property: key,
                            property_short: key_short, 
                            interpolation,
                            interp_handler,
                            unknown_enum,
                            r#type: anim_type,
                            values: {
                                let mut vals = values.into_iter().collect::<Vec<_>>();
                                vals.sort();
                                vals
                            }
                        })
                        .collect::<Vec<_>>();

                    vals.sort_by(|a, b| match (&a.property_short, &b.property_short) {
                        (Some(a), Some(b)) => a.cmp(b),
                        _ => a.property.cmp(&b.property)
                    });
                    vals
                }
            })
            .collect();

        self.prop_anims.sort_by(|a, b| a.id.cmp(&b.id));
    }

    fn process_post_procs(&mut self) {
        // Read post procs
        let post_procs_path = self.game_dir
            .join("world")
            .join("shared")
            .join("camera.milo");

        if let Ok((_, post_procs_dir)) = try_open_milo(post_procs_path.as_path()) {
            self.post_procs = get_names_for_type_from_dir(&post_procs_dir.get_entries(), "PostProc");
        }
    }

    pub fn export<T: AsRef<Path>>(&self, output_dir: T) {
        let output_dir = output_dir.as_ref();

        // Create dir
        if !output_dir.exists() {
            std::fs::create_dir(output_dir).unwrap();
        }

        // Write cams
        let cams_json = serde_json::to_string_pretty(&self.cams).unwrap();
        std::fs::write(output_dir.join("cams.json"), cams_json)
            .expect("Error \"cams.json\" to file");

        // Write post procs
        let post_procs_json = serde_json::to_string_pretty(&self.post_procs).unwrap();
        std::fs::write(output_dir.join("post_procs.json"), post_procs_json)
            .expect("Error \"post_procs.json\" to file");

        // Write char clips
        let char_clips_json = serde_json::to_string_pretty(&self.char_clips).unwrap();
        std::fs::write(output_dir.join("char_clips.json"), char_clips_json)
            .expect("Error \"char_clips.json\" to file");

        // Write light presets
        let light_presets_json = serde_json::to_string_pretty(&self.light_presets).unwrap();
        std::fs::write(output_dir.join("light_presets.json"), light_presets_json)
            .expect("Error \"light_presets.json\" to file");

        // Write trigger groups
        let trigger_groups_json = serde_json::to_string_pretty(&self.trigger_groups).unwrap();
        std::fs::write(output_dir.join("trigger_groups.json"), trigger_groups_json)
            .expect("Error \"trigger_groups.json\" to file");

        // Write prop anims
        let prop_anims_json = serde_json::to_string_pretty(&self.prop_anims).unwrap();
        std::fs::write(output_dir.join("prop_anims.json"), prop_anims_json)
            .expect("Error \"prop_anims.json\" to file");
    }
}

fn try_open_milo(milo_path: &Path) -> Result<(SystemInfo, ObjectDir), Box<dyn Error>> {
    let dir_path = milo_path
        .parent()
        .unwrap();

    let file_name = milo_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap();

    for ext in PLATFORMS.iter() {
        let milo_file_name = PathBuf::from(format!("{file_name}_{ext}"));

        // First try regular path
        let mut result = open_milo(dir_path.join(&milo_file_name).as_path());
        if result.is_ok() {
            return result;
        }

        // Then try w/ gen path
        result = open_milo(dir_path.join("gen").join(&milo_file_name).as_path());
        if result.is_ok() {
            return result;
        }
    }

    Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")))
}

fn open_milo(milo_path: &Path) -> Result<(SystemInfo, ObjectDir), Box<dyn Error>> {
    // Open milo
    let mut stream = FileStream::from_path_as_read_open(&milo_path)?;
    let milo = MiloArchive::from_stream(&mut stream)?;

    // Unpack dir and entries
    let system_info = SystemInfo::guess_system_info(&milo, &milo_path);
    let obj_dir = milo.unpack_directory(&system_info)?;

    Ok((system_info, obj_dir))
}

fn get_names_for_type_from_dir(entries: &[Object], entry_type: &str) -> Vec<String> {
    let mut entries = entries
        .iter()
        .filter(|e| e.get_type().eq(entry_type))
        .map(|e| e.get_name().to_string())
        .collect::<Vec<_>>();

    // Ugh rust std doesn't provide way to compare case-insensitive
    entries.sort_by(|a, b| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));
    entries
}

fn find_song_ids(game_dir: &Path) -> Vec<String> {
    game_dir.join("songs")
        .read_dir()
        .unwrap()
        .map(|d| d.unwrap().path())
        .filter(|d| d.is_dir())
        .map(|p| p.file_name().and_then(|f| f.to_str()).map(|s| s.to_string()).unwrap())
        .filter(|s| s.ne("gen"))
        .collect::<Vec<_>>()
}

#[derive(Default, Deserialize, Serialize)]
pub struct ValueCollection<T> {
    pub id: String,
    pub values: Vec<T>
}

#[derive(Default, Deserialize, Serialize)]
pub struct Cams {
    pub venues: Vec<ValueCollection<String>>,
    pub songs: Vec<ValueCollection<String>>
}

#[derive(Default, Deserialize, Serialize)]
pub struct CharClips {
    pub songs: Vec<ValueCollection<ValueCollection<String>>>
}

#[derive(Default, Deserialize, Serialize)]
pub struct PropAnimInfo {
    pub property: String,
    pub property_short: Option<String>,
    pub interpolation: u32,
    pub interp_handler: Option<String>,
    pub unknown_enum: u32,
    pub r#type: String,
    pub values: Vec<String>
}