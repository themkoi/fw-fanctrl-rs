use std::collections::HashMap;

use crate::fan_config::*;

pub fn default_fan_config() -> FanConfig {
    let mut strategies = HashMap::new();

    strategies.insert(
        "laziest".to_string(),
        Strategy {
            fan_speed_update_frequency: 3.0,
            moving_average_interval: 40,
            speed_curve: vec![
                SpeedPoint { temp: 0.0, speed: 0.0 },
                SpeedPoint { temp: 45.0, speed: 0.0 },
                SpeedPoint { temp: 65.0, speed: 25.0 },
                SpeedPoint { temp: 70.0, speed: 35.0 },
                SpeedPoint { temp: 75.0, speed: 50.0 },
                SpeedPoint { temp: 85.0, speed: 100.0 },
            ],
        },
    );

    strategies.insert(
        "lazy".to_string(),
        Strategy {
            fan_speed_update_frequency: 2.0,
            moving_average_interval: 30,
            speed_curve: vec![
                SpeedPoint { temp: 0.0, speed: 15.0 },
                SpeedPoint { temp: 50.0, speed: 15.0 },
                SpeedPoint { temp: 65.0, speed: 25.0 },
                SpeedPoint { temp: 70.0, speed: 35.0 },
                SpeedPoint { temp: 75.0, speed: 50.0 },
                SpeedPoint { temp: 85.0, speed: 100.0 },
            ],
        },
    );

    strategies.insert(
        "medium".to_string(),
        Strategy {
            fan_speed_update_frequency: 1.0,
            moving_average_interval: 30,
            speed_curve: vec![
                SpeedPoint { temp: 0.0, speed: 15.0 },
                SpeedPoint { temp: 40.0, speed: 15.0 },
                SpeedPoint { temp: 60.0, speed: 30.0 },
                SpeedPoint { temp: 70.0, speed: 40.0 },
                SpeedPoint { temp: 75.0, speed: 80.0 },
                SpeedPoint { temp: 85.0, speed: 100.0 },
            ],
        },
    );

    strategies.insert(
        "agile".to_string(),
        Strategy {
            fan_speed_update_frequency: 0.5,
            moving_average_interval: 15,
            speed_curve: vec![
                SpeedPoint { temp: 0.0, speed: 15.0 },
                SpeedPoint { temp: 40.0, speed: 15.0 },
                SpeedPoint { temp: 60.0, speed: 30.0 },
                SpeedPoint { temp: 70.0, speed: 40.0 },
                SpeedPoint { temp: 75.0, speed: 80.0 },
                SpeedPoint { temp: 85.0, speed: 100.0 },
            ],
        },
    );

    strategies.insert(
        "very-agile".to_string(),
        Strategy {
            fan_speed_update_frequency: 0.5,
            moving_average_interval: 5,
            speed_curve: vec![
                SpeedPoint { temp: 0.0, speed: 15.0 },
                SpeedPoint { temp: 40.0, speed: 15.0 },
                SpeedPoint { temp: 60.0, speed: 30.0 },
                SpeedPoint { temp: 70.0, speed: 40.0 },
                SpeedPoint { temp: 75.0, speed: 80.0 },
                SpeedPoint { temp: 85.0, speed: 100.0 },
            ],
        },
    );

    strategies.insert(
        "deaf".to_string(),
        Strategy {
            fan_speed_update_frequency: 0.5,
            moving_average_interval: 5,
            speed_curve: vec![
                SpeedPoint { temp: 0.0, speed: 20.0 },
                SpeedPoint { temp: 40.0, speed: 30.0 },
                SpeedPoint { temp: 50.0, speed: 50.0 },
                SpeedPoint { temp: 60.0, speed: 100.0 },
            ],
        },
    );

    strategies.insert(
        "aeolus".to_string(),
        Strategy {
            fan_speed_update_frequency: 0.5,
            moving_average_interval: 5,
            speed_curve: vec![
                SpeedPoint { temp: 0.0, speed: 20.0 },
                SpeedPoint { temp: 40.0, speed: 50.0 },
                SpeedPoint { temp: 65.0, speed: 100.0 },
            ],
        },
    );

    FanConfig {
        default_strategy: "lazy".to_string(),
        strategy_on_discharging: "".to_string(),
        strategies,
    }
}