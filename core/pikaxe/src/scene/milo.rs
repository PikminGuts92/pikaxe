use crate::{SystemInfo};
use crate::scene::*;

pub struct Milo {
    pub entries: Vec<Object>,
    pub directory: Option<ObjectDir>, // Used starting w/ v24?
}

impl Milo {
    pub fn new() -> Milo {
        Milo {
            entries: Vec::new(),
            directory: None
        }
    }

    pub fn unpack_entries(&mut self, info: &SystemInfo) {
        let mut new_entries = Vec::<Object>::new();

        while !self.entries.is_empty() {
            let object = self.entries.remove(0);

            let new_object = match object.unpack(info) {
                Some(obj) => obj,
                None => object
            };

            new_entries.push(new_object);
        }

        // Assign new entries
        self.entries = new_entries;
    }
}