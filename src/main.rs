use clap::{Parser, Subcommand};
use rust_clippy::{Result, decode, delete_last, list, store};
use std::env;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cliphist")]
#[command(about = "A clipboard history manager")]
#[command(version = "1.0.0")]
struct Cli {
    /// Maximum number of items to store
    #[arg(long, default_value = "750")]
    max_items: u64,

    /// Maximum number of last items to look through when finding duplicates
    #[arg(long, default_value = "100")]
    max_dedupe_search: u64,

    /// Maximum number of characters to preview
    #[arg(long, default_value = "100")]
    preview_width: u32,

    /// Path to db
    #[arg(long)]
    db_path: Option<PathBuf>,

    /// Overwrite config path to use instead of cli flags
    #[arg(long)]
    config_path: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Store clipboard content
    Store,
    /// List clipboard history
    List,
    /// Decode clipboard entry
    Decode {
        /// Entry ID to decode
        id: Option<String>,
    },
    /// Delete clipboard entry
    Delete,
    /// Delete by query
    DeleteQuery {
        /// Query string
        query: String,
    },
    /// Wipe all clipboard history
    Wipe,
    /// Show version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get cache and config directories (equivalent to Go's os.UserCacheDir() and os.UserConfigDir())
    let cache_home = env::var("XDG_CACHE_HOME")
        .or_else(|_| env::var("HOME").map(|home| format!("{}/.cache", home)))
        .unwrap_or_else(|_| "/tmp".to_string());

    let config_home = env::var("XDG_CONFIG_HOME")
        .or_else(|_| env::var("HOME").map(|home| format!("{}/.config", home)))
        .unwrap_or_else(|_| "/tmp".to_string());

    // Set default paths if not provided
    let db_path = cli
        .db_path
        .unwrap_or_else(|| PathBuf::from(&cache_home).join("cliphist").join("db"));

    let config_path = cli
        .config_path
        .unwrap_or_else(|| PathBuf::from(&config_home).join("cliphist").join("config"));

    // Print configuration for debugging
    println!("Configuration:");
    println!("  max-items: {}", cli.max_items);
    println!("  max-dedupe-search: {}", cli.max_dedupe_search);
    println!("  preview-width: {}", cli.preview_width);
    println!("  db-path: {:?}", db_path);
    println!("  config-path: {:?}", config_path);

    // Handle clipboard state environment variable (equivalent to Go's os.Getenv("CLIPBOARD_STATE"))
    let clipboard_state = env::var("CLIPBOARD_STATE").unwrap_or_default();

    // Match on the command (equivalent to Go's switch statement)
    match cli.command {
        Commands::Store => match clipboard_state.as_str() {
            "sensitive" => {
                // sensitive, skip storage
            }
            "clear" => {
                let _ = delete_last(db_path.as_path());
            }
            _ => {
                let res = store(
                    db_path.as_path(),
                    std::io::stdin(),
                    cli.max_dedupe_search,
                    cli.max_items,
                );

                if res.is_err() {
                    println!("decode result: {:?}", res);
                }
            }
        },
        Commands::List => {
            let _ = list(
                db_path.as_path(),
                std::io::stdout(),
                cli.preview_width as u64,
            );
        }
        Commands::Decode { id } => {
            let res = decode(db_path.as_path(), std::io::stdin(), std::io::stdout(), id);
            // TODO: remove this eventually, perhaps different return code
            if res.is_err() {
                println!("decode result: {:?}", res);
            }
        }
        Commands::DeleteQuery { query } => {
            println!("Delete-query command: Deleting by query");
            println!("  Would call: deleteQuery({:?}, {:?})", db_path, query);
        }
        Commands::Delete => {
            println!("Delete command: Deleting clipboard entry");
            println!("  Would call: delete({:?}, stdin)", db_path);
        }
        Commands::Wipe => {
            println!("Wipe command: Wiping all clipboard history");
            println!("  Would call: wipe({:?})", db_path);
        }
        Commands::Version => {
            println!("Version command: Showing version information");
            println!("version\t1.0.0");
            println!("max-items\t{}", cli.max_items);
            println!("max-dedupe-search\t{}", cli.max_dedupe_search);
            println!("preview-width\t{}", cli.preview_width);
            println!("db-path\t{:?}", db_path);
            println!("config-path\t{:?}", config_path);
        }
    }

    Ok(())
}
