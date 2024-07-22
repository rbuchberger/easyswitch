use std::env;
use std::process::Command;

// This is a very simple program that allows you to switch presets in EasyEffects.
// It is intended to be used with a waybar module.

const PRESETS: &[&str] = &["Bose", "AKG + TAGA amp", "Voice Booster"];

fn main() {
    let args: Vec<String> = env::args().collect();

    match args[1].as_str() {
        "next" => set_next_preset(),
        "get" => println!("{}", get_current_preset()),
        _ => println!("Usage: easyswitch next | get"),
    }
}

fn get_current_preset() -> String {
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

fn set_next_preset() {
    let next_preset_index = get_next_preset_index();
    let next_preset = PRESETS[next_preset_index].to_string();

    Command::new("easyeffects")
        .arg("--load-preset")
        .arg(&next_preset)
        .output()
        .expect("failed to write preset");

    println!("Set preset to {}", next_preset);
}
