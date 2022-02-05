use std::{
    fs,
    process::{
        Command, Stdio,
        },
    io::{
        BufRead, BufReader, Write
    },
    path::{
        Path, PathBuf,
    },
    sync::mpsc::{
            Sender, Receiver, channel
        },
    thread
};
use clap::Parser;
use rand::seq::SliceRandom;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to the programs to run
    runfile: PathBuf,

    /// Path to the words list file
    wordsfile: PathBuf,
}

fn start_process(sender: Sender<String>, reciever: Receiver<String>, cmd: &Path, args: Vec<&str>) {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect(&format!("Failed to launch {:?}", cmd));

    println!("Started process: {:?} : {}", cmd, child.id());

    thread::spawn(move || {
        let mut f = BufReader::new(child.stdout.take().unwrap());
        let mut stdin = child.stdin.take().unwrap();

        let mut buf = String::new();
        while f.read_line(&mut buf).is_ok() {
            sender.send(buf.clone()).unwrap();
            if let Ok(Some(s)) = child.try_wait() {
                break;
            }
            let recvd = reciever.recv().unwrap();
            print!("{}", recvd);
            stdin.write_all(recvd.as_bytes()).unwrap();
            buf = String::new();
        }
    });
}

#[derive(Debug)]
enum TestError {
    EarlyExit,
}


fn test(cmd: &Path, wordsfile: &Path, word: String) -> Result<i32, TestError> {
    println!("{:?}, {:?}", cmd, wordsfile);

    let mut i: i32 = 0;
    let mut reply: String;
    let mut matched: String;

    let (tx1, rx1): (Sender<String>, Receiver<String>) = channel();
    let (tx2, rx2): (Sender<String>, Receiver<String>) = channel();

    start_process(tx1, rx2, cmd, vec![wordsfile.to_str().unwrap()]);

    for mut guess in rx1 {
        guess.pop();
        println!("{}", guess);
        reply = String::new();
        matched = String::new();

        guess = guess.chars().map(|c| c.to_ascii_lowercase()).collect();

        for (ch_g, ch_w) in guess.chars().zip(word.chars()) {
            if ch_g == ch_w {
                reply.push('g');
                matched.push(ch_g);
            } else if word.find(ch_g).is_some() && matched.find(ch_g).is_none() {
                reply.push('y');
            } else {
                reply.push('b');
            }
        }

        reply.push('\n');
        let tx_r = tx2.send(reply.clone());
        i += 1;

        if reply == "ggggg\n" {
            return Ok(i);
        } else if tx_r.is_err() {
            return Err(TestError::EarlyExit);
        }
    }

    Ok(i)
}

fn main() {
    let args = Args::parse();

    let runs_s = fs::read_to_string(&args.runfile).expect("Failed to read file");
    let words_s = fs::read_to_string(&args.wordsfile).expect("Failed to read file");

    let runspath = &args.runfile.parent().unwrap();
    let runs: Vec<PathBuf> = runs_s.split("\n").map(|s| PathBuf::from(s)).collect();
    let words: Vec<String> = words_s.split("\n").map(|s| s.to_string()).collect();

    println!("{:?}", runs);

    for c in runs.iter() {
        if *c != PathBuf::from("") {
            let i = test(&runspath.join(c), &args.wordsfile, words.choose(&mut rand::thread_rng()).unwrap().clone()).unwrap();
            println!("{:?}: {:?}", c, i);
        }
    }
}
