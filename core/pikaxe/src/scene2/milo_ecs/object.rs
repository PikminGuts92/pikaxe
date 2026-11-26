use bevy_ecs::prelude::*;
use pikaxe_derive::autotrait;
use pyo3::{prelude::*, types::PyType};
use super::ObjectTyped;

#[derive(Default, Clone, Component)]
//#[autotrait(extends=Default + Clone + Bundle)]
pub struct ObjectComponent {
    pub id: u32,
    pub note: String,
}

pub trait Object { // Default + Clone + Bundle
    fn get_class_name(&self) -> &'static str;

    fn get_object_component(&self) -> &ObjectComponent;
    fn get_object_component_mut(&mut self) -> &mut ObjectComponent;

    fn get_id(&self) -> u32 {
        self.get_object_component().id
    }

    fn set_id(&mut self, value: u32) {
        self.get_object_component_mut().id = value;
    }

    fn get_note(&self) -> &String {
        &self.get_object_component().note
    }

    fn get_note_mut(&mut self) -> &mut String {
        &mut self.get_object_component_mut().note
    }

    fn set_note(&mut self, value: String) {
        self.get_object_component_mut().note = value;
    }
}

pub(crate) trait LoadObjectFromQuery : Object + Sized {
    fn load_from_query_with_id(world: &mut World, entity: Entity) -> Option<Self>;
}

#[derive(Default, Clone, Bundle)]
//#[pyclass(name="Object", subclass)]
pub struct ObjectInstance {
    pub(crate) object: ObjectComponent,
}

impl LoadObjectFromQuery for ObjectInstance {
    fn load_from_query_with_id(world: &mut World, entity: Entity) -> Option<Self> {
        let mut query = world.query::<super::ObjectQuery>(); // TODO: Move query to same file
        let Ok(obj_data) = query.get(world, entity) else {
            return None;
        };

        let dyn_obj: Box<dyn Object> = Box::new(Self {
            object: obj_data.object.to_owned()
        });

        let class_name = dyn_obj.get_class_name();

        Some(Self {
            object: obj_data.object.to_owned()
        })
    }
}

impl Object for ObjectInstance {
    fn get_class_name(&self) -> &'static str {
        "Object"
    }

    fn get_object_component(&self) -> &ObjectComponent {
        &self.object
    }

    fn get_object_component_mut(&mut self) -> &mut ObjectComponent {
        &mut self.object
    }
}

impl From<ObjectInstance> for Box<dyn Object> {
    fn from(value: ObjectInstance) -> Self {
        Box::new(value)
    }
}

impl From<ObjectInstance> for ObjectTyped {
    fn from(value: ObjectInstance) -> Self {
        ObjectTyped::Object(value.into())
    }
}

// Python bindings

#[derive(Default)]
#[pyclass(name="Object", subclass)]
pub struct ObjectPython {
    pub value: Option<ObjectInstance>,
}

#[pymethods]
impl ObjectPython {
    // TODO: Possibly only initialize from MiloEngine/ObjectDir create_object() method...
    #[new]
    fn new() -> Self {
        Self {
            value: Some(Default::default())
        }
    }

    #[getter]
    fn get_note(&self) -> String {
        self.value.as_ref().unwrap().object.note.to_owned()
    }

    #[setter]
    fn set_note(&mut self, value: String) {
        self.value.as_mut().unwrap().object.note = value;
    }
}