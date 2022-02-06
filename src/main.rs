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
    thread, collections::HashMap
};
use clap::Parser;
use rand::seq::SliceRandom;
use futures::future::join_all;
use tokio;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to the programs to run
    runfile: PathBuf,

    /// Path to the words list file
    wordsfile: PathBuf,
}

fn start_process(sender: Sender<Option<String>>, reciever: Receiver<Option<String>>, cmd: &str, args: Vec<&str>) {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect(&format!("Failed to launch {:?}", cmd));

    thread::spawn(move || {
        let mut f = BufReader::new(child.stdout.take().unwrap());
        let mut stdin = child.stdin.take().unwrap();

        let mut buf = String::new();
        while f.read_line(&mut buf).is_ok() {
            if let Ok(Some(_)) = child.try_wait() {
                break;
            }
            sender.send(Some(buf.clone())).unwrap();
            let recvd = reciever.recv().unwrap();
            if let Some(r) = recvd {
                stdin.write_all(r.as_bytes()).unwrap();
            } else {
                child.kill().unwrap();
                break;
            }
            buf = String::new();
        }
    });
}

#[derive(Debug)]
enum TestError {
    EarlyExit,
}


async fn test(cmd: &str, wordsfile: &Path, word: String) -> Result<i32, TestError> {
    println!("Testing: {}", word);
    let mut i: i32 = 0;
    let mut reply: String;
    let mut matched: String;

    let (tx1, rx1): (Sender<Option<String>>, Receiver<Option<String>>) = channel();
    let (tx2, rx2): (Sender<Option<String>>, Receiver<Option<String>>) = channel();

    start_process(tx1, rx2, cmd, vec![wordsfile.to_str().unwrap()]);

    for g in rx1 {
        if let Some(mut guess) = g {
            guess.pop();
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
            i += 1;

            if reply == "ggggg\n" {
                let _tx_r = tx2.send(Some(reply.clone()));
                let _tx_r = tx2.send(None);
                println!("Done: {}", word);
                return Ok(i);
            } else {
                let tx_r = tx2.send(Some(reply.clone()));
                if let Err(_) = tx_r {
                    println!("Done: {}", word);
                    return Err(TestError::EarlyExit);
                }
            }
        } else {
            break;
        }
    }

    println!("Done: {}", word);

    Ok(i)
}

async fn test_multi(cmd: String, wordsfile: &Path, words: Vec<&String>) -> i32 {
    let mut score: i32 = 0;
    let mut tests = vec![];
    for word in words.iter() {
        tests.push(test(&cmd, wordsfile, word.to_string()))
    }
    let results = join_all(tests.into_iter()).await;
    for r in results {
        if let Ok(i) = r {
            score += i;
        } else {
            score += 20;
        }
    }
    return score;
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let runs_s = fs::read_to_string(&args.runfile).expect("Failed to read file");
    let words_s = fs::read_to_string(&args.wordsfile).expect("Failed to read file");

    let runspath = &args.runfile.parent().unwrap();
    let runs: Vec<PathBuf> = runs_s.split("\n").map(|s| PathBuf::from(s)).collect();
    let words: Vec<String> = words_s.split("\n").map(|s| s.to_string()).collect();

    let mut scores: HashMap<String, i32> = HashMap::new();
    let mut futures = Vec::new();
    let test_words: Vec<&String> = words.choose_multiple(&mut rand::thread_rng(), 20).collect();

    for c in runs.iter() {
        if *c != PathBuf::from("") {
            let runner = runspath.join(c).to_str().unwrap().to_string();
            futures.push(test_multi(runner, &args.wordsfile, test_words.clone()));
        }
    }

    let results = join_all(futures).await;

    for (i, r) in results.into_iter().enumerate() {
        scores.insert(runs[i].to_str().unwrap().to_string(), r);
    }

    println!("{:?}", scores)
}
