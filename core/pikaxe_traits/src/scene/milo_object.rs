pub trait MiloObject {
    fn get_name(&self) -> &String;
    fn get_name_mut(&mut self) -> &mut String;
    fn set_name(&mut self, name: String);

    fn get_type(&self) -> &String;
    fn get_type_mut(&mut self) -> &mut String;
    fn set_type(&mut self, type2: String);

    // TODO: Add after dtb code written
    /*fn get_props(&self) -> DataArray;
    fn get_props_mut(&mut self) -> &mut DataArray;
    fn set_props(&mut self, data_array: DataArray);*/

    fn get_note(&self) -> &String;
    fn get_note_mut(&mut self) -> &mut String;
    fn set_note(&mut self, note: String);

    //fn get_object_type(&self) -> &str;
}