use super::{Matrix, MiloObject};

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

pub trait Trans : MiloObject {
    fn get_local_xfm(&self) -> &Matrix;
    fn get_local_xfm_mut(&mut self) -> &mut Matrix;
    fn set_local_xfm(&mut self, transform: Matrix);

    fn get_world_xfm(&self) -> &Matrix;
    fn get_world_xfm_mut(&mut self) -> &mut Matrix;
    fn set_world_xfm(&mut self, transform: Matrix);

    fn get_trans_objects(&self) -> &Vec<String>;
    fn get_trans_objects_mut(&mut self) -> &mut Vec<String>;
    fn set_trans_objects(&mut self, trans_objects: Vec<String>);

    fn get_constraint(&self) -> &TransConstraint;
    fn get_constraint_mut(&mut self) -> &mut TransConstraint;
    fn set_constraint(&mut self, constraint: TransConstraint);

    fn get_target(&self) -> &String;
    fn get_target_mut(&mut self) -> &mut String;
    fn set_target(&mut self, target: String);

    fn get_preserve_scale(&self) -> bool;
    fn set_preserve_scale(&mut self, preserve_scale: bool);

    fn get_parent(&self) -> &String;
    fn get_parent_mut(&mut self) -> &mut String;
    fn set_parent(&mut self, parent: String);
}