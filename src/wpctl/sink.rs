use std::collections::HashMap;
use std::io::Cursor;
use std::process::Command;
use regex::Regex;
use skim::prelude::{SkimItemReader, SkimOptionsBuilder};
use skim::Skim;

use crate::wpctl::WPCTL_EXEC;

const SINK_REGEX: &str =
    r"(?P<default>\*)?\s+(?P<id>[0-9]+)\. (?P<name>[a-zA-Z0-9() -]+) \[vol: (?P<volume>[0-9.]+)\]";

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

pub fn set_default() {
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
