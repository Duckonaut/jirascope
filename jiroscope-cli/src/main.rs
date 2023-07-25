use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(version = "1.0", author = "Stanisław Zagórowski")]
struct Args {
    server: String,
    user: String,
    api_token: String,
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clone, Parser)]
enum Subcommand {
    Issue { board_id: String, issue_id: String },
    All,
    CreateMeta,
    EditMeta { board_id: String, issue_id: String },
    Events,
}

fn main() {
    let args = Args::parse();

    let subcommand = args.subcommand;

    let config = jiroscope_core::Config::new(args.server);
    let auth = jiroscope_core::Auth::new(args.user, args.api_token);

    let mut jiroscope = jiroscope_core::Jiroscope::new(config, auth);
    jiroscope.init().unwrap();

    match subcommand {
        Subcommand::Issue { board_id, issue_id } => {
            let issue = jiroscope
                .get_issue(format!("{}-{}", board_id, issue_id).as_str())
                .unwrap();
            println!("{:#?}", issue);
        }
        Subcommand::All => {
            let issues = jiroscope.get_all_issues().unwrap();
            println!("{:#?}", issues);
        }
        Subcommand::CreateMeta => {
            let meta = jiroscope.get_issue_creation_meta().unwrap();
            println!("{:#?}", meta);
        }
        Subcommand::EditMeta { board_id, issue_id } => {
            let meta = jiroscope
                .get_issue_edit_meta(format!("{}-{}", board_id, issue_id).as_str())
                .unwrap();
            println!("{:#?}", meta);
        }
        Subcommand::Events => {
            let events = jiroscope.get_issue_events().unwrap();
            println!("{:#?}", events);
        }
    }
}
