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
}

#[derive(Subcommand)]
pub enum FeedCommands {
    Now,
}

#[derive(Subcommand)]
pub enum RunCommands {
    AiVision,
}
