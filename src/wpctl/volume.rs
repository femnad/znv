use crate::notify;
use crate::wpctl::WPCTL_EXEC;
use crate::wpctl::node::default_sink;
use crate::wpctl::volume::OpType::Toggle;
use std::process::Command;

const DEFAULT_MODIFY_STEP: u32 = 5;
const DEFAULT_SINK_SPECIFIER: &str = "@DEFAULT_AUDIO_SINK@";
const MAXIMUM_VOLUME: f32 = 1.5;
const MINIMUM_MODIFY_STEP: f32 = 0.01;
const MUTED_SUFFIX: &str = "[MUTED]";
const NOTIFICATION_NODE_MAX_LENGTH: usize = 21;

pub struct VolumeOp {
    op_type: OpType,
    step: Option<u32>,
}

impl VolumeOp {
    pub fn new(change_type: OpType, step: Option<u32>) -> Self {
        VolumeOp { op_type: change_type, step }
    }
}

#[derive(Debug)]
pub enum OpType {
    Dec,
    Get,
    Inc,
    Set { value: u32 },
    Show,
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

fn modify(step: Option<u32>, sign: Option<&str>) {
    let mut cmd = Command::new(WPCTL_EXEC);

    let modify_step = step.unwrap_or(DEFAULT_MODIFY_STEP);
    let mut modify_volume = f32::max(modify_step as f32 / 100.0, MINIMUM_MODIFY_STEP).to_string();
    let max_vol = MAXIMUM_VOLUME.to_string();

    if sign.is_some() {
        modify_volume.push_str(sign.unwrap());
    }

    cmd.args([
        "set-volume",
        "-l",
        max_vol.as_str(),
        DEFAULT_SINK_SPECIFIER,
        format!("{modify_volume}").as_str(),
    ]);
    cmd.status().expect("error setting volume");
}

fn modify_rel(step: Option<u32>, sign: &str) {
    modify(step, Some(sign));
}

fn modify_set(value: u32) {
    modify(Some(value), None);
}

fn toggle() {
    let mut cmd = Command::new(WPCTL_EXEC);

    cmd.args(["set-mute", DEFAULT_SINK_SPECIFIER, "toggle"]);
    cmd.status().expect("error toggling volume");
}

fn notify(volume: f32) {
    let sink = default_sink();
    let truncated = sink.chars().take(NOTIFICATION_NODE_MAX_LENGTH).collect();
    notify::volume(volume, truncated);
}

pub fn apply(change: VolumeOp) {
    let old_volume = lookup();
    match change.op_type {
        dec_or_inc @ (OpType::Dec | OpType::Inc) => {
            let sign = match dec_or_inc {
                OpType::Dec => "-",
                OpType::Inc => "+",
                _ => unreachable!("Unexpected volume change type {:?}", dec_or_inc),
            };
            modify_rel(change.step, sign);
        }
        OpType::Get => {
            println!("volume: {}", old_volume);
            return;
        }
        OpType::Set { value } => modify_set(value),
        OpType::Show => {
            notify(old_volume);
            return;
        },
        Toggle => toggle(),
    };

    let new_volume = lookup();
    if old_volume != new_volume || new_volume == 0.0 {
        notify(new_volume);
    }
}
