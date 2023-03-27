use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(version = "1.0", author = "Stanisław Zagórowski")]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clone, Parser)]
enum Subcommand {
    Create { message: String },
    Get { id: Option<usize> },
    Update { id: usize, message: String },
}

fn main() {
    let args = Args::parse();

    let subcommand = args.subcommand;

    let jiroscope = jiroscope_core::Jiroscope::new();

    match subcommand {
        Subcommand::Create { message } => {
            match jiroscope.register_note(message) {
                Ok(note) => println!("Note created with id: {}", note.id.unwrap()),
                Err(e) => println!("Error: {}", e),
            }
        }
        Subcommand::Get { id } => match id {
            Some(id) => match jiroscope.get_note_by_id(id) {
                Ok(note) => println!("Note: {}", note.message),
                Err(e) => println!("Error: {}", e),
            },
            None => match jiroscope.get_notes() {
                Ok(notes) => {
                    for note in notes {
                        println!("Note: {}", note.message);
                    }
                }
                Err(e) => println!("Error: {}", e),
            },
        },
        Subcommand::Update { id, message } => {
            match jiroscope.update_note_by_id(id, message) {
                Ok(note) => println!("Note id: {} updated", note.id.unwrap()),
                Err(e) => println!("Error: {}", e),
            }
        }
    }
}
