// view.rs
use crate::model::Candle;
use eframe::egui;
use egui::{Color32, Id, PointerButton, Pos2, Rect, Response};

struct ChartBounds {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

pub struct ChartView {
    id: String,
    padding: f32,
    visible_count: usize,
    offset: usize,
    last_candle_count: usize,
    drag_origin: Option<(Pos2, usize)>,
    auto_scroll: bool,
}

impl ChartView {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            padding: 20.0,
            visible_count: 30,
            offset: 0,
            last_candle_count: 0,
            drag_origin: None,
            auto_scroll: true,
        }
    }

    pub fn seek_to_latest(&mut self) {
        self.offset = 0;
        self.drag_origin = None;
        self.auto_scroll = true;
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        candles: &[Candle],
        current_candle: &Candle,
        title: &str,
    ) {
        if self.last_candle_count == 0 {
            self.seek_to_latest();
        }

        let total_candles = candles.len() + 1;
        if total_candles > self.last_candle_count && self.auto_scroll {
            self.offset = 0;
        }
        self.last_candle_count = total_candles;

        egui::Frame::group(ui.style()).show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(title);
                if ui.button("Latest").clicked() {
                    self.seek_to_latest();
                }
            });

            ui.add_space(4.0);

            // Chart area
            let chart_rect = ui.available_rect_before_wrap();
            let desired_size = egui::vec2(chart_rect.width(), 300.0);
            let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());

            self.handle_interactions(&response, candles.len());

            let bounds = self.calculate_bounds(candles, current_candle);
            let content_width = rect.width() - 2.0 * self.padding;
            let width_per_candle = content_width / self.visible_count as f32;
            let candle_body_width = width_per_candle * 0.8;

            // Draw the chart
            let painter = ui.painter();

            // Background
            painter.rect_filled(rect, 0.0, Color32::from_gray(20));

            // Price line
            let price_y =
                self.price_to_screen_y(current_candle.close, rect, bounds.y_min, bounds.y_max);
            painter.line_segment(
                [
                    Pos2::new(rect.min.x + self.padding, price_y),
                    Pos2::new(rect.max.x - self.padding, price_y),
                ],
                egui::Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 100)),
            );

            // Calculate visible candles
            let start_idx = candles
                .len()
                .saturating_sub(self.visible_count.saturating_sub(1) + self.offset);
            let end_idx = (start_idx + self.visible_count).min(candles.len());
            let visible_candles = &candles[start_idx..end_idx];

            // Draw candles
            for (plot_idx, candle) in visible_candles
                .iter()
                .chain(if self.offset == 0 {
                    Some(current_candle)
                } else {
                    None
                })
                .enumerate()
            {
                let x_center = rect.min.x
                    + self.padding
                    + (plot_idx as f32 * width_per_candle)
                    + (width_per_candle / 2.0);

                let color = if candle.close >= candle.open {
                    Color32::GREEN
                } else {
                    Color32::RED
                };

                let high_y = self.price_to_screen_y(candle.high, rect, bounds.y_min, bounds.y_max);
                let low_y = self.price_to_screen_y(candle.low, rect, bounds.y_min, bounds.y_max);
                let open_y = self.price_to_screen_y(candle.open, rect, bounds.y_min, bounds.y_max);
                let close_y =
                    self.price_to_screen_y(candle.close, rect, bounds.y_min, bounds.y_max);

                painter.line_segment(
                    [Pos2::new(x_center, high_y), Pos2::new(x_center, low_y)],
                    egui::Stroke::new(1.0, color),
                );

                let body_rect = Rect::from_min_max(
                    Pos2::new(x_center - candle_body_width / 2.0, open_y.min(close_y)),
                    Pos2::new(x_center + candle_body_width / 2.0, open_y.max(close_y)),
                );
                painter.rect_filled(body_rect, 0.0, color);
            }

            // Ensure proper layout
            ui.allocate_rect(rect, egui::Sense::hover());
        });
    }

    fn handle_interactions(&mut self, response: &Response, max_offset: usize) {
        let width_per_candle =
            (response.rect.width() - 2.0 * self.padding) / self.visible_count as f32;

        if response.drag_started()
            && response
                .ctx
                .input(|i| i.pointer.button_down(PointerButton::Primary))
        {
            self.drag_origin = response.hover_pos().map(|pos| (pos, self.offset));
            self.auto_scroll = false;
        }

        if let Some((origin_pos, origin_offset)) = self.drag_origin {
            if let Some(current_pos) = response.hover_pos() {
                let delta_x = current_pos.x - origin_pos.x;
                let candle_delta = (delta_x / width_per_candle).round() as i32;

                let new_offset = if candle_delta >= 0 {
                    origin_offset.saturating_add(candle_delta as usize)
                } else {
                    origin_offset.saturating_sub(candle_delta.unsigned_abs() as usize)
                };

                self.offset = new_offset.min(max_offset);
            }
        }

        if response.drag_released() {
            self.drag_origin = None;
        }
    }

    fn calculate_bounds(&self, candles: &[Candle], current_candle: &Candle) -> ChartBounds {
        let start_idx = candles
            .len()
            .saturating_sub(self.visible_count.saturating_sub(1) + self.offset);
        let end_idx = (start_idx + self.visible_count).min(candles.len());
        let visible_candles = &candles[start_idx..end_idx];

        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;

        for candle in visible_candles.iter().chain(if self.offset == 0 {
            Some(current_candle)
        } else {
            None
        }) {
            y_min = y_min.min(candle.low);
            y_max = y_max.max(candle.high);
        }

        let y_padding = (y_max - y_min) * 0.1;
        y_min -= y_padding;
        y_max += y_padding;

        let x_min = 0.0;
        let x_max = self.visible_count as f64;

        ChartBounds {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    fn price_to_screen_y(&self, price: f64, rect: Rect, y_min: f64, y_max: f64) -> f32 {
        let price_range = y_max - y_min;
        let normalized = (price - y_min) / price_range;
        let y_pixels = rect.height() - 2.0 * self.padding;
        rect.max.y - self.padding - (normalized * y_pixels as f64) as f32
    }
}
