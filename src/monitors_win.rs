use std::collections::HashMap;
use std::ffi::c_void;
use std::io;
use ddc_winapi::get_physical_monitors_from_hmonitor;
use windows::Win32::Graphics::Gdi::*;

pub fn build_os_to_ddc_map() -> Result<HashMap<String, Vec<String>>, io::Error> {
    let mut mapping = HashMap::new();
    
    let hwindows = ddc_winapi::enumerate_monitors()?;

    for hwindow in hwindows {
        let mut gdi_name = None;

        let mut monitor_info = MONITORINFOEXW::default();
        monitor_info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;

        let hmonitor = HMONITOR(hwindow as *mut c_void);

        unsafe {
            if GetMonitorInfoW(hmonitor, &mut monitor_info.monitorInfo).as_bool() {
                gdi_name = Some(String::from_utf16_lossy(&monitor_info.szDevice)
                    .trim_end_matches('\0')
                    .to_string());
                println!("Got monitor GDI {}", gdi_name.as_ref().unwrap()); // SAFETY: Just set to Some.
            }
        }
        
        let mut monitor_descs = Vec::new();

        let physical_monitors = match get_physical_monitors_from_hmonitor(hwindow) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to get physical monitors from HMONITOR: {e:?}");
                continue;
            }
        };

        for physical_monitor in physical_monitors {
            let monitor = unsafe { ddc_winapi::Monitor::new(physical_monitor) };
            let description = monitor.description();

            println!("Got monitor {:?}", description);
            monitor_descs.push(description);
        }

        if let Some(gdi_name) = gdi_name {
            mapping.insert(gdi_name, monitor_descs);
        }
    }

    Ok(mapping)
}