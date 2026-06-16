mod commands;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use envtools_infrastructure::{FileStateWriter, TomlGroupRepository};

fn default_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".envtools")
}

#[derive(Parser)]
#[command(
    name = "envtools",
    version,
    about = "Environment variable group manager"
)]
struct Cli {
    /// Custom config directory
    #[arg(long, global = true)]
    config_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize envtools configuration
    Init,
    /// Manage environment variable groups
    Group {
        #[command(subcommand)]
        action: GroupAction,
    },
    /// Set variables in a group: envtools set <group> KEY=VALUE ...
    Set {
        group: String,
        #[arg(required = true)]
        vars: Vec<String>,
    },
    /// Remove variables from a group
    Unset {
        group: String,
        #[arg(required = true)]
        keys: Vec<String>,
    },
    /// Enable one or more groups
    Enable {
        #[arg(required = true)]
        names: Vec<String>,
    },
    /// Disable one or more groups
    Disable {
        #[arg(required = true)]
        names: Vec<String>,
    },
    /// Show current active environment status
    Status,
    /// Output shell hook script for integration
    Shell {
        #[command(subcommand)]
        action: ShellAction,
    },
    /// Manage hosts entries
    Hosts {
        #[command(subcommand)]
        action: HostsAction,
    },
    /// Manage profiles (group collections)
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Export configuration to JSON file
    Export {
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Export only specific groups
        #[arg(short, long)]
        groups: Vec<String>,
    },
    /// Import configuration from JSON file
    Import {
        /// Input file path
        file: PathBuf,
        /// Overwrite existing groups
        #[arg(long, default_value_t = false)]
        overwrite: bool,
    },
}

#[derive(Subcommand)]
enum GroupAction {
    /// List all groups
    List,
    /// Create a new group
    Create {
        name: String,
        #[arg(short, long, default_value = "")]
        description: String,
        /// Group kind: env or hosts
        #[arg(short, long, default_value = "env")]
        kind: String,
        #[arg(short, long, default_value_t = 0)]
        priority: u32,
    },
    /// Delete a group
    Delete { name: String },
    /// Show group details
    Show { name: String },
}

#[derive(Subcommand)]
enum HostsAction {
    /// Add a hosts entry to a group
    Add {
        group: String,
        ip: String,
        hostname: String,
    },
    /// Remove a hosts entry from a group
    Remove { group: String, hostname: String },
    /// Sync active hosts groups to system hosts file
    Sync,
}

#[derive(Subcommand)]
enum ProfileAction {
    /// List all profiles
    List,
    /// Create a new profile
    Create {
        name: String,
        #[arg(short, long, default_value = "")]
        description: String,
        /// Comma-separated group names
        #[arg(short, long, value_delimiter = ',')]
        groups: Vec<String>,
    },
    /// Delete a profile
    Delete { name: String },
    /// Show profile details
    Show { name: String },
    /// Activate all groups in a profile
    Activate { name: String },
    /// Deactivate all groups in a profile
    Deactivate { name: String },
}

#[derive(Subcommand)]
enum ShellAction {
    /// Output hook initialization script
    Init {
        /// Shell type: bash, zsh, fish, pwsh
        shell_type: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
            #[cfg(windows)]
            {
                use std::io::{self, Read};
                eprintln!("\nPress Enter to exit...");
                let _ = io::stdin().read(&mut [0u8]);
            }
            return;
        }
    };

    let config_dir = cli.config_dir.unwrap_or_else(default_config_dir);
    let config_path = config_dir.join("config.toml");

    let repo = TomlGroupRepository::new(&config_path);
    let state_writer = FileStateWriter::new(&config_dir);

    let result = match command {
        Commands::Init => commands::init(&config_dir, &repo),
        Commands::Group { action } => match action {
            GroupAction::List => commands::group_list(&repo),
            GroupAction::Create {
                name,
                description,
                kind,
                priority,
            } => commands::group_create(&repo, &name, &description, &kind, priority),
            GroupAction::Delete { name } => commands::group_delete(&repo, &name),
            GroupAction::Show { name } => commands::group_show(&repo, &name),
        },
        Commands::Set { group, vars } => commands::set_vars(&repo, &group, &vars),
        Commands::Unset { group, keys } => commands::unset_vars(&repo, &group, &keys),
        Commands::Enable { names } => commands::enable(&repo, &state_writer, &names),
        Commands::Disable { names } => commands::disable(&repo, &state_writer, &names),
        Commands::Status => commands::status(&repo),
        Commands::Shell { action } => match action {
            ShellAction::Init { shell_type } => commands::shell_init(&config_dir, &shell_type),
        },
        Commands::Hosts { action } => match action {
            HostsAction::Add {
                group,
                ip,
                hostname,
            } => commands::hosts_add(&repo, &group, &ip, &hostname),
            HostsAction::Remove { group, hostname } => {
                commands::hosts_remove(&repo, &group, &hostname)
            }
            HostsAction::Sync => commands::hosts_sync(&repo),
        },
        Commands::Profile { action } => match action {
            ProfileAction::List => commands::profile_list(&repo),
            ProfileAction::Create {
                name,
                description,
                groups,
            } => commands::profile_create(&repo, &name, &description, groups),
            ProfileAction::Delete { name } => commands::profile_delete(&repo, &name),
            ProfileAction::Show { name } => commands::profile_show(&repo, &name),
            ProfileAction::Activate { name } => {
                commands::profile_activate(&repo, &state_writer, &name)
            }
            ProfileAction::Deactivate { name } => {
                commands::profile_deactivate(&repo, &state_writer, &name)
            }
        },
        Commands::Export { output, groups } => {
            let filter = if groups.is_empty() {
                None
            } else {
                Some(groups)
            };
            commands::export_config(&repo, output.as_deref(), filter.as_deref())
        }
        Commands::Import { file, overwrite } => commands::import_config(&repo, &file, overwrite),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
