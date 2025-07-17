use std::collections::HashMap;
use std::io;
use std::thread::sleep;
use std::time::Duration;
use ddc_hi::*;

const VCP_LUMINANCE: u8 = 0x10;
const VCP_CONTRAST: u8 = 0x12;
const VCP_DELAY: Duration = Duration::from_millis(10);

pub struct Monitors {
    ddc_displays: HashMap<String, Display>,
    os_to_ddc: HashMap<String, Vec<String>>,

    saved_luminance: HashMap<String, VcpValue>,
    saved_contrast: HashMap<String, VcpValue>,
}

impl Monitors {
    pub fn new() -> Result<Self, io::Error> {
        let mut displays = Display::enumerate();

        let mut saved_luminance = HashMap::new();
        let mut saved_contrast = HashMap::new();

        for display in &mut displays {
            if let Err(e) = display.update_capabilities() {
                println!("Failed to update display capabilities for {}: {e:?}", display.info.id);
                continue;
            }

            if let Some(feature) = display.info.mccs_database.get(VCP_LUMINANCE) {
                match display.handle.get_vcp_feature(feature.code) {
                    Ok(old_luminance) => {
                        if saved_luminance.contains_key(&display.info.id) {
                            println!("Warning: display ID {} already has a saved luminance! Ignoring...", display.info.id);
                        } else {
                            saved_luminance.insert(display.info.id.clone(), old_luminance);
                        }
                    }
                    Err(e) => {
                        println!("Capabilities report luminance supported, but errored retrieving: {e:?}");
                    }
                }
            }

            sleep(VCP_DELAY);

            if let Some(feature) = display.info.mccs_database.get(VCP_CONTRAST) {
                match display.handle.get_vcp_feature(feature.code) {
                    Ok(old_contrast) => {
                        if saved_contrast.contains_key(&display.info.id) {
                            println!("Warning: display ID {} already has a saved contrast! Ignoring...", display.info.id);
                        } else {
                            saved_contrast.insert(display.info.id.clone(), old_contrast);
                        }
                    }
                    Err(e) => {
                        println!("Capabilities report contrast supported, but errored retrieving: {e:?}");
                    }
                }
            }

            sleep(VCP_DELAY);
        }

        let os_to_ddc = Self::map_os_to_ddc()?;

        let ddc_displays: HashMap<String, Display> = displays.into_iter().map(|x| {
            (x.info.id.clone(), x)
        }).collect();

        println!("{saved_luminance:#?}");
        println!("{saved_contrast:#?}");
        println!("{os_to_ddc:#?}");

        Ok(Self {
            ddc_displays,
            os_to_ddc,
            saved_luminance,
            saved_contrast,
        })
    }

    fn map_os_to_ddc() -> Result<HashMap<String, Vec<String>>, io::Error> {
        #[cfg(windows)]
        crate::monitors_win::build_os_to_ddc_map()
    }

    pub fn spotlight(&mut self, os_names: Vec<&str>) {
        let mut to_spotlight = Vec::new();

        for os_name in os_names {
            if let Some(monitors) = self.os_to_ddc.get(os_name).cloned() {
                to_spotlight.extend(monitors);
            }
        }

        let to_dim = self.ddc_displays.keys().cloned().filter(|x| !to_spotlight.contains(&x)).collect::<Vec<_>>();

        println!("to dim: {to_dim:#?}");
        println!("to spotlight: {to_spotlight:#?}");

        for id in to_dim {
            self.dim(&id);
        }

        for id in to_spotlight {
            self.restore(&id);
        }
    }

    pub fn dim(&mut self, id: &String) {
        if let Some(display) = self.ddc_displays.get_mut(id) {
            if let Some(feature) = display.info.mccs_database.get(VCP_LUMINANCE) {
                match display.handle.set_vcp_feature(feature.code, 0) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Capabilities report luminance supported, but errored setting: {e:?}");
                    }
                }
            }

            sleep(VCP_DELAY);

            if let Some(feature) = display.info.mccs_database.get(VCP_CONTRAST) {
                match display.handle.set_vcp_feature(feature.code, 0) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Capabilities report contrast supported, but errored setting: {e:?}");
                    }
                }
            }
        }
    }

    pub fn restore(&mut self, id: &String) {
        if let Some(display) = self.ddc_displays.get_mut(id) {
            if let Some(old_luminance) = self.saved_luminance.get(id) {
                if let Some(feature) = display.info.mccs_database.get(VCP_LUMINANCE) {
                    println!("Resetting {} to luminance: {}", display.info.id, old_luminance.value());
                    match display.handle.set_vcp_feature(feature.code, old_luminance.value()) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Capabilities report luminance supported, but errored setting: {e:?}");
                        }
                    }
                }
            }

            sleep(VCP_DELAY);

            if let Some(old_contrast) = self.saved_contrast.get(id) {
                if let Some(feature) = display.info.mccs_database.get(VCP_CONTRAST) {
                    println!("Resetting {} to contrast: {}", display.info.id, old_contrast.value());
                    match display.handle.set_vcp_feature(feature.code, old_contrast.value()) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Capabilities report contrast supported, but errored setting: {e:?}");
                        }
                    }
                }
            }
        }
    }

    pub fn dim_all(&mut self) {
        let ids = &self.ddc_displays.keys().cloned().collect::<Vec<String>>();
        for id in ids {
            self.dim(id);
            sleep(VCP_DELAY);
        }
    }

    pub fn restore_all(&mut self) {
        let ids = &self.ddc_displays.keys().cloned().collect::<Vec<String>>();
        for id in ids {
            self.restore(id);
            sleep(VCP_DELAY);
        }
    }
}

impl Drop for Monitors {
    fn drop(&mut self) {
        self.restore_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_displays() -> Result<(), io::Error>{
        let monitors = Monitors::new()?;
        Ok(())
    }

    #[test]
    fn test_dump_diagnostics() -> Result<(), io::Error>{
        let monitors = Monitors::new()?;

        let displays = &monitors.ddc_displays;
        for (id, display) in displays {
            print!("{id} : ");
            match &display.handle {
                Handle::WinApi(_) => println!("\tWinApi"),
                Handle::Nvapi(_) => println!("\tNVAPI"),
            }

            println!("{:#?}", display.info);

            println!()
        }

        Ok(())
    }

    #[test]
    fn test_spotlight_display() -> Result<(), io::Error> {
        let mut monitors = Monitors::new()?;
        monitors.spotlight(vec!["\\\\.\\DISPLAY1"]);
        Ok(())
    }

    #[test]
    fn test_dim_displays() -> Result<(), io::Error> {
        let mut monitors = Monitors::new()?;
        monitors.dim_all();
        sleep(Duration::from_secs(5));
        Ok(())
    }
}