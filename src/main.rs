//! A language all about iteration to play your music files.
//! This project implements the interpreter (mps-interpreter), music player (mps-player), and CLI interface for MPS (root).
//! The CLI interface includes a REPL for running scripts.
//! The REPL interactive mode also provides more details about using MPS through the `?help` command.
//!
//! # Usage
//! To access the REPL, simply run `cargo run`. You will need the [Rust toolchain installed](https://rustup.rs/).
//!
//! # Examples
//! For now, check out `./src/tests`, `./mps-player/tests`, and `./mps-interpreter/tests` for examples.
//! One day I'll add pretty REPL example pictures and some script files...
//! // TODO
//!
//! # FAQ
//! ## Is MPS Turing-Complete?
//! **No**. It can't perform arbitrary calculations (yet), which easily disqualifies MPS from being Turing-complete.
//!
//! ## Can I use MPS right now?
//! **Sure!** It's not complete, but MPS is completely useable for basic music queries right now. Hopefully most of the bugs have been ironed out as well...
//!
//! ## Why write a new language?
//! **I thought it would be fun**. I also wanted to be able to play my music without having to be at the whim of someone else's algorithm (and music), and playing just by album or artist was getting boring. I also thought designing a language specifically for iteration would be a novel approach to a language (though every approach is a novel approach for me).
//!
//! ## What is MPS?
//! **Music Playlist Script (MPS) is technically a query language for music files.** It uses an (auto-generated) SQLite3 database for SQL queries and can also directly query the filesystem. Queries can be modified by using filters, functions, and sorters built-in to MPS (see mps-interpreter's README.md).
//!
//! ## Is MPS a scripting language?
//! **No**. Technically, it was designed to be one, but it doesn't meet the requirements of a scripting language (yet). One day, I would like it be Turing-complete and then it could be considered a scripting language. At the moment it is barely a query language.
//!

mod channel_io;
mod cli;
mod help;
mod repl;

use std::io;
use std::path::PathBuf;

use mps_interpreter::MpsRunner;
use mps_player::{MpsController, MpsPlayer, PlaybackError};

#[allow(dead_code)]
fn play_cursor() -> Result<(), PlaybackError> {
    let cursor = io::Cursor::<&'static str>::new("sql(`SELECT * FROM songs JOIN artists ON songs.artist = artists.artist_id WHERE artists.name like 'thundercat'`);");
    let runner = MpsRunner::with_stream(cursor);
    let mut player = MpsPlayer::new(runner)?;
    player.play_all()
}

fn main() {
    let args = cli::parse();

    if let Some(script_file) = &args.file {
        // interpret script
        // script file checks
        if file_checks(script_file).is_err() {
            return;
        }
        // build playback controller
        let script_file2 = script_file.clone();
        let volume = args.volume.clone();
        let player_builder = move || {
            let script_reader = io::BufReader::new(
                std::fs::File::open(&script_file2)
                    .unwrap_or_else(|_| panic!("Abort: Cannot open file `{}`", &script_file2)),
            );
            let runner = MpsRunner::with_stream(script_reader);

            let player = MpsPlayer::new(runner).unwrap();
            if let Some(vol) = volume {
                player.set_volume(vol);
            }
            player
        };
        if let Some(playlist_file) = &args.playlist {
            // generate playlist
            let mut player = player_builder();
            let mut writer =
                io::BufWriter::new(std::fs::File::create(playlist_file).unwrap_or_else(|_| {
                    panic!("Abort: Cannot create writeable file `{}`", playlist_file)
                }));
            match player.save_m3u8(&mut writer) {
                Ok(_) => println!(
                    "Succes: Finished playlist `{}` from script `{}`",
                    playlist_file, script_file
                ),
                Err(e) => eprintln!("{}", e),
            }
        } else {
            // live playback
            let ctrl = MpsController::create(player_builder);
            match ctrl.wait_for_done() {
                Ok(_) => println!("Succes: Finished playback from script `{}`", script_file),
                Err(e) => eprintln!("{}", e),
            }
        }
    } else {
        // start REPL
        println!("Welcome to MPS interactive mode!");
        println!("Run ?help for usage instructions.");
        //println!("End a statement with ; to execute it.");
        repl::repl(args)
    }
}

fn file_checks(path_str: &str) -> Result<(), ()> {
    let path = PathBuf::from(path_str);
    if !path.exists() {
        eprintln!("Abort: File `{}` does not exist", path_str);
        return Err(());
    }
    if !path.is_file() {
        eprintln!("Abort: Path `{}` is not a file", path_str);
        return Err(());
    }
    Ok(())
}
