use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // Database File Path
    #[arg(short('d'), long)]
    pub dburl: String,
}

