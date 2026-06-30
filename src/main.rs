use clap::{
    Parser,
    Subcommand,
};
use std::{
    collections::{HashMap, HashSet},
};
use unity_asset_graph::{AssetType, Database, DatabaseFile, Id, Relation, BoundAsset, BoundRelation, AssetFilter};

#[derive(Parser)]
struct CliArgs {
    #[command(subcommand)]
    command: CliCommand,
    #[arg(long, short = 'd', default_value = "db.bin", help = "Path to the database file (default: db.bin)")]
    db_path: String,
}

#[derive(Subcommand)]
enum CliCommand {
    #[command(about = "Find assets in a Unity project directory and create a database file")]
    Build {
        #[arg(long, short = 'p', help = "Path to the directory containing a Unity project")]
        root_path: String,
        #[arg(long, short = 'r', default_value = ".", help = "If supplied, make paths in the database relative to this path")]
        relative_to: String,
    },
    #[command(about = "Get information about specific assets by ID or name")]
    Info {
        #[arg(long, short, help = "Partial ID of the asset")]
        id: Option<String>,
        #[arg(long, short, help = "Partial path of the asset")]
        path: Option<String>,
        #[arg(long, help = "Show the list of detected package roots")]
        roots: bool,
    },
    #[command(about = "Find unused assets in the database")]
    Unused {
        #[arg(long, help = "Filter by ID type: 'guid' or 'loc'")]
        id_type: Option<OrphanFilter>,
        #[arg(long, default_value = "false", help = "Only print IDs of unused assets")]
        id_only: bool,
        #[arg(long, default_value = "false", help = "Only print totals")]
        summarize: bool,
    },
    #[command(about = "Find broken references in the database")]
    Broken {
        #[arg(long, help = "Filter by ID type: 'guid' or 'loc'")]
        id_type: Option<OrphanFilter>,
        #[arg(long, default_value = "false", help = "If true, only print IDs of broken references")]
        id_only: bool,
    },
    /// Show usages by in-group assets of out-group assets
    Outside {
        /// Assets recursively contained by this partial ID are "in"
        #[arg(long)]
        in_id: Vec<String>,

        /// Assets recursively contained by this partial path are "in"
        #[arg(long)]
        in_path: Vec<String>,

        /// Only show out-group assets with this partial id
        #[arg(long)]
        out_id: Vec<String>,

        /// Only show out-group assets with this partial path
        #[arg(long)]
        out_path: Vec<String>,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum OrphanFilter {
    UnityGuid,
    Loc,
    CsDeclaration,
}

impl OrphanFilter {
    pub fn matches(&self, id: &Id) -> bool {
        match self {
            OrphanFilter::UnityGuid => match id {
                Id::Guid(_) => true,
                _ => false,
            },
            OrphanFilter::Loc => match id {
                Id::Loc(_) => true,
                _ => false,
            },
            OrphanFilter::CsDeclaration => match id {
                Id::CsType { .. } => true,
                _ => false,
            },
        }
    }
}

impl From<String> for OrphanFilter {
    fn from(value: String) -> Self {
        if value.eq_ignore_ascii_case("guid") {
            OrphanFilter::UnityGuid
        } else if value.eq_ignore_ascii_case("loc") {
            OrphanFilter::Loc
        } else {
            panic!("Invalid orphan filter type: {}", value);
        }
    }
}

impl From<&Id> for OrphanFilter {
    fn from(value: &Id) -> Self {
        match value {
            Id::None => panic!("Cannot convert Id::None to OrphanFilter"),
            Id::Guid(_) => Self::UnityGuid,
            Id::Loc(_) => Self::Loc,
            Id::CsType { .. } => Self::CsDeclaration,
        }
    }
}

impl std::fmt::Display for OrphanFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnityGuid => write!(f, "Unity object"),
            Self::Loc => write!(f, "Localized string"),
            Self::CsDeclaration => write!(f, "C# declaration"),
        }
    }
}

fn main() {
    let args = CliArgs::parse();
    match args.command {
        CliCommand::Build { root_path, relative_to } => {
            find_assets(args.db_path, root_path, Some(relative_to));
        },
        CliCommand::Info { id, path, roots } => {
            info(&args.db_path, id, path, roots);
        },
        CliCommand::Unused { id_type, id_only, summarize} => {
            find_unused(&args.db_path, id_type, id_only, summarize);
        },
        CliCommand::Broken { id_type, id_only } => {
            find_broken_refs(&args.db_path, id_type, id_only);
        },
        CliCommand::Outside { in_id, in_path, out_id, out_path } => {
            find_outside_refs(&args.db_path, in_id, in_path, out_id, out_path);
        }
    }
}

fn find_assets(db_path: String, root_path: String, relative_to: Option<String>) {
    let mut db = Database::new(&root_path, relative_to.as_deref()).expect("Error initializing database");

    if let Err(e) = db.populate() {
        panic!("Error finding assets: {}", e);
    }

    DatabaseFile::from(db).save(&db_path).expect("Error saving database file");
}

fn info(db_path: &str, id: Option<String>, path: Option<String>, roots: bool) {
    let db = DatabaseFile::load(db_path)
        .expect(format!("Failed to load database file from {}", db_path).as_str())
        .database;

    if roots {
        let mut sorted_roots: Vec<String> = db.roots().iter().map(|r| r.display().to_string()).collect();
        sorted_roots.sort();
        for r in &sorted_roots {
            println!("- {r}");
        }
    }
    else if let Some(id) = id {
        let id = id.try_into().unwrap_or_else(|e| panic!("Invalid regular expression: {e}"));
        let assets = db.find_assets_by_id(&id);
        if assets.len() == 0 {
            panic!("No assets found with id: {id}");
        } else {
            for a in assets {
                println!("{}", a.display_full());
            }
        }
    }
    else if let Some(path) = path {
        let path = path.replace('/', &regex::escape(std::path::MAIN_SEPARATOR_STR))
            .try_into()
            .unwrap_or_else(|e| panic!("Invalid regular expression: {e}"));
        let assets = db.find_assets_by_path(&path);
        if assets.len() == 0 {
            panic!("No assets found with path: {path}");
        } else {
            for a in assets {
                println!("{}", a.display_full());
            }
        }
    }
    else {
        panic!("One of --id or --path must be provided");
    }
    
}

fn find_unused(db_path: &str, id_type: Option<OrphanFilter>, id_only: bool, summarize: bool) {
    let db = DatabaseFile::load(db_path)
        .expect(format!("Failed to load database file from {}", db_path).as_str())
        .database;

    let mut orphans = HashMap::new();
    let mut types: HashMap<OrphanFilter, usize> = HashMap::new();
    for asset in db.assets() {
        if let Some(id_type) = id_type && !id_type.matches(&asset.id()) {
            continue;
        }

        if asset.asset().relations_iter().all(|r| !matches!(r, Relation::UsedBy(_))) {
            orphans.insert(asset.id().clone(), asset.clone());

            let type_class: OrphanFilter = asset.id().into();
            let count = types.get(&type_class).unwrap_or(&0);
            types.insert(type_class, count + 1);
        }
    }

    println!("Unused assets ({}):", orphans.len());
    if summarize {
        for (t, count) in &types {
            println!("  {t}: {count}");
        }
    }
    else {
        for asset in orphans.values() {
            if id_only {
                println!("{}", asset.id());
            }
            else {
                println!("{}", asset.clone().indent().display_full());
            }
        }
    }
    if orphans.is_empty() {
        println!("No unused assets found.");
    }
}

fn find_broken_refs(db_path: &str, id_type: Option<OrphanFilter>, id_only: bool) {
    let db = DatabaseFile::load(db_path)
        .expect(format!("Failed to load database file from {}", db_path).as_str())
        .database;

    let mut broken_refs = HashMap::new();
    for asset in db.assets() {
        if let Some(id_type) = id_type {
            if id_type == OrphanFilter::UnityGuid && let Id::Loc(_) = asset.id() {
                continue;
            }
            if id_type == OrphanFilter::Loc && let Id::Guid(_) = asset.id() {
                continue;
            }
        }

        if asset.asset_type() == &AssetType::BrokenRef {
            broken_refs.insert(asset.id().clone(), asset);
        }
    }

    println!("\nBroken references ({}):", broken_refs.len());
    for asset in broken_refs.values() {
        if id_only {
            println!("{}", asset.id());
        }
        else {
            println!("{}", asset.clone().indent().display_full());
        }
    }
    if broken_refs.is_empty() {
        println!("No broken references found.");
    }
}

fn find_outside_refs(db_path: &str, in_id: Vec<String>, in_path: Vec<String>, out_id: Vec<String>, out_path: Vec<String>) {
    let db = DatabaseFile::load(db_path)
        .expect(format!("Failed to load database file from {}", db_path).as_str())
        .database;

    let mut roots = HashSet::new();
    for id in in_id {
        let id = id.try_into().unwrap_or_else(|e| panic!("Invalid regular expression: {e}"));
        roots.extend(db.find_assets_by_id(&id));
    }
    for path in in_path.into_iter().map(|p| p.replace("/", &regex::escape(std::path::MAIN_SEPARATOR_STR))) {
        let path = path.try_into().unwrap_or_else(|e| panic!("Invalid regular expression: {e}"));
        roots.extend(db.find_assets_by_path(&path));
    }
    if roots.is_empty() {
        panic!("At least one container asset must be specified via --id or --path");
    }

    let out_id: Vec<_> = out_id.into_iter()
        .filter_map(|p| {
            let re = AssetFilter::try_from(p.as_str());
            if let Err(e) = &re {
                eprintln!("Supplied out-group ID is not a valid regular expression: {p}\r\n{e}");
            }
            re.ok()
        })
        .collect();

    let out_path: Vec<_> = out_path.into_iter()
        .filter_map(|p| {
            let p = p.replace("/", &regex::escape(std::path::MAIN_SEPARATOR_STR));
            let re =  AssetFilter::try_from(p.as_str());
            if let Err(e) = &re {
                eprintln!("Supplied out-group path is not a valid regular expression: {p}\r\n{e}");
            }
            re.ok()
        })
        .collect();

    let root_len = roots.len();
    let mut inside = HashMap::new();
    for root in roots {
        inside = find_all(root, inside);
    }
    println!("In-group contains {} assets from {root_len} containers", inside.len());

    let mut outside = HashMap::new();
    for asset in inside.values() {
        for relation in asset.relations_iter() {
            if let BoundRelation::Uses(asset) = relation
                && !inside.contains_key(asset.id())
                && (out_id.len() == 0 || out_id.iter().any(|re| asset.asset().id_matches(re)))
                && (out_path.len() == 0 || out_path.iter().any(|re| asset.path_matches(re))) {
                outside.insert(asset.id().clone(), asset);
            }
        }
    }

    println!("Outside references ({}):", outside.len());
    for outside in outside.values() {
        println!("{}", outside.display_full_filtered(|r| {
            if let BoundRelation::UsedBy(a) = r && inside.contains_key(a.id()) {
                true
            } else {
                false
            }
        }));
    }
}

fn find_all<'a>(asset: BoundAsset<'a>, mut results: HashMap<Id, BoundAsset<'a>>) -> HashMap<Id, BoundAsset<'a>> {
    for relation in asset.relations_iter() {
        if let BoundRelation::Contains(other) = relation {
            results = find_all(other, results);
        }
    }
    results.insert(asset.id().clone(), asset);
    results
}

