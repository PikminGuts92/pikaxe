use crate::SystemInfo;
use crate::io::MemoryStream;
use crate::scene::*;

pub enum Object {
    Anim(AnimObject),
    BandPlacer(BandPlacer),
    Cam(CamObject),
    CharBone(CharBone),
    CharClipSamples(CharClipSamples),
    CharHair(CharHair),
    CharLipSync(CharLipSync),
    ColorPalette(ColorPalette),
    CubeTex(CubeTexObject),
    Draw(DrawObject),
    Group(GroupObject),
    Mat(MatObject),
    Mesh(MeshObject),
    MeshAnim(MeshAnim),
    Morph(Morph),
    P9SongPref(P9SongPref),
    PropAnim(PropAnim),
    SynthSample(SynthSample),
    Tex(Tex),
    Trans(TransObject),
    TransAnim(TransAnim),
    Packed(PackedObject),
}

#[derive(Debug)]
pub struct PackedObject {
    pub name: String,
    pub object_type: String,
    pub data: Vec<u8>
}

impl Object {
    pub fn get_name(&self) -> &str {
        match self {
            Object::Anim(anim) => &anim.name,
            Object::BandPlacer(band_placer) => &band_placer.name,
            Object::Cam(cam) => &cam.name,
            Object::CharBone(char_bone) => &char_bone.name,
            Object::CharClipSamples(ccs) => &ccs.name,
            Object::CharHair(ch) => &ch.name,
            Object::CharLipSync(cls) => &cls.name,
            Object::ColorPalette(color_palette) => &color_palette.name,
            Object::CubeTex(cube) => &cube.name,
            Object::Draw(draw) => &draw.name,
            Object::Group(grp) => &grp.name,
            Object::Mat(mat) => &mat.name,
            Object::Mesh(mesh) => &mesh.name,
            Object::MeshAnim(mesh_anim) => &mesh_anim.name,
            Object::Morph(morph) => &morph.name,
            Object::P9SongPref(pref) => &pref.name,
            Object::PropAnim(prop) => &prop.name,
            Object::SynthSample(synth) => &synth.name,
            Object::Tex(tex) => &tex.name,
            Object::Trans(trans) => &trans.name,
            Object::TransAnim(trans_anim) => &trans_anim.name,
            Object::Packed(packed) => &packed.name,
        }
    }

    pub fn get_type(&self) -> &str {
        match self {
            Object::Anim(_) => "Anim",
            Object::BandPlacer(_) => "BandPlacer",
            Object::Cam(_) => "Cam",
            Object::CharBone(_) => "CharBone",
            Object::CharClipSamples(_) => "CharClipSamples",
            Object::CharHair(_) => "CharHair",
            Object::CharLipSync(_) => "CharLipSync",
            Object::ColorPalette(_) => "ColorPalette",
            Object::CubeTex(_) => "CubeTex",
            Object::Draw(_) => "Draw",
            Object::Group(_) => "Group",
            Object::Mat(_) => "Mat",
            Object::Mesh(_) => "Mesh",
            Object::MeshAnim(_) => "MeshAnim",
            Object::Morph(_) => "Morph",
            Object::P9SongPref(_) => "P9SongPref",
            Object::PropAnim(_) => "PropAnim",
            Object::SynthSample(_) => "SynthSample",
            Object::Tex(_) => "Tex",
            Object::Trans(_) => "Trans",
            Object::TransAnim(_) => "TransAnim",
            Object::Packed(packed) => &packed.object_type,
        }
    }

    pub fn is_packed(&self) -> bool {
        match self {
            Object::Packed(_) => true,
            _ => false
        }
    }

    pub fn pack(&self, info: &SystemInfo) -> Option<Object> {
        if self.is_packed() {
            todo!("Already packed");
        }

        let obj: &dyn ObjectReadWrite  = match &self {
            Object::Anim(obj) => obj,
            Object::BandPlacer(obj) => obj,
            Object::Cam(obj) => obj,
            Object::CharClipSamples(obj) => obj,
            Object::ColorPalette(obj) => obj,
            Object::CubeTex(obj) => obj,
            Object::Draw(obj) => obj,
            Object::Group(obj) => obj,
            Object::Mat(obj) => obj,
            Object::Mesh(obj) => obj,
            Object::Morph(obj) => obj,
            Object::P9SongPref(obj) => obj,
            Object::PropAnim(obj) => obj,
            Object::Tex(obj) => obj,
            Object::Trans(obj) => obj,
            Object::TransAnim(obj) => obj,
            _ => todo!("Can't pack \"{}\" yet!", self.get_type()),
        };

        let mut data = Vec::new();
        let mut stream = MemoryStream::from_vector_as_read_write(&mut data);

        if obj.save(&mut stream, info).is_err() {
            println!("ERROR: Unable to pack {} with type {}", self.get_name(), self.get_type());
            return None;
        }

        // Return packed object
        Some(Object::Packed(PackedObject {
            name: self.get_name().to_owned(),
            object_type: self.get_type().to_owned(),
            data,
        }))
    }

    pub fn unpack(&self, info: &SystemInfo) -> Option<Object> {
        match self {
            Object::Packed(packed) => {
                match packed.object_type.as_str() {
                    "Anim" => unpack_object(packed, info).map(|o| Object::Anim(o)),
                    "BandPlacer" => unpack_object(packed, info).map(|o| Object::BandPlacer(o)),
                    "Cam" => unpack_object(packed, info).map(|o| Object::Cam(o)),
                    "CharClipSamples" => unpack_object(packed, info).map(|o| Object::CharClipSamples(o)),
                    "CharHair" => unpack_object(packed, info).map(|o| Object::CharHair(o)),
                    "CharLipSync" => unpack_object(packed, info).map(|o| Object::CharLipSync(o)),
                    "ColorPalette" => unpack_object(packed, info).map(|o| Object::ColorPalette(o)),
                    "CubeTex" => unpack_object(packed, info).map(|o| Object::CubeTex(o)),
                    "Draw" => unpack_object(packed, info).map(|o| Object::Draw(o)),
                    "Group" => unpack_object(packed, info).map(|o| Object::Group(o)),
                    "Mat" => unpack_object(packed, info).map(|o| Object::Mat(o)),
                    "Mesh" => unpack_object(packed, info).map(|o| Object::Mesh(o)),
                    "MeshAnim" => unpack_object(packed, info).map(|o| Object::MeshAnim(o)),
                    "Morph" => unpack_object(packed, info).map(|o| Object::Morph(o)),
                    "P9SongPref" => unpack_object(packed, info).map(|o| Object::P9SongPref(o)),
                    "PropAnim" => unpack_object(packed, info).map(|o| Object::PropAnim(o)),
                    "SynthSample" => unpack_object(packed, info).map(|o| Object::SynthSample(o)),
                    "Tex" => {
                        let mut stream = MemoryStream::from_slice_as_read(packed.data.as_slice());

                        // TODO: Update tex to use same io traits
                        match Tex::from_stream(&mut stream, info) {
                            Ok(mut tex) => {
                                tex.name = packed.name.to_owned();
                                Some(Object::Tex(tex))
                            },
                            Err(_) => None,
                        }
                    },
                    "Trans" => unpack_object(packed, info).map(|o| Object::Trans(o)),
                    "TransAnim" => unpack_object(packed, info).map(|o| Object::TransAnim(o)),
                    _ => None
                }
            },
            _ => None
        }
    }
}

fn unpack_object<T: Default + MiloObject + ObjectReadWrite>(packed: &PackedObject, info: &SystemInfo) -> Option<T> {
    let mut stream = MemoryStream::from_slice_as_read(packed.data.as_slice());

    let mut obj = T::default();

    if obj.load(&mut stream, info).is_ok() {
        obj.set_name(packed.name.to_owned());
        return Some(obj);
    }

    None
}