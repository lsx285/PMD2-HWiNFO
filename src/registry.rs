use std::collections::HashMap;
use windows::{
    core::{PCWSTR, PWSTR, w},
    Win32::Foundation::ERROR_SUCCESS,
    Win32::System::Registry::{
        HKEY, RegCreateKeyExW, RegSetValueExW, RegOpenKeyExW, RegCloseKey,
        RegEnumKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
        KEY_READ, KEY_ALL_ACCESS, REG_OPTION_NON_VOLATILE, REG_SZ, REG_VALUE_TYPE,
        RegDeleteValueW,
    },
};
use PMD2_HWiNFO::SENSORS;
use std::env;

pub fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn update_registry(reg_key: HKEY, key: &str, value: f64) -> windows::core::Result<()> {
    unsafe {
        let mut subkey = HKEY::default();
        let key_wide = to_wide_null(key);
        RegOpenKeyExW(reg_key, PCWSTR::from_raw(key_wide.as_ptr()), 0, KEY_ALL_ACCESS, &mut subkey).ok()?;
        let value_wide = to_wide_null(&value.to_string());
        RegSetValueExW(subkey, w!("Value"), 0, REG_SZ,
            Some(std::slice::from_raw_parts(value_wide.as_ptr() as *const u8, value_wide.len() * 2))).ok()?;
        let _ = RegCloseKey(subkey);
        Ok(())
    }
}

pub fn find_com_port() -> String {
    let mut port_name = "COM1".to_string();
    unsafe {
        let mut key = HKEY::default();
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Enum\\USB\\VID_0483&PID_5740"),
            0, KEY_READ, &mut key) == ERROR_SUCCESS {
            let mut name_buf = [0u16; 256];
            let mut name_size = name_buf.len() as u32;
            if RegEnumKeyExW(key, 0, PWSTR(name_buf.as_mut_ptr()), &mut name_size, None,
                PWSTR::null(), None, None) == ERROR_SUCCESS {
                let device_id = String::from_utf16_lossy(&name_buf[..name_size as usize]);
                let mut params_key = HKEY::default();
                let device_path = format!("SYSTEM\\CurrentControlSet\\Enum\\USB\\VID_0483&PID_5740\\{}\\Device Parameters", device_id);
                if RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                    PCWSTR::from_raw(to_wide_null(&device_path).as_ptr()),
                    0, KEY_READ, &mut params_key) == ERROR_SUCCESS {
                    let mut value_buf = [0u16; 256];
                    let mut value_size = (value_buf.len() * 2) as u32;
                    let mut value_type = REG_VALUE_TYPE::default();
                    if RegQueryValueExW(params_key, w!("PortName"), None, Some(&mut value_type),
                        Some(value_buf.as_mut_ptr() as *mut u8), Some(&mut value_size)) == ERROR_SUCCESS
                        && value_type == REG_SZ {
                        port_name = String::from_utf16_lossy(&value_buf[..value_size as usize / 2 - 1]);
                    }
                    let _ = RegCloseKey(params_key);
                }
            }
            let _ = RegCloseKey(key);
        }
    }
    port_name
}

pub fn init_hwinfo_registry() -> windows::core::Result<HKEY> {
    unsafe {
        let mut device_key = HKEY::default();
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\HWiNFO64\\Sensors\\Custom\\ElmorLabs PMD2"),
            0, PCWSTR::null(), REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS,
            None, &mut device_key, None
        ).ok()?;
        Ok(device_key)
    }
}

pub fn setup_sensors(reg_key: HKEY) -> windows::core::Result<HashMap<String, String>> {
    let mut sensor_keys = HashMap::new();
    let mut indices = (0, 0, 0); // (power, volt, current)

    for &(name, sensor, format) in SENSORS {
        let (key, idx) = match format {
            "W" => (format!("Power{}", indices.0), &mut indices.0),
            "V" => (format!("Volt{}", indices.1), &mut indices.1),
            "A" => (format!("Current{}", indices.2), &mut indices.2),
            _ => continue,
        };
        *idx += 1;
        sensor_keys.insert(sensor.to_string(), key.clone());

        unsafe {
            let mut subkey = HKEY::default();
            let key_wide = format!("{}\0", key).encode_utf16().collect::<Vec<u16>>();
            RegCreateKeyExW(reg_key, PCWSTR::from_raw(key_wide.as_ptr()),
                0, PCWSTR::null(), REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, None, &mut subkey, None).ok()?;
            
            let name_wide = format!("{}\0", name).encode_utf16().collect::<Vec<u16>>();
            RegSetValueExW(subkey, w!("Name"), 0, REG_SZ,
                Some(std::slice::from_raw_parts(name_wide.as_ptr() as *const u8, name_wide.len() * 2))).ok()?;
            let zero_wide = "0\0".encode_utf16().collect::<Vec<u16>>();
            RegSetValueExW(subkey, w!("Value"), 0, REG_SZ,
                Some(std::slice::from_raw_parts(zero_wide.as_ptr() as *const u8, zero_wide.len() * 2))).ok()?;
            let _ = RegCloseKey(subkey);
        }
    }

    Ok(sensor_keys)
}

pub fn toggle_startup() -> windows::core::Result<()> {
    unsafe {
        let mut run_key = HKEY::default();
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            None,
            &mut run_key,
            None,
        ).ok()?;

        let app_name = w!("PMD2-HWiNFO");
        
        let mut buf = [0u8; 1024];
        let mut size = buf.len() as u32;
        let mut value_type = REG_VALUE_TYPE::default();
        
        let result = RegQueryValueExW(
            run_key,
            app_name,
            None,
            Some(&mut value_type),
            Some(buf.as_mut_ptr()),
            Some(&mut size),
        );

        if result == ERROR_SUCCESS {
            RegDeleteValueW(run_key, app_name).ok()?;
        } else {
            let exe_path = env::current_exe()?.to_string_lossy().to_string();
            let value = format!("\"{}\"\0", exe_path);
            let value_wide = value.encode_utf16().collect::<Vec<u16>>();
            
            RegSetValueExW(
                run_key,
                app_name,
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    value_wide.as_ptr() as *const u8,
                    value_wide.len() * 2,
                )),
            ).ok()?;
        }

        let _ = RegCloseKey(run_key);
        Ok(())
    }
} 