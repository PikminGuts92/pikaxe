#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum RotationConstraint {
    kRotNone = 9,
    kRotFull = 2,
    kRotX = 3,
    kRotY = 4,
    kRotZ = 5
}

impl From<u32> for RotationConstraint {
    fn from(num: u32) -> RotationConstraint {
        match num {
            9 => RotationConstraint::kRotNone,
            2 => RotationConstraint::kRotFull,
            3 => RotationConstraint::kRotX,
            4 => RotationConstraint::kRotY,
            5 => RotationConstraint::kRotZ,
            // Default
            _ => RotationConstraint::kRotNone,
        }
    }
}