mod monitors;
mod monitors_win;

use libmpv_client::*;
use libmpv_client::event::PropertyValue;
use crate::monitors::Monitors;

#[unsafe(no_mangle)]
extern "C" fn mpv_open_cplugin(ptr: *mut mpv_handle) -> std::os::raw::c_int {
    let handle = Handle::from_ptr(ptr);

    match handle.observe_property("display-names", Format::STRING, 0) {
        Ok(_) => {
            println!("Watching display-names...")
        }
        Err(e) => {
            println!("Error watching display-names: {e:?}");
        }
    }

    // Monitor init takes a while; send a wait_event to mpv so it knows we're alive, don't want to hang it waiting on DDC.
    handle.wait_event(0.0).unwrap();

    let mut monitors = match Monitors::new() {
        Ok(monitors) => monitors,
        Err(e) => {
            println!("Error creating Monitors: {e:?}");
            return -1;
        }
    };
    
    loop {
        match handle.wait_event(0.0) {
            Err(e) => {
                println!("wait_event error: {e:?}");
            }
            Ok(event) => {
                match event {
                    Event::Shutdown => {
                        println!("Shutting down Spotlight!");
                        return 0;
                    },
                    Event::None => {},
                    Event::PropertyChange(property) => {
                        match property.name.as_str() {
                            "display-names" => {
                                match property.value {
                                    Ok(val) => {
                                        if let PropertyValue::String(csv) = val {
                                            let displays = csv.split(",").collect::<Vec<&str>>();
                                            monitors.spotlight(displays);
                                        }
                                    }
                                    Err(e) => {
                                        println!("Got error for display-names: {e:?}");
                                    }
                                }
                            }
                            _ => println!("Got unknown property {property:?}"),
                        }
                    }
                    event => {
                        println!("Got event: {event:?}");
                    },
                }
            }
        }
    }
}