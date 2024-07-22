use std::env;

// This is a very simple program for personal use that makes it easy to cycle through easyeffect
// presets and audio sinks. Maybe one day I'll have it read from config files rather than
// hardcoding preset names in the source code, but it is not this day.

fn main() {
    let args: Vec<String> = env::args().collect();

    match (args[1].as_str(), args[2].as_str()) {
        ("easyeffects", "next") => easyeffects::set_next_preset(),
        ("easyeffects", "get") => println!("{}", easyeffects::get_current_preset()),
        ("audiosink", "next") => audiosink::set_next_sink(),
        ("audiosink", "get") => println!("{}", audiosink::get_current_sink_description()),
        _ => println!("Usage: easyswitch (easyeffects | audiosink) next | get"),
    }
}

mod easyeffects {
    const PRESETS: &[&str] = &["Bose", "AKG + TAGA amp", "Voice Booster"];
    use std::process::Command;

    pub fn get_current_preset() -> String {
        let output = Command::new("dconf")
            .arg("read")
            .arg("/com/github/wwmm/easyeffects/last-loaded-output-preset")
            .output()
            .expect("Failed to get current preset");

        let output = String::from_utf8(output.stdout).expect("failed to parse output");
        return output[1..output.len() - 2].to_string();
    }

    fn get_current_preset_index() -> usize {
        let current_preset = get_current_preset();

        PRESETS.iter().position(|&x| x == current_preset).unwrap()
    }

    fn get_next_preset_index() -> usize {
        let current_index = get_current_preset_index();

        return (current_index + 1) % PRESETS.len();
    }

    pub fn set_next_preset() {
        let next_preset_index = get_next_preset_index();
        let next_preset = PRESETS[next_preset_index].to_string();

        Command::new("easyeffects")
            .arg("--load-preset")
            .arg(&next_preset)
            .output()
            .expect("failed to write preset");

        println!("Set preset to {}", next_preset);
    }
}

mod audiosink {
    use serde_json::Value;
    use std::process::Command;

    fn get_pactl_status() -> Value {
        let status = Command::new("pactl")
            .arg("-f")
            .arg("json")
            .arg("info")
            .output()
            .expect("failed to get pactl status");

        let status =
            String::from_utf8(status.stdout).expect("failed to parse pactl status as a string");

        return serde_json::from_str::<Value>(&status).expect("failed to parse pactl json");
    }

    fn is_physical_sink(sink: &Value) -> bool {
        if let Value::String(is_virtual) = &sink["properties"]["node.virtual"] {
            is_virtual != "true"
        } else {
            // If the property is not present, assume it is a physical sink
            true
        }
    }

    fn get_sinks() -> Vec<Value> {
        let sinks = Command::new("pactl")
            .arg("-f")
            .arg("json")
            .arg("list")
            .arg("sinks")
            .output()
            .expect("pactl list sinks failed");

        let sinks =
            String::from_utf8(sinks.stdout).expect("failed to parse string from pactl list sinks");
        let sinks = serde_json::from_str::<Value>(&sinks)
            .expect("failed to parse json from pactl list sinks");

        if let Value::Array(sinks) = sinks {
            return sinks
                .into_iter()
                .filter(is_physical_sink)
                .collect::<Vec<Value>>();
        } else {
            panic!("pactl list sinks did not return an array");
        }
    }

    fn get_current_sink_data() -> Value {
        let current_sink = get_current_sink_name();
        let sinks = get_sinks();

        return sinks
            .into_iter()
            .find(|sink| sink["name"].as_str().unwrap() == current_sink)
            .expect("failed to find current sink");
    }

    fn get_current_sink_index() -> usize {
        let current_sink = get_current_sink_data();

        if let Value::Number(index) = &current_sink["index"] {
            return index.as_u64().unwrap() as usize;
        } else {
            panic!("pactl list sinks did not return expected json");
        };
    }

    fn get_next_sink_pulseaudio_index() -> usize {
        let current_pulseaudio_index = get_current_sink_index();
        let all_pulseaudio_indices = get_sinks()
            .into_iter()
            .map(|sink| {
                if let Value::Number(pulseaudio_index) = &sink["index"] {
                    return pulseaudio_index.as_u64().unwrap() as usize;
                } else {
                    panic!("pactl list sinks did not return expected json");
                }
            })
            .collect::<Vec<usize>>();

        let current_array_index = all_pulseaudio_indices
            .clone()
            .into_iter()
            .position(|x| x == current_pulseaudio_index)
            .expect("failed to find current pulseaudio index");

        let next_array_index = (current_array_index + 1) % &all_pulseaudio_indices.len();

        return all_pulseaudio_indices[next_array_index];
    }

    fn get_current_sink_name() -> String {
        let status = get_pactl_status();

        let default_sink_name = &status["default_sink_name"];

        return default_sink_name.as_str().unwrap().to_string();
    }

    pub fn get_current_sink_description() -> String {
        let current_sink = get_current_sink_data();

        if let Value::String(description) = &current_sink["properties"]["node.nick"] {
            return description.to_string();
        } else {
            return get_current_sink_name();
        }
    }

    pub fn set_next_sink() {
        let result = Command::new("pactl")
            .arg("set-default-sink")
            .arg(format!("{}", get_next_sink_pulseaudio_index()))
            .output()
            .expect("failed to set default sink");

        println!("Set sink to {}", get_current_sink_name());
    }
}
