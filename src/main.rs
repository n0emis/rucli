use clap::Parser;

use rucli::ssh::{SSHConnection, self};
use rucli::netconf::NETCONFClient;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    name: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match cli.name {
        Some(x) => {
            println!("name: {}", x);
        },
        None => {
            println!("No value");
        },
    }

    let mut ssh_connection = SSHConnection::new("johann", "62.176.232.99:22");
    ssh_connection.connect();

    
    let netconf_session = NETCONFClient::new(ssh_connection.sess);

}