use std::io::Write;

use clap::{Parser, ValueEnum};
use ros_monitor_lib::state::RosState;
use ros_monitor_lib::types::DiscoveryEventWrapper;
use state::RosStateProvider;

mod state;

#[derive(Parser, Debug)]
#[command(disable_help_flag = true)]
#[command(disable_version_flag = true)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Arguments {
    #[arg(global = true, long, help = "ROS2 node name", default_value = "/intrepid/_discovery")]
    node: String,
    #[arg(global = true, short, long, help = "graph update interval in milliseconds", default_value = "800")]
    interval: u64,
    #[arg(global = true, short, long, help = "output format", default_value = "json")]
    format: OutputFormat,
    #[arg(global = true, short, long, help = "print this help message", action = clap::ArgAction::Help)]
    help: Option<bool>,
    #[arg(short = 'V', long, help = "print intrepid agent version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    Bitcode,
}

fn parse_name(node: &str) -> (&str, &str) {
    let parts: Vec<&str> = node.split('/').filter(|s| !s.is_empty()).collect();
    #[allow(clippy::get_first)]
    (parts.get(0).copied().unwrap_or_default(), parts.get(1).copied().unwrap_or_default())
}

fn main() {
    let args = Arguments::parse();
    let (name, namespace) = parse_name(&args.node);

    let ros2_ctx = r2r::Context::create().unwrap();
    let ros2_node = r2r::Node::create(ros2_ctx.clone(), name, namespace).unwrap();
    let mut state = RosState::default();
    let mut stdout = std::io::stdout();
    let mut bitcode_buffer = bitcode::Buffer::new();

    loop {
        let new_state = RosState::from_ros(&ros2_node).unwrap();
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
        let events = new_state.changes(&state);
        state = new_state;
        for event in events {
            let event = DiscoveryEventWrapper { ts, event };
            match args.format {
                OutputFormat::Json => {
                    stdout.write_all(&serde_json::to_vec(&event).unwrap()).unwrap();
                    stdout.write_all(b"\n").unwrap();
                }
                OutputFormat::Bitcode => {
                    let buffer = bitcode_buffer.encode(&event);
                    let buffer_size = buffer.len();
                    stdout.write_all(&(buffer_size as u32).to_le_bytes()).unwrap();
                    stdout.write_all(buffer).unwrap();
                }
            }
            stdout.flush().unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(args.interval));
    }
}
