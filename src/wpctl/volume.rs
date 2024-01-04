use std::process::Command;
use crate::wpctl::WPCTL_EXEC;

const DEFAULT_SINK_SPECIFIER: &str = "@DEFAULT_AUDIO_SINK@";
const MUTED_SUFFIX: &str = "[MUTED]";
const VOLUME_MODIFY_STEP: f32 = 0.01;

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

pub fn modify(sign: &str) -> f32 {
    let mut cmd = Command::new(WPCTL_EXEC);

    cmd.args([
        "set-volume",
        "-l",
        "1.5",
        DEFAULT_SINK_SPECIFIER,
        format!("{VOLUME_MODIFY_STEP}{sign}").as_str(),
    ]);
    cmd.status().expect("error setting volume");

    get_volume()
}

pub fn toggle() -> f32 {
    let mut cmd = Command::new(WPCTL_EXEC);

    cmd.args([
        "set-mute",
        DEFAULT_SINK_SPECIFIER,
        "toggle"
    ]);
    cmd.status().expect("error toggling volume");

    get_volume()
}
