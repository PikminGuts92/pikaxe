

#[cfg(test)]
mod tests {
    use rstest::*;
    use super::*;

    #[rstest]
    fn calc_world_space_transform() {
        use nalgebra as na;

        // Need to find local matrix that produces same computed world matrix
        // Gltf space
        let orig_anim_transform = na::Translation3::new(1.0, 2.0, 3.0).to_homogeneous()
            * na::Rotation3::from_axis_angle(&na::Vector3::y_axis(), std::f32::consts::FRAC_PI_2).to_homogeneous() // 90 degrees on y-axis
            * na::Scale3::new(5.0, 10.0, 15.0).to_homogeneous();

        let node_transforms = [
            crate::model::MILOSPACE_TO_GLSPACE,
            na::Rotation3::from_axis_angle(&na::Vector3::x_axis(), std::f32::consts::PI).to_homogeneous(), // 180 degrees on x-axis
            na::Translation3::new(30.0, 20.0, -40.0).to_homogeneous(),
        ];

        let gltf_parent_world_transform = node_transforms
            .iter()
            .fold(na::Matrix4::identity(), |acc, t| acc * t);

        let gltf_parent_world_transform_inverse = gltf_parent_world_transform.try_inverse().unwrap();

        // Milo space
        let new_anim_transform = gltf_parent_world_transform_inverse
            * crate::model::MILOSPACE_TO_GLSPACE
            * gltf_parent_world_transform
            * orig_anim_transform;

        // Both should equal each other
        let orig_milo_world_anim_transform = crate::model::MILOSPACE_TO_GLSPACE
            * gltf_parent_world_transform
            * orig_anim_transform;

        let new_milo_world_anim_transform = gltf_parent_world_transform * new_anim_transform;

        assert_eq!(orig_milo_world_anim_transform, new_milo_world_anim_transform);
    }
}