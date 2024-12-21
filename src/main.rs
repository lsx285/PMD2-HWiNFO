use std::{thread, time::Duration, env};
mod registry;
mod serial;

use PMD2_HWiNFO::RAILS;
use crate::registry::{init_hwinfo_registry, update_registry, find_com_port, setup_sensors, toggle_startup};
use crate::serial::{open_serial_port, read_sensor_data};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Handle startup argument
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1].eq_ignore_ascii_case("-s")) {
        toggle_startup()?;
        return Ok(());
    }

    // Initialize registry
    let reg_key = init_hwinfo_registry()?;
    let sensor_keys = setup_sensors(reg_key)?;

    // Setup serial communication
    let port_name = find_com_port();
    let mut port = open_serial_port(&port_name)?;

    // Main loop
    loop {
        if let Some(data) = read_sensor_data(&mut port) {
            // Update summary values
            for (key, val) in [
                ("POWER", data.total_power), ("EPS", data.eps_power),
                ("PCIE", data.pcie_power), ("MB", data.mb_power)
            ] {
                if let Some(k) = sensor_keys.get(key) {
                    update_registry(reg_key, k, val as f64)?;
                }
            }

            // Update detailed readings
            for (prefix, reading) in RAILS.iter().zip(data.power_readings.iter()) {
                for (suffix, val) in [
                    ("_V", reading.voltage as f64 / 1000.0),
                    ("_I", reading.current as f64 / 1000.0),
                    ("_P", reading.power as f64 / 1000.0)
                ] {
                    if let Some(k) = sensor_keys.get(&format!("{}{}", prefix, suffix)) {
                        update_registry(reg_key, k, val)?;
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}