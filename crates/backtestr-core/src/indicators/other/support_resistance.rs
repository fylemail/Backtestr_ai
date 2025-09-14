use std::collections::VecDeque;
use crate::indicators::indicator_trait::{BarData, Indicator};

#[derive(Debug)]
pub struct SupportResistance {
    period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    current_resistance: Option<f64>,
    current_support: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct SupportResistanceOutput {
    pub support: f64,
    pub resistance: f64,
}

impl SupportResistance {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
            current_resistance: None,
            current_support: None,
        }
    }

    pub fn get_levels(&self) -> Option<SupportResistanceOutput> {
        if let (Some(support), Some(resistance)) = (self.current_support, self.current_resistance) {
            Some(SupportResistanceOutput { support, resistance })
        } else {
            None
        }
    }
}

impl Indicator for SupportResistance {
    type Input = BarData;
    type Output = f64;

    fn name(&self) -> &str {
        "SupportResistance"
    }

    fn warm_up_period(&self) -> usize {
        self.period
    }

    fn update(&mut self, input: BarData) -> Option<f64> {
        self.highs.push_back(input.high);
        self.lows.push_back(input.low);

        if self.highs.len() > self.period {
            self.highs.pop_front();
            self.lows.pop_front();
        }

        if self.highs.len() == self.period {
            let resistance = self.highs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let support = self.lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));

            self.current_resistance = Some(resistance);
            self.current_support = Some(support);

            Some((resistance + support) / 2.0)
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        if let (Some(s), Some(r)) = (self.current_support, self.current_resistance) {
            Some((s + r) / 2.0)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.current_resistance = None;
        self.current_support = None;
    }
}