use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Status,
    Feed {
        #[command(subcommand)]
        action: FeedCommands,
    },
    Run {
        #[command(subcommand)]
        action: RunCommands,
    },
    Serve {
        #[command(subcommand)]
        action: ServeCommands,
    },
}

#[derive(Subcommand)]
pub enum FeedCommands {
    Now,
}

#[derive(Subcommand)]
pub enum RunCommands {
    AiVision {
        #[arg(long)]
        image: Option<String>,
        #[arg(long, default_value_t = 0)]
        camera_index: u32,
        #[arg(long, default_value_t = 1)]
        frames: u32,
    },
}

#[derive(Subcommand)]
pub enum ServeCommands {
    Actuators,
}
