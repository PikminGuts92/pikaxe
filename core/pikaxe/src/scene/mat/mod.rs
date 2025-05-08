mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

#[milo]
pub struct MatObject {
    pub blend: Blend,
    pub color: Color3,
    pub alpha: f32,

    pub prelit: bool,
    pub use_environ: bool,

    pub z_mode: ZMode,
    pub alpha_cut: bool,
    pub alpha_threshold: u32,
    pub alpha_write: bool,

    pub tex_gen: TexGen,
    pub tex_wrap: TexWrap,

    pub tex_xfm: Matrix,
    pub diffuse_tex: String,

    pub next_pass: String,
    pub intensify: bool,
    pub cull: bool,

    pub emissive_multiplier: f32,
    pub specular_rgb: Color3,
    pub specular_power: f32,

    pub normal_map: String,
    pub emissive_map: String,
    pub specular_map: String,
    pub environ_map: String,

    pub per_pixel_lit: PerPixel,
    pub stencil_mode: StencilMode,

    pub fur: String,
    pub de_normal: f32,
    pub anisotropy: f32,

    pub norm_detail_tiling: f32,
    pub norm_detail_strength: f32,
    pub norm_detail_map: String,

    pub point_lights: bool,
    pub proj_lights: bool,
    pub fog: bool,
    pub fade_out: bool,
    pub color_adjust: bool,

    pub rim_rgb: Color3,
    pub rim_power: f32,
    pub rim_map: String,
    pub rim_always_show: bool,

    pub screen_aligned: bool,
    pub shader_variation: ShaderVariation,

    pub specular2_rgb: Color3,
    pub specular2_power: f32,

    pub unknown_1: f32,
    pub unknown_2: f32,
    pub unknown_3: f32,
    pub unknown_4: f32,
    pub unknown_5: f32,

    pub alpha_mask: String,
    pub ps3_force_trilinear: bool,
}

impl Default for MatObject {
    fn default() -> MatObject {
        // TODO: Match default values to C++ code
        MatObject {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // Mat object
            blend: Blend::default(),
            color: Color3::white(),
            alpha: 1.0,

            prelit: true,
            use_environ: false,

            z_mode: ZMode::default(),
            alpha_cut: false,
            alpha_threshold: 0,
            alpha_write: false,

            tex_gen: TexGen::default(),
            tex_wrap: TexWrap::kTexWrapRepeat,

            tex_xfm: Matrix::default(),
            diffuse_tex: String::default(),

            next_pass: String::default(),
            intensify: false,
            cull: true,

            emissive_multiplier: 0.0,
            specular_rgb: Color3::white(),
            specular_power: 1.0,

            normal_map: String::default(),
            emissive_map: String::default(),
            specular_map: String::default(),
            environ_map: String::default(),

            per_pixel_lit: PerPixel::default(),
            stencil_mode: StencilMode::default(),

            fur: String::default(),
            de_normal: 0.0,
            anisotropy: 0.0,

            norm_detail_tiling: 1.0,
            norm_detail_strength: 0.0,
            norm_detail_map: String::default(),

            point_lights: false,
            proj_lights: false,
            fog: false,
            fade_out: false,
            color_adjust: true,

            rim_rgb: Color3::white(),
            rim_power: 1.0,
            rim_map: String::default(),
            rim_always_show: false,

            screen_aligned: false,
            shader_variation: ShaderVariation::default(),

            specular2_rgb: Color3::white(),
            specular2_power: 1.0,

            unknown_1: 1.0,
            unknown_2: 1.0,
            unknown_3: 1.0,
            unknown_4: 1.0,
            unknown_5: 1.0,

            alpha_mask: String::default(),
            ps3_force_trilinear: false,
        }
    }
}