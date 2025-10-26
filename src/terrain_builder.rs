use noise::{NoiseFn, Perlin};

pub fn terrain_builder(seed: u32, height: usize) -> impl Fn(usize, usize) -> usize {
    let perlin = Perlin::new(seed);

    // Height is the approximate "maximum" elevation of terrain variation.
    // We'll use it as the amplitude range.
    move |x: usize, z: usize| {
        let scale = 0.01; // smaller = larger hills / smoother terrain
        let amplitude = height as f64 / 2.0; // vertical variation (half up, half down)
        let base = amplitude; // midline, so terrain stays roughly within 0..height

        // Noise value is between -1.0 and +1.0
        let height_value = perlin.get([x as f64 * scale, z as f64 * scale]) * amplitude + base;

        // Return true if voxel is below the terrain surface
        height_value as usize
    }
}
