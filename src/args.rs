use clap::Parser;




#[derive(Parser)]
pub struct CgiDArgs {
    #[clap(long)]
    /// The address to listen for persistent server mode
    pub listen_addr: Option<String>,

    #[clap(long)]
    /// The name of the FD provided by systemd to use in single connection mode
    pub socket_name: Option<String>,

    #[clap(long)]
    /// The base directory in which to locate scripts
    pub root_path: String

}