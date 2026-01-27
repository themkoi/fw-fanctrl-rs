use crate::fan_config::*;
use std::collections::VecDeque;

pub struct FanController {
    buffer: VecDeque<f32>,
}

impl FanController {
    pub fn new(strategy: &Strategy) -> Self {
        Self {
            buffer: VecDeque::with_capacity(strategy.moving_average_interval as usize),
        }
    }

    pub fn update(&mut self, temperature: f32, strategy: &Strategy) -> f32 {
        if self.buffer.is_empty() {
            self.buffer.push_back(temperature);
            return self.interpolate(temperature, strategy);
        }

        let prev = *self.buffer.back().unwrap();
        let alpha = 1.0 / strategy.moving_average_interval.max(1) as f32;
        let smooth = prev + alpha * (temperature - prev);

        self.buffer.push_back(smooth);
        if self.buffer.len() > strategy.moving_average_interval as usize {
            self.buffer.pop_front();
        }

        self.interpolate(smooth, strategy)
    }

    fn interpolate(&self, temperature: f32, strategy: &Strategy) -> f32 {
        let points = &strategy.speed_curve;
        if points.is_empty() {
            return 0.0;
        }

        if temperature <= points[0].temp {
            return points[0].speed;
        }
        if temperature >= points[points.len() - 1].temp {
            return points[points.len() - 1].speed;
        }

        for i in 0..points.len() - 1 {
            let a = &points[i];
            let b = &points[i + 1];
            if temperature >= a.temp && temperature <= b.temp {
                let t = (temperature - a.temp) / (b.temp - a.temp);
                return a.speed + t * (b.speed - a.speed);
            }
        }

        points.last().unwrap().speed
    }
}
