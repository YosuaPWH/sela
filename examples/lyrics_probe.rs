fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("Usage: cargo run --example lyrics_probe -- TITLE ARTIST SECONDS");
        std::process::exit(2);
    }
    let key = sela_notch::lyrics::TrackKey {
        title: args[0].clone(),
        artist: args[1].clone(),
        duration: args[2].parse().expect("duration must be seconds"),
    };
    match sela_notch::lyrics::Client::new().and_then(|c| c.lookup(&key)) {
        Ok(sela_notch::lyrics::Data::Synced(lines)) => {
            println!("LRCLIB: {} synchronized lyric lines received", lines.len())
        }
        Ok(sela_notch::lyrics::Data::Plain(_)) => println!("LRCLIB: plain lyrics received"),
        Ok(other) => println!("LRCLIB: {other:?}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
