use clap::Parser;

mod node;

use node::Node;


#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    period: u64,
    #[arg(long)]
    port: u16,
    #[arg(long)]
    connect: Option<String>,
}


fn main() {
    let args = Args::parse();
    let current_node = Node::new(args.port, args.period);

    current_node.start(args.connect);
}
