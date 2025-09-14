use crate::indicators::indicator_trait::{BarData, Indicator};

#[derive(Debug)]
pub struct PivotPoints {
    previous_high: Option<f64>,
    previous_low: Option<f64>,
    previous_close: Option<f64>,
    current_pivot: Option<f64>,
    current_r1: Option<f64>,
    current_r2: Option<f64>,
    current_s1: Option<f64>,
    current_s2: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct PivotOutput {
    pub pivot: f64,
    pub r1: f64,
    pub r2: f64,
    pub s1: f64,
    pub s2: f64,
}

impl PivotPoints {
    pub fn new() -> Self {
        Self {
            previous_high: None,
            previous_low: None,
            previous_close: None,
            current_pivot: None,
            current_r1: None,
            current_r2: None,
            current_s1: None,
            current_s2: None,
        }
    }

    pub fn get_levels(&self) -> Option<PivotOutput> {
        if let (Some(pivot), Some(r1), Some(r2), Some(s1), Some(s2)) = (
            self.current_pivot,
            self.current_r1,
            self.current_r2,
            self.current_s1,
            self.current_s2,
        ) {
            Some(PivotOutput {
                pivot,
                r1,
                r2,
                s1,
                s2,
            })
        } else {
            None
        }
    }
}

impl Default for PivotPoints {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for PivotPoints {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "PivotPoints"
    }

    fn warm_up_period(&self) -> usize {
        1
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        if let (Some(prev_high), Some(prev_low), Some(prev_close)) =
            (self.previous_high, self.previous_low, self.previous_close)
        {
            let pivot = (prev_high + prev_low + prev_close) / 3.0;
            let r1 = 2.0 * pivot - prev_low;
            let r2 = pivot + (prev_high - prev_low);
            let s1 = 2.0 * pivot - prev_high;
            let s2 = pivot - (prev_high - prev_low);

            self.current_pivot = Some(pivot);
            self.current_r1 = Some(r1);
            self.current_r2 = Some(r2);
            self.current_s1 = Some(s1);
            self.current_s2 = Some(s2);
        }

        self.previous_high = Some(input.high);
        self.previous_low = Some(input.low);
        self.previous_close = Some(input.close);

        self.current_pivot
    }

    fn current(&self) -> Option<f64> {
        self.current_pivot
    }

    fn reset(&mut self) {
        self.previous_high = None;
        self.previous_low = None;
        self.previous_close = None;
        self.current_pivot = None;
        self.current_r1 = None;
        self.current_r2 = None;
        self.current_s1 = None;
        self.current_s2 = None;
    }
}
