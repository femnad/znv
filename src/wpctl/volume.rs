use crate::wpctl::WPCTL_EXEC;
use std::process::Command;

const DEFAULT_MODIFY_STEP: u32 = 5;
const DEFAULT_SINK_SPECIFIER: &str = "@DEFAULT_AUDIO_SINK@";
const MAXIMUM_VOLUME: f32 = 1.5;
const MINIMUM_MODIFY_STEP: f32 = 0.01;
const MUTED_SUFFIX: &str = "[MUTED]";

fn get_volume() -> f32 {
    let mut cmd = Command::new(WPCTL_EXEC);
    cmd.args(["get-volume", DEFAULT_SINK_SPECIFIER]);
    let out = cmd.output().expect("error getting volume");
    let vol_out = String::from_utf8(out.stdout).expect("error parsing cmd output");
    let vol_out_trim = vol_out.trim();
    if vol_out_trim.ends_with(MUTED_SUFFIX) {
        return 0.0;
    }

    let vol = vol_out_trim
        .split(" ")
        .nth(1)
        .expect("Error getting volume field");
    let vol_f: f32 = vol.parse().expect("error parsing volume");
    vol_f
}

pub fn modify(step: Option<u32>, sign: &str) -> f32 {
    let mut cmd = Command::new(WPCTL_EXEC);

    let modify_step = step.unwrap_or(DEFAULT_MODIFY_STEP);
    let modify_step_str = f32::max(modify_step as f32 / 100.0, MINIMUM_MODIFY_STEP).to_string();
    let max_vol = MAXIMUM_VOLUME.to_string();

    cmd.args([
        "set-volume",
        "-l",
        max_vol.as_str(),
        DEFAULT_SINK_SPECIFIER,
        format!("{modify_step_str}{sign}").as_str(),
    ]);
    cmd.status().expect("error setting volume");

    get_volume()
}

pub fn toggle() -> f32 {
    let mut cmd = Command::new(WPCTL_EXEC);

    cmd.args(["set-mute", DEFAULT_SINK_SPECIFIER, "toggle"]);
    cmd.status().expect("error toggling volume");

    get_volume()
}
