#![allow(non_snake_case)]
#[repr(C, packed)]
pub struct PowerSensor {
    pub voltage: u16,
    pub current: u32,
    pub power: u32,
}

#[repr(C, packed)]
pub struct SensorStruct {
    pub vdd: u16,
    pub tchip: u16,
    pub power_readings: [PowerSensor; 10],
    pub eps_power: u16,
    pub pcie_power: u16,
    pub mb_power: u16,
    pub total_power: u16,
}

pub const SENSORS: &[(&str, &str, &str)] = &[
    ("Total Power", "POWER", "W"), ("EPS Power", "EPS", "W"),
    ("PCIE Power", "PCIE", "W"), ("MB Power", "MB", "W"),
    ("ATX 12V Voltage", "ATX12_V", "V"), ("ATX 12V Current", "ATX12_I", "A"),
    ("ATX 12V Power", "ATX12_P", "W"), ("ATX 5V Voltage", "ATX5_V", "V"),
    ("ATX 5V Current", "ATX5_I", "A"), ("ATX 5V Power", "ATX5_P", "W"),
    ("ATX 5VSB Voltage", "ATX5S_V", "V"), ("ATX 5VSB Current", "ATX5S_I", "A"),
    ("ATX 5VSB Power", "ATX5S_P", "W"), ("ATX 3V3 Voltage", "ATX3_V", "V"),
    ("ATX 3V3 Current", "ATX3_I", "A"), ("ATX 3V3 Power", "ATX3_P", "W"),
    ("HPWR Voltage", "HPWR_V", "V"), ("HPWR Current", "HPWR_I", "A"),
    ("HPWR Power", "HPWR_P", "W"), ("EPS1 Voltage", "EPS1_V", "V"),
    ("EPS1 Current", "EPS1_I", "A"), ("EPS1 Power", "EPS1_P", "W"),
    ("EPS2 Voltage", "EPS2_V", "V"), ("EPS2 Current", "EPS2_I", "A"),
    ("EPS2 Power", "EPS2_P", "W"), ("PCIE1 Voltage", "PCIE1_V", "V"),
    ("PCIE1 Current", "PCIE1_I", "A"), ("PCIE1 Power", "PCIE1_P", "W"),
    ("PCIE2 Voltage", "PCIE2_V", "V"), ("PCIE2 Current", "PCIE2_I", "A"),
    ("PCIE2 Power", "PCIE2_P", "W"), ("PCIE3 Voltage", "PCIE3_V", "V"),
    ("PCIE3 Current", "PCIE3_I", "A"), ("PCIE3 Power", "PCIE3_P", "W"),
];

pub const RAILS: &[&str] = &["ATX12", "ATX5", "ATX5S", "ATX3", "HPWR", "EPS1", "EPS2", "PCIE1", "PCIE2", "PCIE3"]; 