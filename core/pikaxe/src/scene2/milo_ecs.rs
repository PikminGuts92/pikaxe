mod draw;
mod mesh;
mod object;
mod object_dir;
mod trans;

use bevy_ecs::{
    query::{QueryData, QueryFilter},
    prelude::*,
};
use draw::*;
use mesh::*;
use object::*;
use object_dir::*;
use std::marker::PhantomData;
use trans::*;

use pyo3::{prelude::*, types::PyType};

/*
MiloFile -> ObjectDirId
MiloFile -> ObjectEntry[]
ObjectDir -> MiloFile[] (inline subdirs)
ObjectEntry -> ObjectId

MiloFile
- revision
- entries
- object_dir_entry
- file_path

Possibly give MiloFile its own "name" property because it doesn't always match object dir
*/

#[derive(Default, Clone)]
pub struct ObjectNamedPointer { // TODO: Add generic type constraint?
    pub name: String,
    pub id: Option<u32>,
}

#[derive(Default, Clone)]
pub struct ObjectPointer<T: Object> {
    _marker: PhantomData<T>,
    pub id: Option<u32>,
}

#[derive(Default, Clone)]
pub struct ObjectNamedPointerTyped<T: Sized> {
    _marker: PhantomData<T>,
    pub name: String,
    pub id: Option<u32>,
}

impl<T: Object> ObjectNamedPointerTyped<T> {
    fn get_object(&self) -> T {
        todo!()
    }
}

pub enum ObjectTyped {
    Object(Box<dyn Object>),
    Trans(Box<dyn Trans>),
    ObjectDir(Box<dyn ObjectDir>),
}

pub enum ObjectDirTyped {
    ObjectDir(Box<dyn ObjectDir>),
}

#[derive(Component, Clone)] // TODO: Implement default?
pub enum ObjectTypeDefinition {
    // TODO: Should be able to add type definition restriction to ObjectNamedPointer
    Object,
    Trans,
    ObjectDir
}

impl ObjectTypeDefinition {
    /*pub(crate) fn get_query(&self) {
        let query = match self {
            ObjectTypeDefinition::Object => QueryState::new(world)
        };
    }*/

    pub(crate) fn get_object_from_query(&self, world: &mut World, entity: Entity) {
        match self {
            ObjectTypeDefinition::Object => {
                let query = world.query::<ObjectDirQuery>();

            },
            _ => {
                todo!()
            }
        }
    }
}

pub fn add_milo_classes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    //m.add_class::<ObjectPython>()?;
    //m.add_class::<ObjectDirPython>()?;

    Ok(())
}

#[derive(QueryData)]
struct ObjectQuery {
    entity: Entity,
    object: &'static ObjectComponent,
    type_definition: &'static ObjectTypeDefinition,
}

#[derive(QueryData)]
struct TransQuery {
    entity: Entity,
    object: &'static ObjectComponent,
    trans: &'static TransComponent,
}

#[derive(QueryData)]
struct ObjectDirQuery {
    entity: Entity,
    object: &'static ObjectComponent,
    object_dir: &'static ObjectDirComponent,
}

#[derive(Default, Clone)]
struct MiloFile {
    pub entries: Vec<ObjectNamedPointer>,
    //pub object: ObjectNamedPointerTyped<Box<dyn Object>>,
    pub dir_object: ObjectNamedPointer,
}

impl MiloFile {
    pub fn get_directory(&self, milo_engine: &mut MiloEnvironment) -> ObjectDirTyped {
        todo!()
    }

    pub fn get_entries(&self, milo_engine: &mut MiloEnvironment) -> impl Iterator<Item = ObjectTyped> {
        self.entries.iter().map(|entry| {
            // TODO: Replace unwraps with something...
            milo_engine.get_object_typed_by_id(entry.id.unwrap()).unwrap()
        })
    }
}

#[derive(Default)]
pub struct MiloEnvironment {
    world: World,
}

impl MiloEnvironment {
    // TODO: Extract query behaviors to another struct
    /*fn create_object<T: Object>(&mut self) -> T {
        let obj_entity = self
            .world
            .spawn_empty()
            .id();

        let mut obj = T::default();
        obj.set_id(obj_entity.index_u32());

        self
            .world
            .commands()
            .entity(obj_entity)
            .insert(obj.clone());

        obj
    }*/

    pub fn get_object_by_id(&mut self, id: u32) -> Option<ObjectInstance> {
        let entity = Entity::from_raw_u32(id).expect("Id is valid");
        //let entity = self.world.commands().entity(entity);

        let mut query = self.world.query::<ObjectQuery>();
        let Ok(obj_data) = query.get(&self.world, entity) else {
            return None;
        };

        let obj_instance = ObjectInstance {
            object: obj_data.object.to_owned()
        };

        //self.world.query()
        /*for obj_data in self.world.query::<ObjectQuery>().iter(&self.world) {
            let obj_instance = ObjectInstance {
                object: obj_data.object.to_owned()
            };
        }*/

        Some(obj_instance)
    }

    pub fn get_object_typed_by_id(&mut self, id: u32) -> Option<ObjectTyped> {
        let entity = Entity::from_raw_u32(id).expect("Id is valid");

        let mut obj_type_query = self.world.query::<&ObjectTypeDefinition>();
        let obj_type_def = obj_type_query.get(&self.world, entity).ok()?;

        self.get_object_typed_by_id_with_type(id, obj_type_def.clone())
    }

    pub fn get_object_typed_by_id_with_type(&mut self, id: u32, type_def: ObjectTypeDefinition) -> Option<ObjectTyped> {
        let entity = Entity::from_raw_u32(id).expect("Id is valid");

        let obj_typed: ObjectTyped = match &type_def {
            &ObjectTypeDefinition::Object => self
                .world.query::<ObjectQuery>()
                .get(&self.world, entity)
                .map(|obj| ObjectInstance {
                    object: obj.object.clone(),
                })
                .map(|t| t.into())
                .ok()?,
            &ObjectTypeDefinition::Trans => self
                .world.query::<TransQuery>()
                .get(&self.world, entity)
                .map(|obj| TransInstance {
                    object: obj.object.clone(),
                    trans: obj.trans.clone(),
                })
                .map(|t| t.into())
                .ok()?,
            _ => todo!()
        };

        Some(obj_typed)
    }

    pub fn get_object_dir_typed_by_id(&mut self, id: u32) -> Option<ObjectDirTyped> {
        todo!()
    }
}

// TODO: Move to common.rs file?

#[derive(Debug, Default, Clone)]
pub struct Sphere {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
}

#[derive(Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug)]
pub struct Color3 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color3 {
    pub fn white() -> Color3 {
        Color3 {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }
}

impl Default for Color3 {
    fn default() -> Color3 {
        Color3::white()
    }
}

#[derive(Debug)]
pub struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4 {
    pub fn white() -> Color4 {
        Color4 {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Default for Color4 {
    fn default() -> Color4 {
        Color4::white()
    }
}

#[derive(Debug, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl PartialEq for Vector3 {
    fn eq(&self, other: &Self) -> bool {
        are_floats_equal(self.x, other.x)
            && are_floats_equal(self.y, other.y)
            && are_floats_equal(self.z, other.z)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
}

impl PartialEq for Quat {
    fn eq(&self, other: &Self) -> bool {
        are_floats_equal(self.x, other.x)
            && are_floats_equal(self.y, other.y)
            && are_floats_equal(self.z, other.z)
            && are_floats_equal(self.w, other.w)
    }
}

#[derive(Debug)]
pub struct Matrix {
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,
    pub m14: f32,
    pub m21: f32,
    pub m22: f32,
    pub m23: f32,
    pub m24: f32,
    pub m31: f32,
    pub m32: f32,
    pub m33: f32,
    pub m34: f32,
    pub m41: f32,
    pub m42: f32,
    pub m43: f32,
    pub m44: f32,
}

impl Matrix {
    pub fn identity() -> Matrix {
        Matrix {
            m11: 1.0,
            m12: 0.0,
            m13: 0.0,
            m14: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 0.0,
            m24: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
            m34: 0.0,
            m41: 0.0,
            m42: 0.0,
            m43: 0.0,
            m44: 1.0,
        }
    }
}

impl Default for Matrix {
    fn default() -> Matrix {
        Matrix::identity()
    }
}

#[derive(Debug)]
pub struct Matrix3 {
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,
    pub m21: f32,
    pub m22: f32,
    pub m23: f32,
    pub m31: f32,
    pub m32: f32,
    pub m33: f32,
}

impl Matrix3 {
    pub fn identity() -> Matrix3 {
        Matrix3 {
            m11: 1.0,
            m12: 0.0,
            m13: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
        }
    }
}

impl Default for Matrix3 {
    fn default() -> Matrix3 {
        Matrix3::identity()
    }
}

fn are_floats_equal(a: f32, b: f32) -> bool {
    (a - b).abs() < f32::EPSILON
}
