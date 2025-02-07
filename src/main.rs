use eframe::egui;
use rand::Rng;
mod model;
mod view;

use model::MarketData;
use view::ChartView;

struct TradingApp {
    market_data: MarketData,
    chart_1m: ChartView,
    chart_5m: ChartView,
}

impl Default for TradingApp {
    fn default() -> Self {
        Self {
            market_data: MarketData::default(),
            chart_1m: ChartView::new("1m_chart"),
            chart_5m: ChartView::new("5m_chart"),
        }
    }
}

impl eframe::App for TradingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut rng = rand::thread_rng();
        let price_change = rng.gen_range(-2.0..2.0);
        let new_price = self.market_data.price + price_change;

        self.market_data.update(new_price);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Trading Platform Demo - Multi-timeframe Charts");
                ui.label(format!("Current Price: ${:.2}", self.market_data.price));

                self.chart_1m.show(
                    ui,
                    &self.market_data.candles_1m,
                    &self.market_data.current_1m,
                    "1 Minute Chart",
                );

                self.chart_5m.show(
                    ui,
                    &self.market_data.candles_5m,
                    &self.market_data.current_5m,
                    "5 Minute Chart",
                );
            });
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Trading Platform",
        native_options,
        Box::new(|_cc| Box::new(TradingApp::default())),
    )
}
