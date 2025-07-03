use clap::{
    command,
    Parser,
    Subcommand,
    arg
};
use uuid::Uuid;
use asset_graph_rs::{
    database::Database,
    id::Id,
};

#[derive(Parser)]
struct CliArgs {
    #[command(subcommand)]
    command: CliCommand,
    #[arg(long, short = 'd', default_value = "db.json")]
    db_path: String,
}

#[derive(Subcommand)]
enum CliCommand {
    FindAssets {
        #[arg(long, short = 'p')]
        root_path: String,
        #[arg(long, short = 'r', default_value = None)]
        relative_to: Option<String>,
    },
    ResolveAssets,
    Info {
        #[arg(long, short = 'i')]
        id: Uuid,
    }
}

fn main() {
    let args = CliArgs::parse();
    match args.command {
        CliCommand::FindAssets { root_path, relative_to } => {
            find_assets(args.db_path, root_path, relative_to);
        },
        CliCommand::ResolveAssets => {
            resolve_assets(args.db_path);
        },
        CliCommand::Info { id } => {
            info(&args.db_path, id);
        }
    }
}

fn find_assets(db_path: String, root_path: String, relative_to: Option<String>) {
    let mut db = match Database::new(&root_path, relative_to.as_deref()) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error initializing database: {}", e);
            std::process::exit(1);
        }
    };

    match db.find_assets() {
        Ok(_) => println!("DB populated with {} assets in {} roots", db.assets().count(), db.roots().len()),
        Err(e) => {
            eprintln!("Error populating database: {}", e);
            std::process::exit(1);
        }
    }

    let file = std::fs::File::create(&db_path).expect("Failed to create db.json");
    let mut writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &db).expect("Failed to write database to db.json");
}

fn resolve_assets(db_path: String) {
    let file = std::fs::File::open(&db_path).expect("Failed to open db.json");
    let mut db: Database = match serde_json::from_reader(file) {
        Ok(db) => {
            println!("Loaded database from {}", db_path);
            db
        },
        Err(e) => {
            eprintln!("Error reading database from db.json: {}", e);
            std::process::exit(1);
        }
    };

    db.resolve_assets();

    let file = std::fs::File::create(&db_path).expect("Failed to create db.json");
    let mut writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &db).expect("Failed to write database to db.json");
}

fn info(db_path: &str, id: Uuid) {
    let file = std::fs::File::open(&db_path).expect("Failed to open db.json");
    let db: Database = match serde_json::from_reader(file) {
        Ok(db) => {
            println!("Loaded database from {}", db_path);
            db
        },
        Err(e) => {
            eprintln!("Error reading database from db.json: {}", e);
            std::process::exit(1);
        }
    };

    match db.asset(&Id::new_uuid(id)) {
        None => {
            eprintln!("No asset found with ID: {}", id);
            std::process::exit(1);
        },
        Some(asset) => {
            println!("{asset}");
        },
    }
}