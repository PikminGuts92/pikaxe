use super::{Anim, Draw, MiloObject, Trans};

pub trait Group : Anim + Draw + MiloObject + Trans {
    fn get_objects(&self) -> &Vec<String>;
    fn get_objects_mut(&mut self) -> &mut Vec<String>;
    fn set_objects(&mut self, objects: Vec<String>);

    fn get_environ(&self) -> &String;
    fn get_environ_mut(&mut self) -> &mut String;
    fn set_environ(&mut self, environ: String);

    fn get_draw_only(&self) -> &String;
    fn get_draw_only_mut(&mut self) -> &mut String;
    fn set_draw_only(&mut self, draw_only: String);

    fn get_lod(&self) -> &String;
    fn get_lod_mut(&mut self) -> &mut String;
    fn set_lod(&mut self, lod: String);

    fn get_lod_screen_size(&self) -> f32;
    fn set_lod_screen_size(&mut self, lod_screen_size: f32);

    fn get_sort_in_world(&self) -> bool;
    fn set_sort_in_world(&mut self, sort_in_world: bool);
}