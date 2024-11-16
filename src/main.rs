use std::{time::Duration, thread, io, collections::HashMap, ptr::addr_of_mut, ffi::c_void};
use windows::{
    Win32::System::Registry::*,
    Win32::Foundation::*,
    Win32::Storage::FileSystem::*,
    Win32::Devices::Communication::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::System::Console::*,
    core::{PCWSTR, PWSTR},
    w,
};

#[repr(C, packed)] struct DeviceData { vdd: u16, temp: u16, sensors: [SensorData; 10], eps_power: u16, 
    pcie_power: u16, mb_power: u16, total_power: u16, ocp: [u8; 10] }
#[repr(C, packed)] struct SensorData { voltage: u16, current: u32, power: u32 }
struct PowerReading { name: String, power: f64 }
struct HWiNFO { reg_key: HKEY, sensor_keys: HashMap<String, String> }
struct SerialPort { handle: HANDLE }
struct PowerMonitorService { port: SerialPort, hwinfo: HWiNFO, readings: Vec<PowerReading> }

impl SerialPort {
    fn new(port_name: &str) -> io::Result<Self> {
        let port_path = format!("\\\\.\\{}", port_name);
        let wide_path: Vec<u16> = port_path.encode_utf16().chain(std::iter::once(0)).collect();
        
        let handle = unsafe {
            CreateFileW(
                PCWSTR::from_raw(wide_path.as_ptr()),
                GENERIC_READ.0 | GENERIC_WRITE.0,
                FILE_SHARE_NONE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )?
        };

        let mut dcb: DCB = unsafe { std::mem::zeroed() };
        dcb.DCBlength = std::mem::size_of::<DCB>() as u32;
        dcb.BaudRate = 115200;
        dcb.ByteSize = 8;
        dcb.StopBits = ONESTOPBIT;
        dcb.Parity = DCB_PARITY(0);
        unsafe {
            let ptr = &mut dcb as *mut DCB;
            let bitfield_ptr = (ptr as *mut u8).add(std::mem::offset_of!(DCB, _bitfield));
            *bitfield_ptr |= 1; // Set fBinary bit
        }
        
        if unsafe { SetCommState(handle, &dcb) }.as_bool() {
            let timeouts = COMMTIMEOUTS {
                ReadIntervalTimeout: 0,
                ReadTotalTimeoutMultiplier: 0,
                ReadTotalTimeoutConstant: 100,
                WriteTotalTimeoutMultiplier: 0,
                WriteTotalTimeoutConstant: 100,
            };
            
            if unsafe { SetCommTimeouts(handle, &timeouts) }.as_bool() {
                Ok(Self { handle })
            } else {
                unsafe { CloseHandle(handle) };
                Err(io::Error::last_os_error())
            }
        } else {
            unsafe { CloseHandle(handle) };
            Err(io::Error::last_os_error())
        }
    }

    fn clear_input(&mut self) -> io::Result<()> {
        if unsafe { PurgeComm(self.handle, PURGE_RXCLEAR) }.as_bool() {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        let mut written = 0u32;
        if unsafe { 
            WriteFile(
                self.handle,
                Some(buf),
                Some(&mut written),
                None,
            )
        }.as_bool() {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let mut read = 0u32;
        if unsafe { 
            ReadFile(
                self.handle,
                Some(buf.as_mut_ptr() as *mut c_void),
                buf.len() as u32,
                Some(&mut read),
                None,
            )
        }.as_bool() {
            if read as usize == buf.len() {
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::UnexpectedEof, "failed to fill whole buffer"))
            }
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

impl Drop for SerialPort {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle) };
    }
}

impl HWiNFO {
    fn new() -> io::Result<Self> {
        let mut key_handle = HKEY::default();
        let result = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                w!("Software\\HWiNFO64\\Sensors\\Custom\\ElmorLabs PMD2"),
                0,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_ALL_ACCESS,
                None,
                &mut key_handle,
                None,
            )
        };
        
        if result != ERROR_SUCCESS {
            return Err(io::Error::last_os_error());
        }

        Ok(Self { 
            reg_key: key_handle,
            sensor_keys: HashMap::new() 
        })
    }

    fn set_registry_value(key: HKEY, name: &str, value: &str) -> io::Result<()> {
        let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let wide_value: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        
        let result = unsafe {
            RegSetValueExW(
                key,
                PCWSTR::from_raw(wide_name.as_ptr()),
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    wide_value.as_ptr() as *const u8,
                    wide_value.len() * 2
                )),
            )
        };

        if result != ERROR_SUCCESS {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn register_sensors(&mut self, readings: &[PowerReading]) -> io::Result<()> {
        for (i, r) in readings.iter().enumerate() {
            for (s, t) in [("Power", "Power")] {
                let k = format!("{}{}", t, i);
                self.sensor_keys.insert(format!("{}_{}", r.name, s), k.clone());
                
                let wide_k: Vec<u16> = k.encode_utf16().chain(std::iter::once(0)).collect();
                let mut subkey = HKEY::default();
                let result = unsafe {
                    RegCreateKeyExW(
                        self.reg_key,
                        PCWSTR::from_raw(wide_k.as_ptr()),
                        0,
                        PCWSTR::null(),
                        REG_OPTION_NON_VOLATILE,
                        KEY_ALL_ACCESS,
                        None,
                        &mut subkey,
                        None,
                    )
                };

                if result != ERROR_SUCCESS {
                    return Err(io::Error::last_os_error());
                }

                Self::set_registry_value(subkey, "Name", &format!("{} {}", r.name, t))?;
                Self::set_registry_value(subkey, "Value", "0")?;
                
                unsafe { RegCloseKey(subkey) };
            }
        }

        let formulas = [
            ("EPS Total Power", "\"EPS 1 Power\" + \"EPS 2 Power\""),
            ("PCIe Total Power", "\"PCIe 1 Power\" + \"PCIe 2 Power\" + \"PCIe 3 Power\" + \"12VHPWR Power\""),
            ("GPU Power", "\"12VHPWR Power\" + \"EPS 1 Power\""),
            ("MB Power", "\"ATX 12V Power\" + \"ATX 5V Power\" + \"ATX 5VSB Power\" + \"ATX 3.3V Power\""),
            ("System Power", "\"ATX 12V Power\" + \"ATX 5V Power\" + \"ATX 5VSB Power\" + \"ATX 3.3V Power\" + \
             \"12VHPWR Power\" + \"EPS 1 Power\" + \"EPS 2 Power\" + \"PCIe 1 Power\" + \"PCIe 2 Power\" + \"PCIe 3 Power\""),
        ];

        for (i, (n, f)) in formulas.iter().enumerate() {
            let k = format!("Power{}", readings.len() + i);
            self.sensor_keys.insert(n.to_string(), k.clone());
            
            let wide_k: Vec<u16> = k.encode_utf16().chain(std::iter::once(0)).collect();
            let mut subkey = HKEY::default();
            let result = unsafe {
                RegCreateKeyExW(
                    self.reg_key,
                    PCWSTR::from_raw(wide_k.as_ptr()),
                    0,
                    PCWSTR::null(),
                    REG_OPTION_NON_VOLATILE,
                    KEY_ALL_ACCESS,
                    None,
                    &mut subkey,
                    None,
                )
            };

            if result != ERROR_SUCCESS {
                return Err(io::Error::last_os_error());
            }

            Self::set_registry_value(subkey, "Name", n)?;
            Self::set_registry_value(subkey, "Value", f)?;
            unsafe { RegCloseKey(subkey) };
        }
        Ok(())
    }

    fn update_values(&self, readings: &[PowerReading]) -> io::Result<()> {
        for r in readings {
            for (s, v) in [("Power",r.power)] {
                if let Some(k) = self.sensor_keys.get(&format!("{}_{}", r.name, s)) {
                    let wide_k: Vec<u16> = k.encode_utf16().chain(std::iter::once(0)).collect();
                    let mut subkey = HKEY::default();
                    let result = unsafe {
                        RegOpenKeyExW(
                            self.reg_key,
                            PCWSTR::from_raw(wide_k.as_ptr()),
                            0,
                            KEY_ALL_ACCESS,
                            &mut subkey,
                        )
                    };
                    if result != ERROR_SUCCESS {
                        return Err(io::Error::last_os_error());
                    }
                    Self::set_registry_value(subkey, "Value", &v.to_string())?;
                    unsafe { RegCloseKey(subkey) };
                }
            }
        }
        Ok(())
    }
}

impl PowerMonitorService {
    fn new() -> io::Result<Self> {
        let mut port = "COM1".to_string();
        
        let mut key_handle = HKEY::default();
        if unsafe { RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Enum\\USB\\VID_0483&PID_5740"),
            0,
            KEY_READ,
            &mut key_handle,
        )} == ERROR_SUCCESS {
            let mut name_buf = [0u16; 256];
            let mut name_size = name_buf.len() as u32;
            
            if unsafe { RegEnumKeyExW(
                key_handle,
                0,
                PWSTR(name_buf.as_mut_ptr()),
                &mut name_size,
                None,
                PWSTR::null(),
                None,
                None,
            )} == ERROR_SUCCESS {
                let device_id = String::from_utf16_lossy(&name_buf[..name_size as usize]);
                let params_path = format!("SYSTEM\\CurrentControlSet\\Enum\\USB\\VID_0483&PID_5740\\{}\\Device Parameters", device_id);
                
                let params_path_wide: Vec<u16> = params_path.encode_utf16().chain(std::iter::once(0)).collect();
                let mut params_key = HKEY::default();
                if unsafe { RegOpenKeyExW(
                    HKEY_LOCAL_MACHINE,
                    PCWSTR::from_raw(params_path_wide.as_ptr()),
                    0,
                    KEY_READ,
                    &mut params_key,
                )} == ERROR_SUCCESS {
                    let mut value_buf = [0u16; 256];
                    let mut value_size = (value_buf.len() * 2) as u32;
                    let mut value_type = REG_VALUE_TYPE::default();
                    
                    if unsafe { RegQueryValueExW(
                        params_key,
                        w!("PortName"),
                        None,
                        Some(&mut value_type),
                        Some(value_buf.as_mut_ptr() as *mut u8),
                        Some(&mut value_size),
                    )} == ERROR_SUCCESS && value_type == REG_SZ {
                        port = String::from_utf16_lossy(&value_buf[..value_size as usize / 2 - 1]);
                    }
                    unsafe { RegCloseKey(params_key) };
                }
            }
            unsafe { RegCloseKey(key_handle) };
        }

        let readings = ["ATX 12V", "ATX 5V", "ATX 5VSB", "ATX 3.3V", "12VHPWR", 
            "EPS 1", "EPS 2", "PCIe 1", "PCIe 2", "PCIe 3"].iter()
            .map(|&n| PowerReading { name: n.into(), power: 0.0 })
            .collect::<Vec<_>>();

        let mut hwinfo = HWiNFO::new()?;
        hwinfo.register_sensors(&readings)?;

        Ok(Self { 
            port: SerialPort::new(&port)?, 
            hwinfo, 
            readings 
        })
    }

    fn run_measurement_cycle(&mut self) -> io::Result<()> {
        static mut BUF: [u8; std::mem::size_of::<DeviceData>()] = [0; std::mem::size_of::<DeviceData>()];
        self.port.clear_input()?;
        self.port.write_all(&[0x04])?;
        thread::sleep(Duration::from_millis(100));
        
        unsafe {
            self.port.read_exact(&mut *addr_of_mut!(BUF))?;
            let data = std::ptr::read(BUF.as_ptr() as *const DeviceData);
            for (r, s) in self.readings.iter_mut().zip(data.sensors.iter()) {
                r.power = s.power as f64 / 1000.0;
            }
        }
        self.hwinfo.update_values(&self.readings)
    }
}

fn main() {
    unsafe { 
        AllocConsole();
        ShowWindow(GetConsoleWindow(), SW_HIDE);
    }

    let mut monitor = PowerMonitorService::new().expect("Failed to initialize");
    loop {
        if let Err(_) = monitor.run_measurement_cycle() {
            thread::sleep(Duration::from_millis(1000));
        }
        thread::sleep(Duration::from_millis(100));
    }
}