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
        let fan_speed: f32 = self.interpolate(temperature, strategy);

        // add to buffer
        self.buffer.push_back(fan_speed);
        if self.buffer.len() > strategy.moving_average_interval as usize {
            self.buffer.pop_front();
        }

        // trimmed moving average like Smooth rule
        let mut values: Vec<f32> = self.buffer.iter().copied().collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = values.len();
        if len == 0 {
            return 0.0;
        }

        let trimmed: &[f32] = if len > 2 {
            let cut = std::cmp::max(1, len / 5);
            &values[cut..len - cut]
        } else {
            &values[..]
        };
        let avg: f32 = trimmed.iter().sum::<f32>() / trimmed.len() as f32;
        avg
    }

    fn interpolate(&self, temperature: f32, strategy: &Strategy) -> f32 {
        if strategy.speed_curve.is_empty() {
            return 0.0;
        }

        let points = &strategy.speed_curve;
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
