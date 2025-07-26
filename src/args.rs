use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // Database File Path
    #[arg(short('d'), long)]
    pub dburl: String,

    //JWT Encoding Key File Path
    #[arg(short('e'), long)]
    pub jwt_en_key: String,

    //JWT Decoding Key File Path
    #[arg(short('d'), long)]
    pub jwt_de_key: String,

    //TLS Encoding Key File Path,
    #[arg(short('k'), long)]
    pub tls_key: String,

    //TLS Decoding Key File Path,
    #[arg(short('c'), long)]
    pub tls_cert: String,
}
