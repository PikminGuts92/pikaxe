use pikaxe::texture::{Bitmap, decode_dx_image, decode_from_bitmap, DXGI_Encoding};
use criterion::{Criterion, criterion_group, criterion_main};

fn benchmark_texture(c: &mut Criterion) {
    let widths = [32, 64, 128, 256, 512, 1024, 2048];

    // Bitmap
    let mut group = c.benchmark_group("decode_bitmap_8bpp");
    for width in widths {
        let bitmap = Bitmap {
            bpp: 8,
            raw_data: vec![0u8; 1024 + (width * width)], // Palette + pixel indicies
            ..Bitmap::new()
        };

        let mut rgba = vec![0u8; (width * width) * 4];

        group.bench_function(width.to_string().as_str(), |b| {
            b.iter(|| {
                decode_from_bitmap(
                    &bitmap,
                    &Default::default(),
                    rgba.as_mut_slice(),
                ).unwrap();
            });
        });
    }
    group.finish();

    // DXT1
    let mut group = c.benchmark_group("decode_dxt1");
    for width in widths {
        let packed_data = vec![0u8; (width * width) / 2]; // 4bpp
        let mut rgba = vec![0u8; (width * width) * 4];

        group.bench_function(width.to_string().as_str(), |b| {
            b.iter(|| {
                decode_dx_image(
                    packed_data.as_slice(),
                    rgba.as_mut_slice(),
                    width as u32,
                    DXGI_Encoding::DXGI_FORMAT_BC1_UNORM,
                    false
                );
            });
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_texture);
criterion_main!(benches);