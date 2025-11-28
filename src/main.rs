use std::env;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process::exit;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::fan_config::Strategy;
use crate::fan_control::FanController;
use log::{debug, error, info, warn};

const SOCK_PATH: &str = "/tmp/fw-fanctrl-rs.sock";

mod fan_config;
mod fan_control;

#[derive(Debug)]
struct TempParsed {
    f75303_local: Option<u32>,
    f75303_cpu: Option<u32>,
    f75303_ddr: Option<u32>,
    apu: Option<u32>,
    dgpu_vr: Option<u32>,
    dgpu_vram: Option<u32>,
    dgpu_amb: Option<u32>,
    dgpu_temp: Option<u32>,
    fan_speeds: Vec<u32>,
}

fn parse_temp(input: &str) -> TempParsed {
    let mut out = TempParsed {
        f75303_local: None,
        f75303_cpu: None,
        f75303_ddr: None,
        apu: None,
        dgpu_vr: None,
        dgpu_vram: None,
        dgpu_amb: None,
        dgpu_temp: None,
        fan_speeds: Vec::new(),
    };

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let to_val = |s: &str| s.parse::<u32>().ok();

        if let Some(v) = line.strip_prefix("F75303_Local:") {
            out.f75303_local = to_val(v.split_whitespace().next().unwrap_or(""));
        } else if let Some(v) = line.strip_prefix("F75303_CPU:") {
            out.f75303_cpu = to_val(v.split_whitespace().next().unwrap_or(""));
        } else if let Some(v) = line.strip_prefix("F75303_DDR:") {
            out.f75303_ddr = to_val(v.split_whitespace().next().unwrap_or(""));
        } else if let Some(v) = line.strip_prefix("APU:") {
            out.apu = to_val(v.split_whitespace().next().unwrap_or(""));
        } else if let Some(v) = line.strip_prefix("dGPU VR:") {
            out.dgpu_vr = to_val(v.split_whitespace().next().unwrap_or(""));
        } else if let Some(v) = line.strip_prefix("dGPU VRAM:") {
            out.dgpu_vram = to_val(v.split_whitespace().next().unwrap_or(""));
        } else if let Some(v) = line.strip_prefix("dGPU AMB:") {
            out.dgpu_amb = to_val(v.split_whitespace().next().unwrap_or(""));
        } else if let Some(v) = line.strip_prefix("dGPU temp:") {
            out.dgpu_temp = v.contains("NotPowered").then(|| 0).or_else(|| to_val(v));
        } else if let Some(v) = line.strip_prefix("Fan Speed:") {
            if let Some(num) = to_val(v.split_whitespace().next().unwrap_or("")) {
                out.fan_speeds.push(num);
            }
        }
    }

    out
}

fn run_daemon() -> std::io::Result<()> {
    if Path::new(SOCK_PATH).exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "daemon already running",
        ));
    }

    let listener = UnixListener::bind(SOCK_PATH)?;
    fs::set_permissions(SOCK_PATH, fs::Permissions::from_mode(0o666))?;

    let config = fan_config::load_or_create_config().unwrap();
    let strategy_name = Arc::new(Mutex::new(config.default_strategy.clone()));
    let current_strategy = Arc::new(Mutex::new(Strategy {
        fan_speed_update_frequency: 2.0,
        moving_average_interval: 30,
        speed_curve: vec![],
    }));

    {
        let name = strategy_name.lock().unwrap();
        let mut profile = current_strategy.lock().unwrap();
        *profile = config
            .strategies
            .get(&*name)
            .expect("Missing default")
            .clone();
    }
    let controller = {
        let profile = current_strategy.lock().unwrap(); // lock temporarily
        FanController::new(&profile) // use profile
    }; // lock is automatically dropped here
    let controller = Arc::new(Mutex::new(controller));

    loop {
        let profile_fan_clone = Arc::clone(&current_strategy);
        let strategy_name_clone = Arc::clone(&strategy_name);
        let controller_clone = Arc::clone(&controller);

        let _fan_thread = thread::spawn(move || loop {
            let sleep_time;

            {
                let profile = profile_fan_clone.lock().unwrap();
                let name = strategy_name_clone.lock().unwrap();
                sleep_time = profile.fan_speed_update_frequency;

                debug!("Update freq: {}", profile.fan_speed_update_frequency);
                debug!("Strategy: {}", *name);

                let temp = Command::new("framework_tool")
                    .arg("--thermal")
                    .output()
                    .expect("framework_tool failed");

                let stdout = String::from_utf8_lossy(&temp.stdout);
                let parsed = parse_temp(&stdout);
                debug!("{:?}", parsed);
                let temperature: f32;
                if parsed.apu > parsed.dgpu_temp {
                    temperature = parsed.apu.map(|v| v as f32).unwrap_or(0.0);
                } else {
                    temperature = parsed.dgpu_temp.map(|v| v as f32).unwrap_or(0.0);
                }
                debug!("temp: {:?}", temperature);
                let fan_speed = {
                    let mut ctrl = controller_clone.lock().unwrap();
                    ctrl.update(temperature, &profile)
                };
                let fan_speed_full: u8 = fan_speed as u8;
                println!("Fan speed: {}", fan_speed_full);
                let output = Command::new("framework_tool")
                    .arg("--fansetduty")
                    .arg(fan_speed_full.to_string())
                    .output()
                    .expect("framework_tool failed");
                let stderr = str::from_utf8(&output.stderr).unwrap_or("<invalid utf8>");
                println!("stderr: {}", stderr);
            }

            thread::sleep(Duration::from_secs_f32(sleep_time));
        });

        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];

            if let Ok(n) = stream.read(&mut buf) {
                let received = String::from_utf8_lossy(&buf[..n]).to_string();

                if let Some(name) = received.strip_prefix("use ") {
                    let name = name.trim();
                    info!("received: {}", received);

                    if config.strategies.contains_key(name) {
                        {
                            let mut name_lock = strategy_name.lock().unwrap();
                            *name_lock = name.to_string();
                        }
                        {
                            let mut profile = current_strategy.lock().unwrap();
                            *profile = config.strategies[name].clone();
                        }

                        info!("Switched to strategy: {}", name);
                        let msg = format!("Switched to strategy: {}", name);
                        stream.write_all(msg.as_bytes())?;
                    } else {
                        warn!("Unknown strategy: {}", name);
                        let msg = format!("Unknown strategy: {}", name);
                        stream.write_all(msg.as_bytes())?;
                    }
                } else {
                    stream.write_all(b"unknown or unfinished argument")?;
                }
            }
        }
    }
}

fn send_to_daemon(msg: String) -> std::io::Result<String> {
    let mut stream = UnixStream::connect(SOCK_PATH)?;
    stream.write_all(msg.as_bytes())?;

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf[..n]).to_string())
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "run" {
        if unsafe { libc::geteuid() != 0 } {
            error!("Root privileges required.");
            exit(1);
        }

        if let Err(e) = run_daemon() {
            error!("failed: {}", e);
        }
        return;
    }

    if args.len() > 1 {
        let msg = args[1..].join(" ");
        match send_to_daemon(msg) {
            Ok(response) => println!("{}", response),
            Err(e) => error!("failed: {}", e),
        }
    }
}
