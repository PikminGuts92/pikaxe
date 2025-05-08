mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

#[milo(RndMesh)]
#[milo_super(Draw, Trans)]
pub struct MeshObject {
    // Using until packed next gen verts are figured out
    pub raw_vertices: Vec<[u8; 36]>
}

impl Default for MeshObject {
    fn default() -> MeshObject {
        MeshObject {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // Trans object
            local_xfm: Matrix::default(),
            world_xfm: Matrix::default(),

            trans_objects: Vec::new(),

            constraint: TransConstraint::default(),
            target: String::default(),

            preserve_scale: false,
            parent: String::default(),

            // Draw object
            showing: true,
            draw_objects: Vec::new(),
            sphere: Sphere::default(),
            draw_order: 0.0,
            override_include_in_depth_only_pass: OverrideIncludeInDepthOnlyPass::default(),

            // Mesh object
            mat: String::default(),
            geom_owner: String::default(),

            mutable: Mutable::kMutableNone,
            volume: Volume::kVolumeTriangles,

            vertices: Vec::new(),
            raw_vertices: Vec::new(),
            faces: Vec::new(),

            face_groups: Vec::new(),
            bones: Vec::new(),
            keep_mesh_data: false,
            exclude_from_self_shadow: false,
            has_ao_calculation: false,
        }
    }
}

impl MeshObject {
    pub fn recompute_face_groups(&mut self) {
        self.face_groups.clear();

        let mut face_count = self.faces.len() as u32;

        while face_count > 0 {
            if face_count < 255 {
                self.face_groups.push(face_count as u8);
                break;
            }

            self.face_groups.push(255);
            face_count -= 255;
        }
    }
}