use std::f32::NAN;

use syntect::highlighting::Color;

pub fn css(c: &Color) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        format!("rgba({}, {}, {}, {:02})", c.r, c.g, c.b, (c.a as f32 / 255.0))
    }
}

// RGB <--> HSL conversions lifted from:
// https://stackoverflow.com/questions/2353211/hsl-to-rgb-color-conversion
//
fn hsl_to_rgb(hue: f32, sat: f32, lum: f32) -> (u8, u8, u8) {
    let r;
    let g;
    let b;

    let hue_to_rgb = |p, q, mut t| {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0/6.0 { return p + (q - p) * 6.0 * t; }
        if t < 1.0/2.0 { return q; }
        if t < 2.0/3.0 { return p + (q - p) * (2.0/3.0 - t) * 6.0; }
        p
    };

    if sat == 0.0 {
        r = lum;
        g = lum;
        b = lum;
    } else {
        let q = if lum < 0.5 { lum * (1.0 + sat) } else { lum + sat - lum * sat };
        let p = 2.0 * lum - q;
        r = hue_to_rgb(p, q, hue + 1.0/3.0);
        g = hue_to_rgb(p, q, hue);
        b = hue_to_rgb(p, q, hue - 1.0/3.0);
    }

    ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
}

fn rgb_to_hsl(r0: u8, g0: u8, b0: u8) -> (f32, f32, f32) {
    let r = r0 as f32 / 255.0;
    let g = g0 as f32 / 255.0;
    let b = b0 as f32 / 255.0;

    let max = [r, g, b].iter().cloned().fold(NAN, f32::max);
    let min = [r, g, b].iter().cloned().fold(NAN, f32::min);
    let mut hue = (max + min) / 2.0;
    let lum = (max + min) / 2.0;
    let sat;

    if max == min {
        hue = 0.0;
        sat = 0.0;
    } else {
        let delta = max - min;

        sat = if lum > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };

        if max == r { hue = (g - b) / delta + if g < b { 6.0 } else { 0.0 }; }
        if max == g { hue = (b - r) / delta + 2.0; }
        if max == b { hue = (r - g) / delta + 4.0; }

        hue /= 6.0;
    }

    (hue, sat, lum)
}

// manipulate a color's saturation / luminance
fn adjust(color: &Color, s_factor: f32, l_factor: f32) -> Color {
    let hsl = rgb_to_hsl(color.r, color.g, color.b);

    let sat = hsl.1 * s_factor;
    let lum = if hsl.2 > 0.0 { hsl.2 * l_factor } else { 0.1 };
    let rgb = hsl_to_rgb(hsl.0, sat, lum);

    Color {
        r: rgb.0,
        g: rgb.1,
        b: rgb.2,
        a: color.a,
    }
}

// make a color lighter by increasing luminance (+ maybe some saturation fix)
pub fn lighten(color: &Color, sat_factor: f32, lum_factor: f32) -> Color {
    adjust(color, sat_factor, lum_factor)
}

// make a color darker by decreasing luminance (+ maybe some saturation fix)
pub fn darken(color: &Color, sat_factor: f32, lum_factor: f32) -> Color {
    adjust(color, sat_factor, lum_factor)
}

// adjust a Color's alpha by a percentage
pub fn alpha(color: &Color, alpha_perc: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a as f32 * alpha_perc).round() as u8,
    }
}

// consider a color "light" if luminance > 40%
pub fn is_light(color: &Color) -> bool {
    rgb_to_hsl(color.r, color.g, color.b).2 > 0.40
}
