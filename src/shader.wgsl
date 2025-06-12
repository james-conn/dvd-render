@group(0) @binding(0) var<storage, read> idx_grid: array<u32>;
@group(0) @binding(1) var<storage, read> atlas: array<u32>;
@group(0) @binding(2) var<storage, read_write> output_img: array<u32>;
@group(0) @binding(3) var<uniform> grid_width: u32;
@group(0) @binding(4) var<uniform> grid_height: u32;
@group(0) @binding(5) var<uniform> img_width: u32;
@group(0) @binding(6) var<uniform> img_height: u32;
@group(0) @binding(7) var<storage, read> color_grid: array<u32>;

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

fn sample_colors(gidx: u32) -> vec2<u32> {
	return vec2(color_grid[gidx * 2], color_grid[(gidx * 2) + 1]);
}

fn atlas_val_at(aidx: u32, rg_pos: vec2<u32>, glyph_size: vec2<u32>) -> u32 {
	let ax = rg_pos.x;
	let ay = (aidx * glyph_size.y) + rg_pos.y;
	let abidx = (ay * glyph_size.x) + ax;
	let qidx = abidx / 4;
	let n_byte = abidx % 4;

	return unpack4xU8(atlas[qidx])[n_byte];
}

// lerp between `a` and `b` from `t=0` to `t=255`
fn qlerp(a: vec4<u32>, b: vec4<u32>, t: u32) -> vec4<u32> {
	let a1 = a * t;
	let b1 = b * (255 - t);

	return (a1 + b1) / 255;
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

	let cov = atlas_val_at(aidx, rg_pos, glyph_size);
	let cols = sample_colors(gidx);
	let fg_col = unpack4xU8(cols[0]);
	let bg_col = unpack4xU8(cols[1]);
	output_img[img_idx] = pack4xU8(qlerp(fg_col, bg_col, cov));
}
