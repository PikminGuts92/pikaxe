use crate::dta::*;
use crate::io::{BinaryStream, FileStream, SeekFrom, Stream};
use crate::SystemInfo;
use grim_traits::scene::{Color3, Color4, Matrix, Matrix3, MiloObject, Quat, Rect, Sphere, Vector2, Vector3};
use std::error::Error;
use std::path::Path;

pub trait ObjectReadWrite {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>>;
    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>>;
}

pub fn save_to_file<T: ObjectReadWrite, S: AsRef<Path>>(obj: &T, out_path: S, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
    // Write to file
    let mut stream = FileStream::from_path_as_read_write_create(out_path.as_ref())?;
    obj.save(&mut stream, info)
}

pub(crate) fn load_object<T: MiloObject>(obj: &mut T, reader: &mut Box<BinaryStream>, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
    if info.version < 24 {
        // Don't read metadata
        return Ok(());
    }

    load_object_type(obj, reader, info)?;
    load_object_rest(obj, reader, info)?;

    Ok(())
}

pub(crate) fn save_object<T: MiloObject>(obj: &T, writer: &mut Box<BinaryStream>, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
    save_object_type(obj, writer, info)?;
    save_object_rest(obj, writer, info)?;

    Ok(())
}


pub(crate) fn load_object_type<T: MiloObject>(obj: &mut T, reader: &mut Box<BinaryStream>, _info: &SystemInfo) -> Result<(), Box<dyn Error>> {
    // Skip revision for now
    reader.read_uint32()?;

    // Read type
    obj.set_type(reader.read_prefixed_string()?);

    Ok(())
}

pub(crate) fn save_object_type<T: MiloObject>(obj: &T, writer: &mut Box<BinaryStream>, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
    // Write revision
    writer.write_uint32(info.get_revision())?;

    // Write type
    writer.write_prefixed_string(obj.get_type())?;

    Ok(())
}

pub(crate) fn load_object_rest<T: MiloObject>(obj: &mut T, reader: &mut Box<BinaryStream>, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
    // Read props
    // Parse dtb data but don't save for now
    let mut root = RootData::new();
    root.load(reader)?;

    // Read note
    if info.version >= 25 {
        obj.set_note(reader.read_prefixed_string()?);
    }

    Ok(())
}

pub(crate) fn save_object_rest<T: MiloObject>(obj: &T, writer: &mut Box<BinaryStream>, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
    // TODO: Write props
    writer.write_boolean(false)?;

    // Write note
    if info.version >= 25 {
        writer.write_prefixed_string(obj.get_note())?;
    }

    Ok(())
}

pub (crate) fn load_color3(color: &mut Color3, reader: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    color.r = reader.read_float32()?;
    color.g = reader.read_float32()?;
    color.b = reader.read_float32()?;

    Ok(())
}

pub (crate) fn save_color3(color: &Color3, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_float32(color.r)?;
    writer.write_float32(color.g)?;
    writer.write_float32(color.b)?;

    Ok(())
}

pub (crate) fn load_color4(color: &mut Color4, reader: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    color.r = reader.read_float32()?;
    color.g = reader.read_float32()?;
    color.b = reader.read_float32()?;
    color.a = reader.read_float32()?;

    Ok(())
}

pub (crate) fn save_color4(color: &Color4, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_float32(color.r)?;
    writer.write_float32(color.g)?;
    writer.write_float32(color.b)?;
    writer.write_float32(color.a)?;

    Ok(())
}


pub (crate) fn load_matrix(mat: &mut Matrix, reader: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    mat.m11 = reader.read_float32()?;
    mat.m12 = reader.read_float32()?;
    mat.m13 = reader.read_float32()?;
    mat.m14 = 0.0;

    mat.m21 = reader.read_float32()?;
    mat.m22 = reader.read_float32()?;
    mat.m23 = reader.read_float32()?;
    mat.m24 = 0.0;

    mat.m31 = reader.read_float32()?;
    mat.m32 = reader.read_float32()?;
    mat.m33 = reader.read_float32()?;
    mat.m34 = 0.0;

    mat.m41 = reader.read_float32()?;
    mat.m42 = reader.read_float32()?;
    mat.m43 = reader.read_float32()?;
    mat.m44 = 1.0;

    Ok(())
}

pub (crate) fn save_matrix(mat: &Matrix, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_float32(mat.m11)?;
    writer.write_float32(mat.m12)?;
    writer.write_float32(mat.m13)?;

    writer.write_float32(mat.m21)?;
    writer.write_float32(mat.m22)?;
    writer.write_float32(mat.m23)?;

    writer.write_float32(mat.m31)?;
    writer.write_float32(mat.m32)?;
    writer.write_float32(mat.m33)?;

    writer.write_float32(mat.m41)?;
    writer.write_float32(mat.m42)?;
    writer.write_float32(mat.m43)?;

    Ok(())
}

pub (crate) fn load_matrix3(mat: &mut Matrix3, reader: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    mat.m11 = reader.read_float32()?;
    mat.m12 = reader.read_float32()?;
    mat.m13 = reader.read_float32()?;

    mat.m21 = reader.read_float32()?;
    mat.m22 = reader.read_float32()?;
    mat.m23 = reader.read_float32()?;

    mat.m31 = reader.read_float32()?;
    mat.m32 = reader.read_float32()?;
    mat.m33 = reader.read_float32()?;

    Ok(())
}

pub (crate) fn save_matrix3(mat: &Matrix3, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_float32(mat.m11)?;
    writer.write_float32(mat.m12)?;
    writer.write_float32(mat.m13)?;

    writer.write_float32(mat.m21)?;
    writer.write_float32(mat.m22)?;
    writer.write_float32(mat.m23)?;

    writer.write_float32(mat.m31)?;
    writer.write_float32(mat.m32)?;
    writer.write_float32(mat.m33)?;

    Ok(())
}

pub (crate) fn load_rect(rect: &mut Rect, reader: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    rect.x = reader.read_float32()?;
    rect.y = reader.read_float32()?;
    rect.w = reader.read_float32()?;
    rect.h = reader.read_float32()?;

    Ok(())
}

pub (crate) fn save_rect(rect: &Rect, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_float32(rect.x)?;
    writer.write_float32(rect.y)?;
    writer.write_float32(rect.w)?;
    writer.write_float32(rect.h)?;

    Ok(())
}

pub (crate) fn load_sphere(sphere: &mut Sphere, reader: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    sphere.x = reader.read_float32()?;
    sphere.y = reader.read_float32()?;
    sphere.z = reader.read_float32()?;
    sphere.r = reader.read_float32()?;

    Ok(())
}

pub (crate) fn save_sphere(sphere: &Sphere, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_float32(sphere.x)?;
    writer.write_float32(sphere.y)?;
    writer.write_float32(sphere.z)?;
    writer.write_float32(sphere.r)?;

    Ok(())
}

pub (crate) fn load_vector2(vector: &mut Vector2, reader: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    vector.x = reader.read_float32()?;
    vector.y = reader.read_float32()?;

    Ok(())
}

pub (crate) fn save_vector2(vector: &Vector2, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_float32(vector.x)?;
    writer.write_float32(vector.y)?;

    Ok(())
}

pub (crate) fn load_vector3(vector: &mut Vector3, reader: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    vector.x = reader.read_float32()?;
    vector.y = reader.read_float32()?;
    vector.z = reader.read_float32()?;

    Ok(())
}

pub (crate) fn save_vector3(vector: &Vector3, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_float32(vector.x)?;
    writer.write_float32(vector.y)?;
    writer.write_float32(vector.z)?;

    Ok(())
}

pub (crate) fn load_quat(quat: &mut Quat, reader: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    quat.x = reader.read_float32()?;
    quat.y = reader.read_float32()?;
    quat.z = reader.read_float32()?;
    quat.w = reader.read_float32()?;

    Ok(())
}

pub (crate) fn save_quat(quat: &Quat, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_float32(quat.x)?;
    writer.write_float32(quat.y)?;
    writer.write_float32(quat.z)?;
    writer.write_float32(quat.w)?;

    Ok(())
}

pub (crate) fn save_quat_packed(quat: &Quat, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    for v in [quat.x, quat.y, quat.z, quat.w] {
        // Convert to signed short and write
        let s = (v * 32767.0).round() as i16;
        writer.write_int16(s)?;
    }

    Ok(())
}