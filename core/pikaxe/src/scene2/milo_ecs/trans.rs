use bevy_ecs::prelude::*;
use pikaxe_derive::{autotrait, milo};
use pyo3::{prelude::*, types::PyType};
use super::{Object, ObjectComponent, ObjectNamedPointer, ObjectPython, ObjectTyped};

#[derive(Default, Clone)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

#[allow(non_camel_case_types)]
#[derive(Default, Clone, Copy)]
#[repr(u32)]
pub enum TransConstraint {
    #[default] kConstraintNone,
    kConstraintLocalRotate,
    kConstraintParentWorld,
    kConstraintLookAtTarget,
    kConstraintShadowTarget,
    kConstraintBillboardZ,
    kConstraintBillboardXZ,
    kConstraintBillboardXYZ,
    kConstraintFastBillboardXYZ
}

impl From<u32> for TransConstraint {
    fn from(num: u32) -> TransConstraint {
        match num {
            0 => TransConstraint::kConstraintNone,
            1 => TransConstraint::kConstraintLocalRotate,
            2 => TransConstraint::kConstraintParentWorld,
            3 => TransConstraint::kConstraintLookAtTarget,
            4 => TransConstraint::kConstraintShadowTarget,
            5 => TransConstraint::kConstraintBillboardZ,
            6 => TransConstraint::kConstraintBillboardXZ,
            7 => TransConstraint::kConstraintBillboardXYZ,
            8 => TransConstraint::kConstraintFastBillboardXYZ,
            // Default
            _ => TransConstraint::kConstraintNone,
        }
    }
}

#[derive(Default, Clone, Component)]
#[require(ObjectComponent)]
#[autotrait(extends=Object)]
pub struct TransComponent {
    pub local_xfm: Transform,
    pub world_xfm: Transform,
    pub trans_objects: Vec<ObjectNamedPointer>,
    pub constraint: TransConstraint,
    pub target: ObjectNamedPointer,
    pub preserve_scale: bool,
    pub parent: ObjectNamedPointer,
}

#[milo(class_name="Trans")]
#[derive(Default, Clone, Bundle)]
pub struct TransInstance {
    pub(crate) object: ObjectComponent,
    pub(crate) trans: TransComponent,
}

impl From<TransInstance> for Box<dyn Trans> {
    fn from(value: TransInstance) -> Self {
        Box::new(value)
    }
}

impl From<TransInstance> for ObjectTyped {
    fn from(value: TransInstance) -> Self {
        ObjectTyped::Trans(value.into())
    }
}