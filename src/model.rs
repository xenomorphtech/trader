// model.rs
use rand::Rng;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Clone, Debug)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub timestamp: f64,
    pub index: usize,
}

impl Candle {
    pub fn new(price: f64, timestamp: f64, index: usize) -> Self {
        Self {
            open: price,
            high: price,
            low: price,
            close: price,
            timestamp,
            index,
        }
    }

    pub fn update(&mut self, price: f64) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
    }
}

pub struct MarketData {
    pub price: f64,
    pub candles_1m: Vec<Candle>,
    pub candles_5m: Vec<Candle>,
    pub current_1m: Candle,
    pub current_5m: Candle,
    pub window_size: usize,
    last_1m_time: f64,
    last_5m_time: f64,
    last_update_time: f64,
}

impl Default for MarketData {
    fn default() -> Self {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let mut rng = rand::thread_rng();
        let mut price: f64 = 100.0;
        let mut candles_1m = Vec::new();
        let mut candles_5m = Vec::new();

        // Generate 60 historical 1-minute candles
        for i in 0..60 {
            let start_price = price;
            let mut high: f64 = price;
            let mut low: f64 = price;

            // Simulate price movement within the candle
            for _ in 0..10 {
                price += rng.gen_range(-1.0..1.0);
                high = f64::max(high, price);
                low = f64::min(low, price);
            }

            let candle = Candle {
                open: start_price,
                high,
                low,
                close: price,
                timestamp: current_time - (60 - i) as f64 * 60.0, // Proper minute timestamps
                index: i,
            };
            candles_1m.push(candle);
        }

        // Generate 12 historical 5-minute candles (60 minutes / 5)
        for i in 0..12 {
            let chunk_size = 5;
            let start_idx = i * chunk_size;
            let end_idx = start_idx + chunk_size;
            let chunk = &candles_1m[start_idx..end_idx];

            let candle = Candle {
                open: chunk[0].open,
                high: chunk.iter().map(|c| c.high).fold(f64::MIN, f64::max),
                low: chunk.iter().map(|c| c.low).fold(f64::MAX, f64::min),
                close: chunk[chunk.len() - 1].close,
                timestamp: chunk[0].timestamp,
                index: i,
            };
            candles_5m.push(candle);
        }

        // Round current time to the nearest minute
        let rounded_time = (current_time / 60.0).floor() * 60.0;

        Self {
            price,
            candles_1m,
            candles_5m,
            current_1m: Candle::new(price, rounded_time, 60),
            current_5m: Candle::new(price, rounded_time, 12),
            window_size: 100,
            last_1m_time: rounded_time,
            last_5m_time: rounded_time,
            last_update_time: current_time,
        }
    }
}

impl MarketData {
    pub fn update(&mut self, new_price: f64) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        // Rate limit price updates to 100ms
        if current_time - self.last_update_time < 0.1 {
            return;
        }
        self.last_update_time = current_time;
        self.price = new_price;

        // Update current candles
        self.current_1m.update(new_price);
        self.current_5m.update(new_price);

        // Check if 1-minute candle should close (real minute interval)
        if current_time - self.last_1m_time >= 60.0 {
            let completed_1m = self.current_1m.clone();
            let next_index = if self.candles_1m.is_empty() {
                0
            } else {
                self.candles_1m.last().unwrap().index + 1
            };

            self.candles_1m.push(completed_1m);
            if self.candles_1m.len() > self.window_size {
                self.candles_1m.remove(0);
            }

            // Round to the nearest minute
            let new_minute_time = (current_time / 60.0).floor() * 60.0;
            self.current_1m = Candle::new(new_price, new_minute_time, next_index + 1);
            self.last_1m_time = new_minute_time;
        }

        // Check if 5-minute candle should close (real 5-minute interval)
        if current_time - self.last_5m_time >= 300.0 {
            let completed_5m = self.current_5m.clone();
            let next_index = if self.candles_5m.is_empty() {
                0
            } else {
                self.candles_5m.last().unwrap().index + 1
            };

            self.candles_5m.push(completed_5m);
            if self.candles_5m.len() > self.window_size {
                self.candles_5m.remove(0);
            }

            // Round to the nearest 5 minutes
            let new_5min_time = (current_time / 300.0).floor() * 300.0;
            self.current_5m = Candle::new(new_price, new_5min_time, next_index + 1);
            self.last_5m_time = new_5min_time;
        }
    }
}
