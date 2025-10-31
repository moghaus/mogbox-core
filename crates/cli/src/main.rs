use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "MogBox")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // Print info and metadata for an audio file
    Info {
        #[arg(value_name = "PATH")]
        path: std::path::PathBuf,
    },
}

fn main() {
    let args: Cli = Cli::parse();
    print_intro(&args);

    match args.command {
        Commands::Info { path } => handle_info(path),
    }
}

// Command Handlers
fn handle_info(path: std::path::PathBuf) {
    print_read_file(path);
}

// Display Utils
fn print_intro(args: &Cli) {
    println!("==================");
    println!("<<< MogBox CLI >>>");
    println!("==================\n");

    println!("Command: {:?}", args.command);
}

fn print_read_file(path: std::path::PathBuf) {
    println!("Reading File: {:?}", path)
}
