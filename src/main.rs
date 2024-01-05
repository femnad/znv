mod notify;
mod wpctl;

extern crate skim;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "znv", version, about = "Tiny wpctl wrapper")]
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
#[command(about = "Modify/toggle volume")]
struct VolArgs {
    #[command(subcommand)]
    op: Op,
}

#[derive(Args, Debug)]
#[command(about = "Set defaults")]
struct DefaultArgs {
    #[command(subcommand)]
    node: Node,
}

#[derive(Debug, Subcommand)]
enum Node {
    #[command(about = "Set default sink")]
    Sink,
}

#[derive(Debug, Subcommand)]
enum Op {
    #[command(about = "Decrease volume")]
    Dec { step: Option<u32> },
    #[command(about = "Increase volume")]
    Inc { step: Option<u32> },
    #[command(about = "Toggle mute state")]
    Toggle,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Vol(op) => {
            let volume = match op.op {
                Op::Dec { step } | Op::Inc { step } => {
                    let sign = match op.op {
                        Op::Dec { .. } => "-",
                        Op::Inc { .. } => "+",
                        _ => unreachable!("No other Op variant should be matched here"),
                    };
                    wpctl::volume::modify(step, sign)
                }
                Op::Toggle => wpctl::volume::toggle(),
            };
            notify::volume(volume);
        }
        Commands::Default(node) => match node.node {
            Node::Sink => wpctl::sink::set_default(),
        },
    }
}
