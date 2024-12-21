use std::{time::Duration, thread};
use serialport::SerialPort;
use PMD2_HWiNFO::SensorStruct;

pub fn open_serial_port(port_name: &str) -> Result<Box<dyn SerialPort>, serialport::Error> {
    serialport::new(port_name, 115_200)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(100))
        .open()
}

pub fn read_sensor_data(port: &mut Box<dyn SerialPort>) -> Option<SensorStruct> {
    let mut buf = [0u8; std::mem::size_of::<SensorStruct>()];
    
    port.clear(serialport::ClearBuffer::Input).ok()?;
    port.write_all(&[0x04]).ok()?;
    thread::sleep(Duration::from_millis(100));

    if port.read_exact(&mut buf).is_ok() {
        Some(unsafe { std::ptr::read(buf.as_ptr() as *const SensorStruct) })
    } else {
        None
    }
} 