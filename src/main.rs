use clap::{
    Parser,
    arg
};
use asset_graph_rs::database::Database;

#[derive(Parser)]
struct CliArgs {
    #[arg(long, short = 'p')]
    root_path: String,
    #[arg(long, short = 'r', default_value = None)]
    relative_to: Option<String>,
}

fn main() {
    let args = CliArgs::parse();

    let mut db = match Database::new(&args.root_path, args.relative_to.as_deref()) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error initializing database: {}", e);
            std::process::exit(1);
        }
    };

    match db.populate() {
        Ok(_) => println!("DB populated with {} assets in {} roots", db.assets().count(), db.roots().len()),
        Err(e) => {
            eprintln!("Error populating database: {}", e);
            std::process::exit(1);
        }
    }

    let file = std::fs::File::create("db.json").expect("Failed to create db.json");
    let mut writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &db).expect("Failed to write database to db.json");

    // let file = std::fs::File::create("db.bin").expect("Failed to create db.bin");
    // let mut writer = std::io::BufWriter::new(file);
    // rmp_serde::encode::write(&mut writer, &db).expect("Failed to write database to db.bin");
}
