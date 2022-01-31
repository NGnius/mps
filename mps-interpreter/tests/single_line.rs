use mps_interpreter::lang::MpsLanguageError;
use mps_interpreter::tokens::{MpsToken, MpsTokenizer, ParseError};
use mps_interpreter::*;
use std::collections::VecDeque;
use std::io::Cursor;

#[test]
fn parse_line() -> Result<(), ParseError> {
    let cursor = Cursor::new("sql(`SELECT * FROM songs;`)");
    let correct_tokens: Vec<MpsToken> = vec![
        MpsToken::Name("sql".into()),
        MpsToken::OpenBracket,
        MpsToken::Literal("SELECT * FROM songs;".into()),
        MpsToken::CloseBracket,
    ];

    let mut tokenizer = MpsTokenizer::new(cursor);
    let mut buf = VecDeque::<MpsToken>::new();
    tokenizer.read_line(&mut buf)?; // operation being tested

    // debug output
    println!("Token buffer:");
    for i in 0..buf.len() {
        println!("  Token #{}: {}", i, &buf[i]);
    }

    // validity tests
    assert_eq!(buf.len(), correct_tokens.len());
    for i in 0..buf.len() {
        assert_eq!(
            buf[i], correct_tokens[i],
            "Tokens at position {} do not match ()",
            i
        );
    }

    tokenizer.read_line(&mut buf)?; // this should immediately return
    Ok(())
}

#[inline(always)]
fn execute_single_line(
    line: &str,
    should_be_emtpy: bool,
    should_complete: bool,
) -> Result<(), Box<dyn MpsLanguageError>> {
    println!("--- Executing MPS code: '{}' ---", line);
    let cursor = Cursor::new(line);

    let tokenizer = MpsTokenizer::new(cursor);
    let interpreter = MpsInterpretor::with_standard_vocab(tokenizer);

    let mut count = 0;
    for result in interpreter {
        if let Ok(item) = result {
            count += 1;
            if count > 100 {
                if should_complete {
                    continue; // skip println, but still check for errors
                } else {
                    println!("Got 100 items, stopping to avoid infinite loop");
                    break;
                }
            } // no need to spam the rest of the songs
            println!(
                "Got song `{}` (file: `{}`)",
                item.field("title")
                    .expect("Expected field `title` to exist")
                    .clone()
                    .to_str()
                    .expect("Expected field `title` to be String"),
                item.field("filename")
                    .expect("Expected field `filename` to exist")
                    .clone()
                    .to_str()
                    .expect("Expected field `filename` to be String")
            );
        } else {
            println!("!!! Got error while iterating (executing) !!!");
            eprintln!("{}", result.as_ref().err().unwrap());
            result?;
        }
    }
    if should_be_emtpy {
        assert_eq!(
            count, 0,
            "{} music items found while iterating over line which should be None",
            count
        );
    } else {
        println!(
            "Got {} items, execution complete (no songs were harmed in the making of this test)",
            count
        );
        assert_ne!(
            count, 0,
            "0 music items found while iterating over line which should have Some results"
        ); // assumption: database is populated
    }
    Ok(())
}

#[test]
fn execute_sql_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("sql(`SELECT * FROM songs ORDER BY artist;`)", false, true)
}

#[test]
fn execute_simple_sql_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("song(`lov`)", false, true)
}

#[test]
fn execute_comment_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("// this is a comment", true, true)?;
    execute_single_line("# this is a special comment", true, true)
}

#[test]
fn execute_repeat_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "repeat(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`))",
        false,
        false,
    )?;
    execute_single_line(
        "repeat(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), 4)",
        false,
        true,
    )?;
    execute_single_line(
        "repeat(files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`), 0)",
        true,
        true,
    )
}

#[test]
fn execute_sql_init_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "sql_init(generate = false, folder = `/home/ngnius/Music`)",
        true,
        true,
    )
}

#[test]
fn execute_assign_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "let some_var = repeat(song(`Christmas in L.A.`))",
        true,
        true,
    )?;
    execute_single_line("let some_var2 = 1234", true, true)
}

#[test]
fn execute_emptyfilter_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).().().()",
        false,
        true,
    )
}

#[test]
fn execute_fieldfilter_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year >= 2000)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year <= 2020)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year == 2016)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year != 2048)",
        false,
        true,
    )
}

#[test]
fn execute_fieldfiltermaybe_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year? >= 2000)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year? <= 2020)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year! == 2016)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year! != `test`)",
        false,
        true,
    )
}

#[test]
fn execute_files_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        r"files(folder=`~/Music/MusicFlac/Bruno Mars/24K Magic/`, re=``, recursive=false)",
        false,
        true,
    )?;
    execute_single_line(
        r"files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)",
        false,
        true,
    )?;
    execute_single_line(r"files()", false, true)
}

#[test]
fn execute_indexfilter_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(2)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(0)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(!0)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(200)",
        true,
        true,
    )
}

#[test]
fn execute_rangefilter_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(..)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(0..=4)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(..=4)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(0..5)",
        false,
        true,
    )
}

#[test]
fn execute_orfilter_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(4 || 5)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year != 2020 || 5)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(year != 2020 || 5 || 4 || 12)",
        false,
        true,
    )
}

#[test]
fn execute_replacefilter_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(if 4: files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(5))",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(if 4: files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(5) else item.())",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(if 4: item.() else files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(0 || 1).(if 200: files() else repeat(item.(), 2)))",
        false,
        true,
    )
}

#[test]
fn execute_emptysort_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).sort()",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~()",
        false,
        true,
    )
}

#[test]
fn execute_likefilter_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(not_a_field? like `24K Magic`)",
        true,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(not_a_field! like `24K Magic`)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).(album like `24K Magic`)",
        false,
        true,
    )
}

#[test]
fn execute_fieldsort_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~(title)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`).sort(not_a_field)",
        false,
        true,
    )
}

#[test]
fn execute_blissfirstsort_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~(advanced bliss_first)",
        false,
        true,
    )
}

#[test]
fn execute_blissnextsort_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~(advanced bliss_next)",
        false,
        true,
    )
}

#[test]
fn execute_emptyfn_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("empty()", true, true)
}

#[test]
fn execute_resetfn_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line("reset(empty())", true, true)
}

#[test]
fn execute_shufflesort_line() -> Result<(), Box<dyn MpsLanguageError>> {
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~(random shuffle)",
        false,
        true,
    )?;
    execute_single_line(
        "files(`~/Music/MusicFlac/Bruno Mars/24K Magic/`)~(shuffle)",
        false,
        true,
    )?;
    execute_single_line("empty()~(shuffle)", true, true)
}
