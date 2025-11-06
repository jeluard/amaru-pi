use crate::timeseries::TimeSeries;

const MAX_DATA_POINTS: usize = 100;
const SMA_WINDOW: usize = 10;

#[derive(Debug, Clone)]
pub struct MetricData {
    pub data_type: String,
    pub time_series_raw: TimeSeries,
    pub time_series_smoothed: TimeSeries,
    pub current_value: f64,
    pub x_counter: u64,
    /// The sum of the last SMA_WINDOW raw data points.
    raw_data_rolling_sum: f64,
}

impl MetricData {
    pub fn new(data_type: String, value: f64) -> Self {
        // Initialize TimeSeries for raw data
        let mut time_series_raw = TimeSeries::new(MAX_DATA_POINTS);
        time_series_raw.add_point((0.0, value));

        // Initialize TimeSeries for smoothed data
        let mut time_series_smoothed = TimeSeries::new(MAX_DATA_POINTS);
        // The first smoothed point is just the value itself
        time_series_smoothed.add_point((0.0, value));

        Self {
            data_type,
            time_series_raw,
            time_series_smoothed,
            current_value: value,
            x_counter: 0,
            // The initial sum is just the first value.
            raw_data_rolling_sum: value,
        }
    }

    /// Adds a new value to the raw series and calculates the new smoothed value
    /// efficiently using a rolling sum.
    pub fn add_value(&mut self, value: f64) {
        self.current_value = value;
        self.x_counter += 1;
        let x = self.x_counter as f64;

        // Add the new raw data point
        self.time_series_raw.add_point((x, value));

        // Calculate and add the new smoothed data point
        let sma_value = self.calculate_sma(value);
        self.time_series_smoothed.add_point((x, sma_value));
    }

    /// Calculates the Simple Moving Average efficiently using a rolling sum.
    fn calculate_sma(&mut self, new_value: f64) -> f64 {
        let raw_data_slice = self.time_series_raw.data();
        let window_len = raw_data_slice.len();

        // Add the new value to our rolling sum
        self.raw_data_rolling_sum += new_value;

        // Determine the number of items that should be in our average
        let sma_len = window_len.min(SMA_WINDOW);

        // If the window is "full", we need to subtract the value that just
        // fell off the back of the SMA window.
        if window_len > SMA_WINDOW {
            let item_to_remove_index = window_len - 1 - SMA_WINDOW;
            let (_, oldest_value_to_remove) = raw_data_slice[item_to_remove_index];
            self.raw_data_rolling_sum -= oldest_value_to_remove;
        }

        // Calculate the new average
        self.raw_data_rolling_sum / (sma_len as f64)
    }
}
