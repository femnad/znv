use notify_rust::Notification;
use regex::Regex;
use skim::prelude::{SkimItemReader, SkimOptionsBuilder};
use skim::Skim;
use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::process::{Command, Stdio};
use std::str::FromStr;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::wpctl::WPCTL_EXEC;

const NODE_REGEX: &str = r"(?P<default>\*)?\s+(?P<id>[0-9]+)\. (?P<name>[a-zA-Z0-9()+/ -]+) \[vol: (?P<volume>[0-9.]+)\]";

enum NodeType {
    Sink,
    Source,
}

impl FromStr for NodeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sink" => Ok(NodeType::Sink),
            "source" => Ok(NodeType::Source),
            _ => Err(()),
        }
    }
}

#[derive(Tabled)]
struct Node {
    #[tabled(order = 0, rename = "ID")]
    id: u32,
    #[tabled(order = 3, rename = "Default")]
    default: bool,
    #[tabled(order = 1, rename = "Name")]
    name: String,
    #[tabled(order = 2, rename = "Volume")]
    volume: f32,
}

pub struct Status {
    sinks: Vec<Node>,
    sources: Vec<Node>,
}

fn get_status() -> Status {
    let out = Command::new(WPCTL_EXEC)
        .arg("status")
        .output()
        .expect("error running wpctl");
    let mut parsing_audio = false;
    let mut parsing_sinks = false;
    let mut parsing_sources = false;

    let node_regex = Regex::new(NODE_REGEX).expect("error parsing node regex");
    let mut sinks: Vec<Node> = vec![];
    let mut sources: Vec<Node> = vec![];

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
        if parsing_audio && line.ends_with::<&str>("Sources:".as_ref()) {
            parsing_sources = true;
            continue;
        }

        if parsing_sinks || parsing_sources {
            let nodes = if parsing_sinks {
                &mut sinks
            } else {
                &mut sources
            };

            if let Some(captures) = node_regex.captures(line) {
                let default = &captures.name("default").is_some();
                let id = &captures.name("id").expect("error getting ID").as_str();
                let id_u: u32 = id.parse().expect("error parsing ID");
                let name = &captures.name("name").expect("error getting name").as_str();
                let volume = &captures
                    .name("volume")
                    .expect("error getting volume")
                    .as_str();
                let volume_f: f32 = volume.parse().expect("error parsing volume");
                let node = Node {
                    default: *default,
                    id: id_u,
                    name: name.trim().to_string(),
                    volume: volume_f,
                };
                nodes.push(node);
            } else {
                parsing_sinks = false;
            }
        }
    }

    Status { sinks, sources }
}

fn print_nodes(nodes: &Vec<Node>, header: &str) {
    if nodes.len() == 0 {
        return;
    }

    let mut table = Table::new(nodes);
    table.with(Style::blank());
    println!("{header}:");
    println!("{table}");
}

pub fn print_status() {
    let status = get_status();
    print_nodes(&status.sinks, "Sinks");
    if status.sinks.len() > 0 && status.sinks.len() > 0 {
        println!()
    }
    print_nodes(&status.sources, "Sources");
}

fn select_with_rofi(node_names: Vec<String>, prompt: &str) -> Option<String> {
    let rofi = Command::new("rofi")
        .args(["-p", prompt, "-dmenu"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("error running rofi");

    let input = node_names.join("\n");
    rofi.stdin
        .as_ref()
        .expect("error getting stdin of rofi")
        .write_all(input.as_ref())
        .expect("error writing to stdin of rofi");

    let output = rofi
        .wait_with_output()
        .expect("error waiting for rofi to complete");
    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn select_with_skim(node_names: Vec<String>, prompt: &str) -> Option<String> {
    let prompt = if prompt.ends_with(": ") {
        prompt.to_string()
    } else {
        format!("{prompt}: ")
    };
    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .multi(false)
        .prompt(Some(prompt.as_str()))
        .build()
        .unwrap();
    let item_reader = SkimItemReader::default();

    let items = item_reader.of_bufread(Cursor::new(node_names.join("\n")));

    let skim_out = Skim::run_with(&options, Some(items)).expect("error selecting with skim");
    if skim_out.is_abort {
        return None;
    }

    let selection = skim_out
        .selected_items
        .get(0)
        .expect("Skim not aborted but there's no selection")
        .output()
        .to_string();
    Some(selection)
}

fn inform(msg: &str, prefer_gui: bool) {
    if !prefer_gui && atty::is(atty::Stream::Stdout) {
        println!("{}", msg);
        return;
    }

    Notification::new()
        .summary("znv")
        .body(msg)
        .show()
        .expect("error showing informational message");
}

fn set_default_node(nodes: Vec<Node>, node_type: &str, prefer_gui: bool) {
    let mut name_id_map: HashMap<String, u32> = HashMap::new();
    let mut node_names: Vec<String> = Vec::new();

    let mut default_node_name = String::new();
    for node in &nodes {
        if node.default {
            default_node_name = node.name.clone();
            continue;
        }
        name_id_map.insert(node.name.clone(), node.id);
        node_names.push(node.name.clone());
    }

    if nodes.len() == 1 {
        let sole_node = nodes.get(0).expect("unable to get the only node");
        inform(
            format!("There's only one {node_type}: {}", sole_node.name).as_str(),
            prefer_gui,
        );
        return;
    }

    if node_names.len() == 0 {
        inform(
            format!("There's only one non-default {node_type}: {default_node_name}").as_str(),
            prefer_gui,
        );
        return;
    }

    let prompt = format!("Set default {node_type}");
    let maybe_node_id = if !prefer_gui && atty::is(atty::Stream::Stdout) {
        select_with_skim(node_names, prompt.as_str())
    } else {
        select_with_rofi(node_names, prompt.as_str())
    };

    if maybe_node_id.is_none() {
        return;
    }

    let node_id = name_id_map
        .get(&maybe_node_id.unwrap())
        .expect(format!("unable to find {node_type} ID").as_str());

    let mut cmd = Command::new(WPCTL_EXEC);
    cmd.arg("set-default")
        .arg(node_id.to_string())
        .status()
        .expect(format!("error setting default %s {node_type}").as_str());
}

pub fn reset_default() {
    let mut cmd = Command::new(WPCTL_EXEC);
    cmd.arg("clear-default")
        .status()
        .expect("error clearing default configured nodes");
}

pub fn set_default(node_type: &str, prefer_gui: bool) {
    let status = get_status();

    let parsed_type: Result<NodeType, ()> = node_type.parse();
    let parsed_type = match parsed_type {
        Ok(nt) => Some(nt),
        Err(_) => None,
    }
    .expect("error determining node type");

    let nodes = match parsed_type {
        NodeType::Sink => status.sinks,
        NodeType::Source => status.sources,
    };

    set_default_node(nodes, node_type, prefer_gui);
}
