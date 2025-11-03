use clap::{Parser, Subcommand};
use mogbox_io::AudioFile;
use mogbox_runtime::AudioPlayer;

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
    // Play an audio file
    Play {
        #[arg(value_name = "PATH")]
        path: std::path::PathBuf,
    },
}

fn main() {
    let args: Cli = Cli::parse();
    print_intro(&args);

    match args.command {
        Commands::Info { path } => handle_info(path),
        Commands::Play { path } => handle_play(path),
    }
}

// Command Handlers
fn handle_info(path: std::path::PathBuf) {
    print_read_file(&path);

    match AudioFile::open(&path) {
        Ok(audio_file) => {
            println!("Sample Rate: {} Hz", audio_file.sample_rate);
            println!("Channels: {}", audio_file.channels);
            println!("Track ID: {}", audio_file.track_id);
        }
        Err(e) => {
            eprintln!("Error opening audio file: {}", e);
        }
    }
}

fn handle_play(path: std::path::PathBuf) {
    print_read_file(&path);

    match AudioFile::open(&path) {
        Ok(audio_file) => {
            println!(
                "Playing: {} Hz, {} channels\n",
                audio_file.sample_rate, audio_file.channels
            );

            match AudioPlayer::play(audio_file) {
                Ok(_player) => {
                    println!("Playback started. Press Ctrl+C to stop.");
                    // Keep the player alive until interrupted
                    std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
                }
                Err(e) => {
                    eprintln!("Error during playback: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error opening audio file: {}", e);
        }
    }
}

// Display Utils
fn print_intro(args: &Cli) {
    println!("==================");
    println!("<<< MogBox CLI >>>");
    println!("==================\n");

    println!("Command: {:?}", args.command);
}

fn print_read_file(path: &std::path::PathBuf) {
    println!("Reading File: {:?}", path)
}
