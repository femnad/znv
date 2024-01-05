use std::collections::HashMap;
use std::io::Cursor;
use std::process::Command;
use std::string::String;

extern crate skim;

use crate::VolOp::{Dec, Inc};
use clap::{Args, Parser, Subcommand};
use regex::Regex;
use skim::prelude::*;

#[derive(Debug, Parser)]
#[command(name = "znv")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Vol(VolArgs),
    Default(DefaultArgs),
}

#[derive(Args, Debug)]
struct VolArgs {
    #[command(subcommand)]
    op: Op,
}

#[derive(Args, Debug)]
struct DefaultArgs {
    #[command(subcommand)]
    node: Node,
}

#[derive(Debug, Subcommand)]
enum Node {
    Sink,
}

#[derive(Debug, Subcommand)]
enum Op {
    Dec,
    Inc,
    Toggle,
}

#[derive(Debug)]
struct Sink {
    id: u32,
    default: bool,
    name: String,
    _volume: f32,
}

#[derive(Debug)]
struct Status {
    sinks: Vec<Sink>,
}

const DEFAULT_SINK_SPECIFIER: &str = "@DEFAULT_AUDIO_SINK@";
const SINK_REGEX: &str =
    r"(?P<default>\*)?\s+(?P<id>[0-9]+)\. (?P<name>[a-zA-Z0-9() -]+) \[vol: (?P<volume>[0-9.]+)\]";

const VOLUME_MODIFY_STEP: f32 = 0.01;
const WPCTL_EXEC: &str = "wpctl";
const MUTED_SUFFIX: &str = "[MUTED]";

fn get_status() -> Status {
    let out = Command::new(WPCTL_EXEC)
        .arg("status")
        .output()
        .expect("error running wpctl");
    let mut parsing_audio = false;
    let mut parsing_sinks = false;

    let sink_regex = Regex::new(SINK_REGEX).expect("error parsing sink regex");
    let mut sinks: Vec<Sink> = vec![];

    let output = String::from_utf8(out.stdout).expect("error getting command output");
    for line in output.lines() {
        if line.starts_with::<&str>("Audio".as_ref()) {
            parsing_audio = true;
            continue;
        }
        if parsing_audio && line.ends_with::<&str>("Sinks:".as_ref()) {
            parsing_sinks = true;
            continue;
        }

        if parsing_sinks {
            if let Some(captures) = sink_regex.captures(line) {
                let default = &captures.name("default").is_some();
                let id = &captures.name("id").expect("error getting ID").as_str();
                let id_u: u32 = id.parse().expect("error parsing ID");
                let name = &captures.name("name").expect("error getting name").as_str();
                let volume = &captures
                    .name("volume")
                    .expect("error getting volume")
                    .as_str();
                let volume_f: f32 = volume.parse().expect("error parsing volume");
                let sink = Sink {
                    default: *default,
                    id: id_u,
                    name: name.trim().to_string(),
                    _volume: volume_f,
                };
                sinks.push(sink);
            } else {
                parsing_sinks = false;
            }
        }
    }

    Status { sinks }
}

fn set_default_sink() {
    let status = get_status();

    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(false)
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();

    let mut name_id_map: HashMap<String, u32> = HashMap::new();
    let mut sink_names: Vec<String> = Vec::new();

    let mut default_sink = String::new();
    for sink in status.sinks {
        if sink.default {
            default_sink = sink.name;
            continue;
        }
        name_id_map.insert(sink.name.clone(), sink.id);
        sink_names.push(sink.name.clone());
    }

    if sink_names.len() == 0 {
        println!("There's only one sink: {default_sink}");
        return;
    }

    let items = item_reader.of_bufread(Cursor::new(sink_names.join("\n")));

    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    let sel_id = selected_items
        .get(0)
        .expect("no selection")
        .output()
        .to_string();
    let sink_id = name_id_map.get(&sel_id).expect("unable to find sink ID");
    let sink_id_str = sink_id.to_string();
    let mut cmd = Command::new(WPCTL_EXEC);
    cmd.arg("set-default")
        .arg(sink_id_str)
        .status()
        .expect("error setting default sink");
}

enum VolOp {
    Dec,
    Inc,
}

fn get_volume() -> f32 {
    let mut cmd = Command::new(WPCTL_EXEC);
    cmd.args(["get-volume", DEFAULT_SINK_SPECIFIER]);
    let out = cmd.output().expect("error getting volume");
    let vol_out = String::from_utf8(out.stdout).expect("error parsing cmd output");
    let vol_out_trim = vol_out.trim();
    if vol_out_trim.ends_with(MUTED_SUFFIX) {
        return 0.0
    }
    let vol = vol_out_trim
        .split(" ")
        .nth(1)
        .expect("Error getting volume field");
    let vol_f: f32 = vol.parse().expect("error parsing volume");
    vol_f
}

fn get_volume_classifier(volume: f32) -> String {
    let vol = if volume == 0.0 {
        "muted"
    } else if volume <= 0.3 {
        "low"
    } else if volume <= 0.6 {
        "medium"
    } else if volume <= 1.0 {
        "high"
    } else {
        "overamplified"
    };
    vol.to_string()
}

fn get_icon(volume: f32) -> String {
    let classifier = get_volume_classifier(volume);
    format!("audio-volume-{}-symbolic", classifier)
}

fn notify_volume() {
    let volume = get_volume();
    let boosting = volume > 1.0;
    let boosted = if boosting { "1x boost" } else { "" };

    let mut cmd = Command::new("notify-send");
    let icon = get_icon(volume);
    let vol_int = (volume * 100.0) as u32;
    cmd.args([
        "-a",
        "volume",
        "-u",
        "low",
        "-h",
        format!("int:value:{vol_int}").as_str(),
        "-i",
        icon.as_str(),
        format!("Volume{boosted}").as_str(),
    ])
    .status()
    .expect("error notifying");
}

fn modify_volume(op: VolOp) {
    let mut cmd = Command::new(WPCTL_EXEC);

    let sign = match op {
        Dec => "-",
        Inc => "+",
    };
    cmd.args([
        "set-volume",
        "-l",
        "1.5",
        DEFAULT_SINK_SPECIFIER,
        format!("{VOLUME_MODIFY_STEP}{sign}").as_str(),
    ]);
    cmd.status().expect("error setting volume");

    notify_volume();
}

fn toggle_volume() {
    let mut cmd = Command::new(WPCTL_EXEC);

    cmd.args([
        "set-mute",
        DEFAULT_SINK_SPECIFIER,
        "toggle"
    ]);
    cmd.status().expect("error toggling volume");

    notify_volume();
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Vol(op) => match op.op {
            Op::Dec => modify_volume(Dec),
            Op::Inc => modify_volume(Inc),
            Op::Toggle => toggle_volume(),
        },
        Commands::Default(node) => match node.node {
            Node::Sink => set_default_sink(),
        },
    }
}
