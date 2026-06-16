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
        #[arg(short, long, default_value_t = 0)]
        priority: u32,
    },
    /// Delete a group
    Delete { name: String },
    /// Show group details
    Show { name: String },
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
                priority,
            } => commands::group_create(&repo, &name, &description, priority),
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
