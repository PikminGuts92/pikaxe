use gltf_json as json;
use itertools::*;
use serde::ser::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BufferType {
    Mesh,
    Skin,
    Animation
}

pub struct AccessorBuilder {
    // Key = stride, Value = (idx, data)
    working_data: HashMap<(usize, BufferType), (usize, Vec<u8>)>,
    accessors: Vec<json::Accessor>,
}

impl AccessorBuilder {
    pub fn new() -> AccessorBuilder {
        AccessorBuilder {
            working_data: Default::default(),
            accessors: Vec::new()
        }
    }

    fn calc_stride<const N: usize, T: ComponentValue>(&self) -> usize {
        N * T::size()
    }

    fn update_buffer_view<const N: usize, T: ComponentValue>(&mut self, mut data: Vec<u8>, buffer_type: BufferType) -> (usize, usize) {
        let stride = self.calc_stride::<N, T>();
        let data_size = data.len();
        let next_idx = self.working_data.len();

        // Upsert buffer data
        let (idx, buff) = self.working_data
            .entry((stride, buffer_type))
            .and_modify(|(_, b)| b.append(&mut data))
            .or_insert_with(|| (next_idx, data));

        // Return index of updated buffer view + insert offset
        (*idx, buff.len() - data_size)
    }

    pub fn add_scalar<S: Into<String>, T: ComponentValue, U: IntoIterator<Item = T>>(&mut self, name: S, data: U, buffer_type: BufferType) -> Option<usize> {
        // Map to iter of single-item arrays (definitely hacky)
        self.add_array(name, data.into_iter().map(|d| [d]), buffer_type)
    }

    pub fn add_array<const N: usize, S: Into<String>, T: ComponentValue, U: IntoIterator<Item = V>, V: Into<[T; N]>>(&mut self, name: S, data: U, buffer_type: BufferType) -> Option<usize> {
        let comp_type = T::get_component_type();

        let acc_type = match N {
            1 => json::accessor::Type::Scalar,
            2 => json::accessor::Type::Vec2,
            3 => json::accessor::Type::Vec3,
            4 => json::accessor::Type::Vec4,
            9 => json::accessor::Type::Mat3,
            16 => json::accessor::Type::Mat4,
            _ => unimplemented!()
        };

        // Write to stream and find min/max values
        let mut data_stream = Vec::new();
        let (count, min, max) = data
            .into_iter()
            .fold((0usize, [T::max(); N], [T::min(); N]), |(count, mut min, mut max), item| {
                let mut i = 0;
                for v in item.into() {
                    // Encode + append each value to master buffer
                    data_stream.append(&mut v.encode());

                    // Calc min + max values
                    min[i] = min[i].get_min(v);
                    max[i] = max[i].get_max(v);

                    i += 1;
                }

                (count + 1, min, max)
            });

        if count == 0 {
            // If count is 0, don't bother adding
            return None;
        }

        // Update buffer views
        let (buff_idx, buff_off) = self.update_buffer_view::<N, T>(data_stream, buffer_type);

        let acc_index = self.accessors.len();

        let (min_value, max_value) = Self::get_min_max_values(
            &acc_type,
            min,
            max
        ).unwrap();

        // Create accessor
        let accessor = json::Accessor {
            buffer_view: Some(json::Index::new(buff_idx as u32)),
            byte_offset: Some(buff_off.into()),
            count: count.into(),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(comp_type)),
            extensions: None,
            extras: Default::default(),
            type_: json::validation::Checked::Valid(acc_type),
            min: Some(min_value),
            max: Some(max_value),
            name: match name.into() {
                s if !s.is_empty() => Some(s),
                _ => None
            },
            normalized: false,
            sparse: None
        };

        self.accessors.push(accessor);
        Some(acc_index)
    }

    pub fn get_array_by_name_mut<T: ComponentValue, const N: usize>(&mut self, name: &str, buffer_type: BufferType) -> Option<&mut [[T; N]]> {
        let stride = self.calc_stride::<N, T>();

        // TODO: Validate com_type and stride?
        let (_bv_idx, b_off, count, _com_type) = self
            .accessors
            .iter()
            .find(|acc| acc.name.as_ref().map(|n| n.eq(name)).unwrap_or_default())
            .map(|acc| (
                acc.buffer_view.as_ref().map(|bv| bv.value()).unwrap(),
                acc.byte_offset.as_ref().map(|bo| bo.0 as usize).unwrap(),
                acc.count.0 as usize,
                *acc.component_type.as_ref().unwrap()
            ))?;

        let (_buf_idx_wd, buffer) = self
            .working_data
            .get_mut(&(stride, buffer_type))?;

        let data = &mut buffer[b_off..(b_off + (stride * count))];

        //let res: &mut [f32] = cast_slice_mut(data);
        //let mut bytes: [u8; 7] = [1, 2, 3, 4, 5, 6, 7];

        let (_prefix, new_data, _suffix) = unsafe { data.align_to_mut::<[T; N]>() };
        Some(new_data)
    }

    pub fn recalc_min_max_values<T: ComponentValue, const N: usize>(&mut self, name: &str, buffer_type: BufferType) {
        let stride = self.calc_stride::<N, T>();

        // TODO: Validate com_type and stride?
        let Some((_bv_idx, b_off, count, _com_type)) = self
            .accessors
            .iter()
            .find(|acc| acc.name.as_ref().map(|n| n.eq(name)).unwrap_or_default())
            .map(|acc| (
                acc.buffer_view.as_ref().map(|bv| bv.value()).unwrap(),
                acc.byte_offset.as_ref().map(|bo| bo.0 as usize).unwrap(),
                acc.count.0 as usize,
                *acc.component_type.as_ref().unwrap()
            )) else {
                return;
            };

        let Some((_buf_idx_wd, buffer)) = self
            .working_data
            .get_mut(&(stride, buffer_type)) else {
                return;
            };

        let data = &mut buffer[b_off..(b_off + (stride * count))];
        let (_prefix, new_data, _suffix) = unsafe { data.align_to::<[T; N]>() };

        let (min, max) = new_data
            .iter()
            .fold(([T::max(); N], [T::min(); N]), |(mut min, mut max), item| {
                let mut i = 0;
                for v in item.iter() {
                    // Calc min + max values
                    min[i] = min[i].get_min(*v);
                    max[i] = max[i].get_max(*v);

                    i += 1;
                }

                (min, max)
            });

        let acc_type = match N {
            1 => json::accessor::Type::Scalar,
            2 => json::accessor::Type::Vec2,
            3 => json::accessor::Type::Vec3,
            4 => json::accessor::Type::Vec4,
            9 => json::accessor::Type::Mat3,
            16 => json::accessor::Type::Mat4,
            _ => unimplemented!()
        };

        let (min_value, max_value) = Self::get_min_max_values(
            &acc_type,
            min,
            max
        ).unwrap();

        // Update accessor
        let accessor = self
            .accessors
            .iter_mut()
            .find(|acc| acc.name.as_ref().map(|n| n.eq(name)).unwrap_or_default());

        if let Some(acc) = accessor {
            acc.min = Some(min_value);
            acc.max = Some(max_value);
        }
    }

    fn generate_buffer_views(&mut self) -> (Vec<json::buffer::View>, Vec<u8>) {
        // Get view info and sort by assigned index
        let view_data = self.working_data
            .drain()
            .map(|(k, (idx, data))| (idx, k, data)) // (idx, stride, data)
            .sorted_by(|(a, ..), (b, ..)| a.cmp(b));

        let mut views = Vec::new();
        let mut all_data = Vec::new();

        for (_idx, (stride, buffer_type), mut data) in view_data {
            // Pad buffer view if required
            let padded_size = align_to_multiple_of_four(data.len());
            if padded_size > data.len() {
                let diff_size = padded_size - data.len();
                data.append(&mut vec![0u8; diff_size]);
            }

            let data_size = data.len();
            let data_offset = all_data.len();

            // Move data from view to full buffer
            all_data.append(&mut data);

            views.push(json::buffer::View {
                name: None,
                byte_length: data_size.into(),
                byte_offset: Some(data_offset.into()),
                byte_stride: match (stride, buffer_type) {
                    (_, bt) if bt.eq(&BufferType::Animation) || bt.eq(&BufferType::Skin) => None,
                    (s, _) if s % 4 == 0 => Some(json::buffer::Stride(stride)),
                    _ => None // Don't encode if not multiple
                },
                buffer: json::Index::new(0),
                target: None,
                extensions: None,
                extras: Default::default()
            });
        }

        (views, all_data)
    }

    pub fn generate<T: Into<String>>(mut self, name: T) -> (Vec<json::Accessor>, Vec<json::buffer::View>, json::Buffer, Vec<u8>) {
        // Generate buffer views + final buffer blob
        let (views, buffer_data) = self.generate_buffer_views();

        // Create buffer json
        let buffer = json::Buffer {
            name: None,
            byte_length: buffer_data.len().into(),
            uri: match name.into() {
                s if !s.is_empty() => Some(s),
                _ => None
            },
            extensions: None,
            extras: Default::default()
        };

        // Return everything
        (self.accessors,
            views,
            buffer,
            buffer_data)
    }

    fn get_min_max_values<const N: usize, T: ComponentValue>(acc_type: &json::accessor::Type, min: [T; N], max: [T; N]) -> Option<(json::Value, json::Value)> {
        let result = match acc_type {
            json::accessor::Type::Scalar => (
                json::serialize::to_value([min.iter().fold(T::max(), |acc, m| acc.get_min(*m))]),
                json::serialize::to_value([max.iter().fold(T::min(), |acc, m| acc.get_max(*m))]),
            ),
            _ => (
                json::serialize::to_value(min.to_vec()),
                json::serialize::to_value(max.to_vec()),
            ),
        };

        match result {
            (Ok(min), Ok(max)) => Some((min, max)),
            _ => None
        }
    }
}

pub trait ComponentValue : Copy + Serialize {
    fn min() -> Self;
    fn max() -> Self;

    fn get_min(self, other: Self) -> Self;
    fn get_max(self, other: Self) -> Self;

    fn encode(self) -> Vec<u8>;
    fn get_component_type() -> json::accessor::ComponentType;

    fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl ComponentValue for u16 {
    fn min() -> Self {
        u16::MIN
    }

    fn max() -> Self {
        u16::MAX
    }

    fn get_min(self, other: Self) -> Self {
        std::cmp::min(self, other)
    }

    fn get_max(self, other: Self) -> Self {
        std::cmp::max(self, other)
    }

    fn encode(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn get_component_type() -> json::accessor::ComponentType {
        json::accessor::ComponentType::U16
    }
}

impl ComponentValue for f32 {
    fn min() -> Self {
        f32::MIN
    }

    fn max() -> Self {
        f32::MAX
    }

    fn get_min(self, other: Self) -> Self {
        f32::min(self, other)
    }

    fn get_max(self, other: Self) -> Self {
        f32::max(self, other)
    }

    fn encode(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn get_component_type() -> json::accessor::ComponentType {
        json::accessor::ComponentType::F32
    }
}

fn align_to_multiple_of_four(n: usize) -> usize {
    (n + 3) & !3
}