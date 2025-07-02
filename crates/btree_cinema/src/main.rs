use std::io::Write;

use datastructurs::btree::BTreeSet;
use getopts::Options;
use rand::prelude::IteratorRandom;

pub type Item = u8;

#[derive(Debug)]
#[allow(unused)] // we use Debug
enum Action {
    Insert(Item),
    Remove(Item),
}

fn wait(automatic: Option<u64>) {
    if let Some(ms) = automatic {
        println!();
        clear();
        std::thread::sleep(std::time::Duration::from_millis(ms));
    } else {
        (std::io::stdin().read_line(&mut String::new())).unwrap();
    }
}

fn clear() {
    print!("{}[2J", 27 as char);
}

fn fill_ratio(len: usize, base_ratio: f64) -> f64 {
    if len == 0 {
        return 1.0;
    }
    if len >= 512 {
        return 1.0;
    }
    (1.0 - (len as f64 / Item::MAX as f64) * (base_ratio)).clamp(0.0, 1.0)
}

fn render(tree: &BTreeSet<Item>, fill: f64, tick: usize, action: Action) {
    clear();
    println!("{tree}\n\n");
    println!("fill: {fill:.02}");
    println!("len: {}", tree.len());
    println!("nodes: {}", tree.node_count());
    println!("action: {action:?}");
    println!("tick: {tick}");
    print!("branching factor: {}", tree.branching_factor());

    std::io::stdout().flush().expect("could not flush stdout")
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {program} [options]");
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag(
        "r",
        "random",
        "insert random items instead of incremental modulo items",
    );
    opts.optopt("a", "auto", "tick without user input each n MS", "MS");
    opts.optflag("h", "help", "print this help menu");
    opts.optopt(
        "b",
        "branching-factor",
        "set branching factor of the BTree",
        "BRANCHING_FACTOR",
    );

    opts.optopt(
        "t",
        "ratio",
        "ratio of insert to remove (usually 1.0)",
        "RATIO",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{f}");
            std::process::exit(1)
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    cinema(
        matches.opt_present("random"),
        matches.opt_get("auto").expect("could not parse ms"),
        matches
            .opt_get_default("branching-factor", 3)
            .expect("could not parse branching factor"),
        matches
            .opt_get_default("ratio", 1.0)
            .expect("could not parse branching factor"),
    );
}

fn cinema(random_insert: bool, auto: Option<u64>, branching_factor: usize, ratio: f64) {
    let mut tree = BTreeSet::<Item>::new(branching_factor);
    let mut fill = 1.0;
    let mut action;
    let mut tick: usize = 0;
    loop {
        if rand::random_bool(fill) {
            let item: Item = if random_insert {
                rand::random()
            } else {
                (tick % Item::MAX as usize) as u8
            };
            action = Action::Insert(item);
            tree.insert(item);
        } else {
            let key = tree.iter().choose(&mut rand::rng()).cloned();
            if key.is_none() {
                continue;
            }
            let key = key.unwrap();
            action = Action::Remove(key);
            tree.remove(&key);
        }

        render(&tree, fill, tick, action);
        wait(auto);
        fill = fill_ratio(tree.len(), ratio);
        tick += 1;
    }
}
