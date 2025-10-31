use clap::Parser;

#[derive(Parser)]
struct CmdArg {
    // Which operation to perform on the file
    cmd: String,
    // The path to the file we want to operate on
    path: std::path::PathBuf,
}

fn main() {
    let args = CmdArg::parse();
    println!("Command: {:?} File Path: {:?}", args.cmd, args.path)
}
