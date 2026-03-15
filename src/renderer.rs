use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
    Frame,
};
use crate::config::{ColorMode, RgbColor as CfgColor};

const BLOCKS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn lerp_color(a: CfgColor, b: CfgColor, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::Rgb(
        (a.0 as f32 + (b.0 as f32 - a.0 as f32) * t) as u8,
        (a.1 as f32 + (b.1 as f32 - a.1 as f32) * t) as u8,
        (a.2 as f32 + (b.2 as f32 - a.2 as f32) * t) as u8,
    )
}

// ── Wave renderer (existing continuous wave mode) ────────────────────────────

pub struct WaveRenderer<'a> {
    points: &'a [f32],
    color_mode: &'a ColorMode,
    color1: CfgColor,
    color2: CfgColor,
    visual_gain: f32,
}

impl<'a> WaveRenderer<'a> {
    fn sample(&self, t: f32) -> f32 {
        let n = self.points.len();
        let idx = t * (n - 1) as f32;
        let lo = idx.floor() as usize;
        let hi = (lo + 1).min(n - 1);
        let frac = idx - lo as f32;
        self.points[lo] * (1.0 - frac) + self.points[hi] * frac
    }
}

impl<'a> Widget for WaveRenderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = area.width as usize;
        let mid = area.top() as f32 + area.height as f32 / 2.0;
        let half = (area.height / 2) as f32;
        let bottom = area.bottom().saturating_sub(1);

        for screen_x in 0..width {
            let t = screen_x as f32 / (width - 1).max(1) as f32;
            let phys = (self.sample(t) * self.visual_gain).clamp(-1.0, 1.0);

            let intensity = phys.abs();
            let color = match self.color_mode {
                ColorMode::Solid    => lerp_color(self.color1, self.color1, 0.0),
                ColorMode::Gradient => lerp_color(self.color1, self.color2, intensity),
            };
            let style = Style::default().fg(color);

            let bar_top_f = mid - phys * half;
            let bar_top = bar_top_f.floor() as u16;
            let frac = bar_top_f.fract();
            let block_idx = ((1.0 - frac) * 8.0).round() as usize;

            let fill_start = (bar_top + 1).min(bottom);
            for y in fill_start..=bottom {
                buf.get_mut(area.left() + screen_x as u16, y)
                    .set_char('█')
                    .set_style(style);
            }
            if bar_top <= bottom && block_idx > 0 && block_idx < 9 {
                buf.get_mut(area.left() + screen_x as u16, bar_top)
                    .set_char(BLOCKS[block_idx])
                    .set_style(style);
            }
        }
    }
}

// ── Cava-style bars renderer ──────────────────────────────────────────────────
//
// Draws N evenly-spaced vertical bars (each `bar_width` columns wide) separated
// by single-column gaps. Bar heights are sampled from the physics wave points.
// Sub-character vertical precision is achieved with Unicode block elements.

pub struct BarsRenderer<'a> {
    points: &'a [f32],
    color_mode: &'a ColorMode,
    color1: CfgColor,
    color2: CfgColor,
    visual_gain: f32,
}

impl<'a> BarsRenderer<'a> {
    /// Linearly interpolate the physics array at a fractional index `t ∈ [0, 1]`.
    fn sample(&self, t: f32) -> f32 {
        let n = self.points.len();
        let idx = t * (n - 1) as f32;
        let lo = idx.floor() as usize;
        let hi = (lo + 1).min(n - 1);
        let frac = idx - lo as f32;
        self.points[lo] * (1.0 - frac) + self.points[hi] * frac
    }
}

impl<'a> Widget for BarsRenderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width  = area.width  as usize;
        let height = area.height as usize;
        if width == 0 || height == 0 { return; }

        let bottom  = area.bottom().saturating_sub(1);
        // phys=0 maps to the bottom row; phys=1 maps to the top.
        // bar_top_f = bottom - phys * (height - 1)
        // Fill solid blocks from (bar_top+1) down to bottom, partial block at bar_top.
        let half = (height as f32) / 2.0;
        let mid  = area.top() as f32 + half;

        let gap    = 1usize;
        let bar_w  = 2usize;
        let stride = bar_w + gap;
        let n_bars = ((width + gap) / stride).max(1);

        for bar in 0..n_bars {
            let t = if n_bars == 1 { 0.5 } else { bar as f32 / (n_bars - 1) as f32 };
            // No physics-space clamp — let visual_gain push bars all the way to
            // the screen edge. Screen-space clamping below handles overflow.
            // Divide by 10.0 to normalise physics amplitude so visual_gain=1
            // fills roughly 0–100% at typical loudness.
            let phys = self.sample(t) / 5.0 * self.visual_gain;

            let color = match self.color_mode {
                ColorMode::Solid    => lerp_color(self.color1, self.color1, 0.0),
                ColorMode::Gradient => lerp_color(self.color1, self.color2, phys.abs().min(1.0)),
            };
            let style = Style::default().fg(color);

            // Clamp in screen-space so bars fill to the terminal edge cleanly.
            let bar_top_f = (mid - phys * half)
                .clamp(area.top() as f32, area.bottom() as f32);
            let bar_top = bar_top_f.floor() as u16;
            let frac = bar_top_f.fract();
            let block_idx = ((1.0 - frac) * 8.0).round() as usize;

            let x_left = area.left() as usize + bar * stride;

            for col in 0..bar_w {
                let x = x_left + col;
                if x >= area.left() as usize + width { break; }
                let x = x as u16;

                let fill_start = (bar_top + 1).min(bottom);
                for y in fill_start..=bottom {
                    buf.get_mut(x, y).set_char('█').set_style(style);
                }
                if bar_top <= bottom && block_idx > 0 && block_idx < 9 {
                    buf.get_mut(x, bar_top)
                        .set_char(BLOCKS[block_idx])
                        .set_style(style);
                }
            }
        }
    }
}

// ── Top-level Renderer ────────────────────────────────────────────────────────

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self { Self }

    /// Cava-style spaced bars.
    pub fn draw_bars(
        &self,
        frame: &mut Frame,
        points: &[f32],
        color_mode: &ColorMode,
        color1: CfgColor,
        color2: CfgColor,
        visual_gain: f32,
    ) {
        let area = frame.size();
        frame.render_widget(BarsRenderer { points, color_mode, color1, color2, visual_gain }, area);
    }

    /// Continuous wave display.
    pub fn draw(
        &self,
        frame: &mut Frame,
        points: &[f32],
        color_mode: &ColorMode,
        color1: CfgColor,
        color2: CfgColor,
        visual_gain: f32,
    ) {
        let area = frame.size();
        frame.render_widget(WaveRenderer { points, color_mode, color1, color2, visual_gain }, area);
    }

    pub fn draw_debug(&self, frame: &mut Frame, input: &[f32]) {
        use ratatui::{style::{Color, Style}, text::Span};

        let area = frame.size();
        let buf = frame.buffer_mut();
        let n = input.len();
        let width = area.width as usize;
        let height = area.height as usize;
        let bottom = area.bottom().saturating_sub(1);

        for i in 0..n {
            let x_start = (i * width) / n;
            let x_end = ((i + 1) * width) / n;
            let val = input[i].clamp(0.0, 1.0);
            let bar_rows = (val * (height - 2) as f32) as u16;
            let bar_top = bottom.saturating_sub(bar_rows);
            let color = Color::Rgb((val * 180.0) as u8, (val * 210.0) as u8, 255);
            let style = Style::default().fg(color);

            for x in x_start..x_end {
                for y in bar_top..=bottom {
                    buf.get_mut(area.left() + x as u16, y).set_char('█').set_style(style);
                }
            }
            let label = format!("{i}");
            let lx = area.left() + x_start as u16;
            if lx + label.len() as u16 <= area.right() {
                buf.set_span(lx, bottom, &Span::styled(label, Style::default().fg(Color::White)), (x_end - x_start) as u16);
            }
        }

        let header = format!(" DEBUG — {} driver bars (q to quit) ", n);
        buf.set_span(area.left(), area.top(), &Span::styled(
            header, Style::default().fg(Color::Black).bg(Color::Yellow)
        ), area.width);
    }
}
