use std::{
    fs,
    process::{
        Command,
        Stdio,
        }, io::{Read, Write},
};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to the programs to run
    runfile: String,

    /// Path to the words list file
    wordsfile: String,
}

fn test(cmd: String, word: String) -> std::io::Result<i32> {
    let mut child = Command::new(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut child_stdout = child.stdout.take();
    let mut child_stdin = child.stdin.as_mut().unwrap();

    let mut i: i32 = 0;
    let mut guess = String::new();
    let mut reply: String;
    let mut matched: String;

    loop {
        reply = String::new();
        matched = String::new();
        match child_stdout {
            Some(ref mut out) => { out.read_to_string(&mut guess)? },
            None => { break }
        };

        i += 1;

        for (ch_g, ch_w) in guess.chars().zip(word.chars()) {
            if ch_g == ch_w {
                reply.push('g');
                matched.push(ch_g);
            } else if word.matches(ch_g).count() > 1 && matched.matches(ch_g).count() == 0 {
                reply.push('y');
            } else {
                reply.push('b');
            }
        }

        reply.push('\n');

        child_stdin.write_all(reply.as_bytes())?;
    }

    child.wait()?;

    Ok(i)
}

fn main() {
    let args = Args::parse();

    let runs_s = fs::read_to_string(args.runfile).expect("Failed to read file");
    let words_s = fs::read_to_string(args.wordsfile).expect("Failed to read file");

    let runs: Vec<String> = runs_s.split(" ").map(|s| s.to_string()).collect();
    let words: Vec<String> = words_s.split(" ").map(|s| s.to_string()).collect();
}
