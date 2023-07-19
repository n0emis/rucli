use core::time;
use std::{env, fs, thread};

use clap::{Parser, Subcommand, ValueEnum, ArgAction};

use rucli::netconf::NETCONFClient;
use rucli::ssh::{get_config_for_host, SSHConnection};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    hostname: String,

    #[arg(long, short)]
    user: Option<String>,

    #[arg(long, short, action=ArgAction::SetTrue)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Format {
    Text,
    JSON,
    // TODO: We need to get into quick-xml to find out how to handle internal XML as strings.
    // XML
}

#[derive(Subcommand)]
enum Commands {
    /// Executes an given command on the router
    Exec {
        #[clap(value_enum)]
        format: Format,

        command: Vec<String>,
    },

    /// Applies local configuration file on router
    Apply { local_file: String },

    /// Loads local configuration onto router and shows a diff
    Check { local_file: String },
}

fn main() {
    let cli = Cli::parse();

    let config = get_config_for_host(cli.hostname.as_str());

    let ssh_user = match cli.user {
        Some(user) => user,
        None => config.clone().unwrap_or_else(|| (String::new(), env::var("USER").unwrap())).1
    };

    let hostname = match config {
        Some((host, _user)) => host,
        None => cli.hostname
    };

    let mut ssh_connection =
        SSHConnection::new(ssh_user.as_str(), format!("{}:830", hostname).as_str(), cli.debug);
    ssh_connection.connect().unwrap();

    let mut netconf_session = NETCONFClient::new(ssh_connection.channel.expect(""));
    netconf_session.init().unwrap();

    match cli.command {
        Commands::Exec { format, command } => {
            let format_str = match format {
                Format::Text => "text",
                Format::JSON => "json",
            };

            let command_str = command.join(" ").to_owned();

            let r = netconf_session
                .send_command(command_str, format_str.to_owned())
                .unwrap();

            println!("{}", r)
        }
        Commands::Apply { local_file } => {
            let data = fs::read_to_string(local_file).unwrap();

            let _ = netconf_session.lock_configuration().unwrap();

            let load_config_reply = netconf_session.load_configuration(data).unwrap();
            println!("{}", load_config_reply);

            let diff_reply = netconf_session
                .diff_configuration("text".to_string())
                .unwrap();
            println!("{}", diff_reply);

            println!("Applying configuration and waiting some time...");

            let apply_reply = netconf_session.apply_configuration().unwrap();
            println!("{}", apply_reply);

            let ten_millis = time::Duration::from_secs(5);
            thread::sleep(ten_millis);

            println!("Confirming configuration");

            let confirm_reply = netconf_session.confirm_configuration().unwrap();
            println!("{}", confirm_reply);

            let _ = netconf_session.unlock_configuration().unwrap();
        }
        Commands::Check { local_file } => {
            let data = fs::read_to_string(local_file).unwrap();

            let _ = netconf_session.lock_configuration().unwrap();

            let load_config_reply = netconf_session.load_configuration(data).unwrap();
            println!("{}", load_config_reply);

            let diff_reply = netconf_session
                .diff_configuration("text".to_string())
                .unwrap();
            println!("{}", diff_reply);

            let _ = netconf_session.unlock_configuration().unwrap();
        }
    }
}
