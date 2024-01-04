use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "vnz")]
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
    op: Op
}

#[derive(Args, Debug)]
struct DefaultArgs {
    #[command(subcommand)]
    node: Node
}

#[derive(Debug, Subcommand)]
enum Node {
    Sink,
    Source,
}

#[derive(Debug, Subcommand)]
enum Op {
    Dec,
    Inc,
    Toggle,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Vol(op) => {
            match op.op {
                Op::Dec => {println!("dec")}
                Op::Inc => {println!("inc")}
                Op::Toggle => {println!("tog")}
            }
        }
        Commands::Default(node) => {
            match node.node {
                Node::Sink => {println!("sink")}
                Node::Source => {println!("source")}
            }
        }
    }
}
