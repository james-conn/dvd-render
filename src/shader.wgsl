@group(0) @binding(0) var<storage, read> idx_grid: array<u32>;
@group(0) @binding(1) var<storage, read> atlas: array<u32>;
@group(0) @binding(2) var<storage, read_write> output_img: array<u32>;
@group(0) @binding(3) var<uniform> grid_width: u32;
@group(0) @binding(4) var<uniform> grid_height: u32;
@group(0) @binding(5) var<uniform> img_width: u32;
@group(0) @binding(6) var<uniform> img_height: u32;

fn img_idx(x: u64, y: u64) -> u64 {
	return (y * u64(img_width)) + x;
}

fn grid_idx(img_pos: vec2<u32>) -> u32 {
	let gx = (img_pos.x * grid_width) / img_width;
	let gy = (img_pos.y * grid_height) / img_height;

	return (gy * grid_width) + gx;
}

fn rel_grid_pos(img_pos: vec2<u32>, glyph_size: vec2<u32>) -> vec2<u32> {
	return img_pos % glyph_size;
}

fn atlas_idx(gidx: u32) -> u32 {
	return idx_grid[gidx];
}

fn atlas_val_at(aidx: u32, rg_pos: vec2<u32>, glyph_size: vec2<u32>) -> u32 {
	let ax = rg_pos.x;
	let ay = (aidx * glyph_size.y) + rg_pos.y;
	let abidx = (ay * glyph_size.x) + ax;
	let qidx = abidx / 4;
	let n_byte = abidx % 4;

	return unpack4xU8(atlas[qidx])[n_byte];
}

@compute @workgroup_size(16, 16, 1)
fn sample_atlas(@builtin(global_invocation_id) global_id: vec3<u32>) {
	if (global_id.x >= img_width || global_id.y >= img_height) {
		return;
	}

	let img_pos = vec2(global_id.x, global_id.y);
	let glyph_size = vec2(
		img_width / grid_width,
		img_height / grid_height
	);

	let img_idx = img_idx(u64(global_id.x), u64(global_id.y));
	let gidx = grid_idx(img_pos);
	let rg_pos = rel_grid_pos(img_pos, glyph_size);
	let aidx = atlas_idx(gidx);

	let c = atlas_val_at(aidx, rg_pos, glyph_size);
	output_img[img_idx] = pack4xU8(vec4(c, c, c, 255));
}
