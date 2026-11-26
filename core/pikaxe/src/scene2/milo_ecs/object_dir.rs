use bevy_ecs::prelude::*;
use pikaxe_derive::{autotrait, milo};
use pyo3::{prelude::*, types::PyType};
use super::{Object, ObjectComponent, ObjectNamedPointer, ObjectPython};

#[derive(Default, Clone, Component)]
#[require(ObjectComponent)]
#[autotrait(extends=Object)]
pub struct ObjectDirComponent {
    pub entries: Vec<ObjectNamedPointer>,
}

#[milo(class_name="ObjectDir")]
#[derive(Default, Clone, Bundle)]
pub struct ObjectDirInstance {
    pub(crate) object: ObjectComponent,
    pub(crate) object_dir: ObjectDirComponent,
}

// Python bindings

#[derive(Default)]
#[pyclass(name="ObjectDir", subclass, extends=ObjectPython)]
pub struct ObjectDirPython {
    pub value: Option<ObjectDirInstance>,
}

#[pymethods]
impl ObjectDirPython {
    #[new]
    fn new() -> PyClassInitializer<Self> {
        PyClassInitializer::from(ObjectPython::default())
            .add_subclass(Self {
                value: Some(Default::default())
            })
    }

    #[getter]
    fn get_note(&self) -> String {
        self.value.as_ref().unwrap().object.note.to_owned()
    }

    #[setter]
    fn set_note(&mut self, value: String) {
        self.value.as_mut().unwrap().object.note = value;
    }

    /*#[getter]
    fn get_entries(&self) -> String {
        //self.value.as_ref().unwrap().object.note.to_owned()
    }

    #[setter]
    fn set_entries(&mut self, value: String) {
        //self.value.as_mut().unwrap().object.note = value;
    }*/
}