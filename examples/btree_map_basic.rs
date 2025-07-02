use std::fmt::Display;

use datastructurs::btree::{BTreeMap, DEFAULT_BRANCH_FACTOR};
use datastructurs::vec::Vec;
use rand::{distr::StandardUniform, prelude::*, random, random_range};

const SHEEP_WORDS: &[&str] = &[
    "Bah",
    "Baah",
    "Baaah",
    "Mäh",
    "Määh",
    "Määäh",
    "Baaaah",
    "Wool",
    "Grass",
    "Herd",
    "Lamb",
    "Longwool",
    "Ram",
    "Ewe",
    "Yoe",
    "Dodl",
    "Bock",
    "Rida",
    "Merino",
    "Baa",
    "Ba",
    "Mä",
    "Mäe",
    "Wolle",
    "Yarn",
    "Baha",
    "Baahaa",
    "Määhää",
    "Mähä",
    "Mäha",
];

#[derive(Clone, Debug, Copy)]
enum SheepKind {
    Rainbow,
    White,
    Brown,
    Black,
}

#[derive(Clone, Debug)]
struct Sheep {
    age: u8,
    kind: SheepKind,
    text: String,
    name: String,
}

impl Distribution<SheepKind> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SheepKind {
        match rng.random_range(0..=3) {
            0 => SheepKind::Rainbow,
            1 => SheepKind::White,
            2 => SheepKind::Brown,
            3 => SheepKind::Black,
            _ => unreachable!(),
        }
    }
}

impl Distribution<Sheep> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, _rng: &mut R) -> Sheep {
        let name: String = sheepspeech(random_range(2..=3));
        let text: String = sheepspeech(random_range(4..=10));
        let kind = random();

        Sheep {
            age: random_range(0..=20),
            kind,
            text,
            name,
        }
    }
}

impl Display for SheepKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Display for Sheep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {} years old, is a {} sheep and says \"{}\"",
            self.name, self.age, self.kind, self.text
        )
    }
}

fn sheepspeech(n: usize) -> String {
    let mut rng = rand::rng();
    SHEEP_WORDS
        .choose_multiple(&mut rng, n)
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    let sheeps = 2000;

    let mut bm = BTreeMap::new(DEFAULT_BRANCH_FACTOR);
    let mut sheep_ids: Vec<String> = Vec::new();

    for _ in 0..sheeps {
        let sid = sheepspeech(6);
        sheep_ids.push(sid.clone());
        let sheep: Sheep = random();
        bm.insert(sid, sheep);
    }

    println!("Layout: {:#?}", bm);

    for i in (0..sheeps) {
        let sid = &sheep_ids[i];
        println!("\"{sid}\": {}", bm.get(sid).unwrap())
    }

    println!("Amount of sheep: {}", bm.len());
    assert_eq!(bm.len(), sheeps);
}
