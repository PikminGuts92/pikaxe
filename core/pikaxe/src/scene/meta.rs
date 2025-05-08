use pikaxe_macros::*;

/*pub trait Version {
    fn get_version(&self) -> Option<u32>;
    fn set_version(&mut self, version: Option<u32>);
}*/

pub trait Name {
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);
}

pub trait AsObject<T>: AsMut<T> + AsRef<T> {}

pub trait Metadata: AsObject<MetadataObject> {
    //fn get_metadata(&mut)
}

//#[derive(Version)]
#[version]
pub struct MetadataObject {
    // pub version: Option<u32>,
    pub revision: Option<u32>,
    pub r#type: String,
    // pub props: DataArray,
    pub note: String,
}