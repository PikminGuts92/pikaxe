use super::MiloObject;

pub trait Poll : MiloObject {
    fn get_target_1(&self) -> &String;
    fn get_target_1_mut(&mut self) -> &mut String;
    fn set_target_1(&mut self, target: String);

    fn get_target_2(&self) -> &String;
    fn get_target_2_mut(&mut self) -> &mut String;
    fn set_target_2(&mut self, target: String);
}