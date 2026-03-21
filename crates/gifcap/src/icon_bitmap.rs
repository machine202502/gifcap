//! Shared RGBA for app icon (egui + Windows .ico via build.rs).

fn lerp_u8(a: f32, b: f32, t: f32) -> u8 {
    (a + (b - a) * t.clamp(0.0, 1.0)).round() as u8
}

/// Signed distance to a rounded box; `c` = center, `h` = positive half-extents, `r` corner radius.
fn sd_rounded_box(px: f32, py: f32, cx: f32, cy: f32, hx: f32, hy: f32, r: f32) -> f32 {
    let p = ((px - cx).abs(), (py - cy).abs());
    let q = (p.0 - hx + r, p.1 - hy + r);
    let w = (q.0.max(0.0), q.1.max(0.0));
    w.0.hypot(w.1) + q.0.min(0.0).min(q.1) - r
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Sea-breeze vertical gradient + Instagram-style outline camera (white on blue).
pub fn icon_rgba(size: u32) -> Vec<u8> {
    let n = size.max(32) as f32;
    let stroke = (n * 0.065).clamp(2.0, n * 0.11);
    let aa = (stroke * 0.42).max(0.6);

    let mut out = vec![0u8; (size * size * 4) as usize];

    let cx = 0.5 * n;
    let cy = 0.52 * n;
    let body_hx = 0.28 * n;
    let body_hy = 0.22 * n;
    let body_r = 0.14 * n;
    let shrink = stroke * 0.92;
    let body_in_hx = (body_hx - shrink).max(1.0);
    let body_in_hy = (body_hy - shrink).max(1.0);
    let body_in_r = (body_r - shrink * 0.9).max(1.0);

    let lens_cx = cx - 0.14 * n;
    let lens_cy = cy - 0.02 * n;
    let lens_ro = 0.20 * n;

    let dot_cx = cx + 0.20 * n;
    let dot_cy = cy - 0.20 * n;
    let dot_r = 0.052 * n;

    for y in 0..size {
        let py = y as f32 + 0.5;
        let t = y as f32 / (size.saturating_sub(1).max(1) as f32);
        for x in 0..size {
            let px = x as f32 + 0.5;
            let i = ((y * size + x) * 4) as usize;

            let br = lerp_u8(200.0, 58.0, t);
            let bg = lerp_u8(236.0, 172.0, t);
            let bb = lerp_u8(252.0, 212.0, t);

            let d_o = sd_rounded_box(px, py, cx, cy, body_hx, body_hy, body_r);
            let d_i = sd_rounded_box(px, py, cx, cy, body_in_hx, body_in_hy, body_in_r);
            let inside_o = 1.0 - smoothstep(-aa, aa, d_o);
            let outside_i = smoothstep(-aa, aa, d_i);
            let body = (inside_o * outside_i).clamp(0.0, 1.0);

            let dist_l = ((px - lens_cx).powi(2) + (py - lens_cy).powi(2)).sqrt();
            let lens = smoothstep(stroke * 0.55, 0.0, (dist_l - lens_ro).abs()).clamp(0.0, 1.0);

            let dist_d = ((px - dot_cx).powi(2) + (py - dot_cy).powi(2)).sqrt();
            let dot = (1.0 - smoothstep(dot_r - aa, dot_r + aa, dist_d)).clamp(0.0, 1.0);

            let mut w = body.max(lens).max(dot);
            if w < 1.0 {
                w = w.max(0.0);
            }

            let fr = lerp_u8(br as f32, 255.0, w);
            let fg = lerp_u8(bg as f32, 255.0, w);
            let fb = lerp_u8(bb as f32, 255.0, w);
            out[i..i + 4].copy_from_slice(&[fr, fg, fb, 255]);
        }
    }
    out
}
