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

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to the programs to run
    runfile: PathBuf,

    /// Path to the words list file
    wordsfile: PathBuf,
}

fn start_process(sender: Sender<String>, reciever: Receiver<String>, cmd: &Path, args: Vec<&str>) {
    let child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect(&format!("Failed to launch {:?}", cmd));

    println!("Started process: {:?} : {}", cmd, child.id());

    thread::spawn(move || {
        let mut f = BufReader::new(child.stdout.unwrap());
        let mut stdin = child.stdin.unwrap();
        // for line in reciever {
        //     let mut buf = String::new();
        //     match f.read_line(&mut buf) {
        //         Ok(_) => {
        //             sender.send(buf).unwrap();
        //         },
        //         Err(e) => {
        //             println!("Error!: {:?}", e);
        //             break;
        //         }
        //     }
        //     stdin.write_all(line.as_bytes()).unwrap();
        // }

        let mut buf = String::new();
        while f.read_line(&mut buf).is_ok() {
            sender.send(buf.clone()).unwrap();
            let recvd = reciever.recv().unwrap();
            println!("{}", recvd);
            stdin.write_all(recvd.as_bytes()).unwrap();
            buf = String::new();
        }
    });
}

fn test(cmd: &Path, wordsfile: &Path, word: String) -> std::io::Result<i32> {
    println!("{:?}, {:?}", cmd, wordsfile);

    let mut i: i32 = 0;
    let mut reply: String;
    let mut matched: String;

    let (tx1, rx1): (Sender<String>, Receiver<String>) = channel();
    let (tx2, rx2): (Sender<String>, Receiver<String>) = channel();

    start_process(tx1, rx2, cmd, vec![wordsfile.to_str().unwrap()]);

    for mut guess in rx1 {
        println!("{}", guess);
        reply = String::new();
        matched = String::new();

        guess = guess.chars().map(|c| c.to_ascii_lowercase()).collect();

        i += 1;

        for (ch_g, ch_w) in guess.chars().zip(word.chars()) {
            println!("{}:{}", ch_g, ch_w);
            if ch_g == ch_w {
                reply.push('g');
                matched.push(ch_g);
            } else if word.find(ch_g).is_some() && matched.find(ch_g).is_none() {
                reply.push('y');
            } else {
                reply.push('b');
            }
            println!("{}, {}", reply, matched);
        }

        reply.push('\n');

        tx2.send(reply).unwrap();
    }

    Ok(i)
}

fn main() {
    let args = Args::parse();

    let runs_s = fs::read_to_string(&args.runfile).expect("Failed to read file");
    let words_s = fs::read_to_string(&args.wordsfile).expect("Failed to read file");

    let runspath = &args.runfile.parent().unwrap();
    let runs: Vec<PathBuf> = runs_s.split("\n").map(|s| PathBuf::from(s)).collect();
    let words: Vec<String> = words_s.split(" ").map(|s| s.to_string()).collect();

    for c in runs.iter() {
        let i = test(&runspath.join(c), &args.wordsfile, "franc".to_string()).unwrap();
        println!("{:?}: {}", c, i);
    }
}
