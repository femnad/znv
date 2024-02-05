use crate::notify;
use crate::wpctl::volume::ChangeType::Toggle;
use crate::wpctl::WPCTL_EXEC;
use std::process::Command;

const DEFAULT_MODIFY_STEP: u32 = 5;
const DEFAULT_SINK_SPECIFIER: &str = "@DEFAULT_AUDIO_SINK@";
const MAXIMUM_VOLUME: f32 = 1.5;
const MINIMUM_MODIFY_STEP: f32 = 0.01;
const MUTED_SUFFIX: &str = "[MUTED]";

pub struct Change {
    change_type: ChangeType,
    step: Option<u32>,
}

impl Change {
    pub fn new(change_type: ChangeType, step: Option<u32>) -> Self {
        Change { change_type, step }
    }
}

#[derive(Debug)]
pub enum ChangeType {
    Dec,
    Inc,
    Toggle,
}

fn lookup() -> f32 {
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

fn modify(step: Option<u32>, sign: &str) {
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
}

fn toggle() {
    let mut cmd = Command::new(WPCTL_EXEC);

    cmd.args(["set-mute", DEFAULT_SINK_SPECIFIER, "toggle"]);
    cmd.status().expect("error toggling volume");
}

pub fn apply(change: Change) {
    let old_volume = lookup();
    match change.change_type {
        dec_or_inc @ (ChangeType::Dec | ChangeType::Inc) => {
            let sign = match dec_or_inc {
                ChangeType::Dec => "-",
                ChangeType::Inc => "+",
                _ => unreachable!("Unexpected volume change type {:?}", dec_or_inc),
            };
            modify(change.step, sign);
        }
        Toggle => toggle(),
    };

    let new_volume = lookup();
    if old_volume != new_volume || new_volume == 0.0 {
        notify::volume(new_volume);
    }
}
