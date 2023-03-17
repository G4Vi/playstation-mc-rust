use std::env;
use std::fs::File;
use std::io;
use std::process;

pub mod psmc;
use crate::psmc::MemoryCard;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("mcs2raw <filename>");
        process::exit(1);
    }
    let f = File::open(&args[1])?;
    let files = [f];
    let mc = MemoryCard::from_files(&files);
    if mc.saves.len() != 1 {
        panic!("Can only convert one save to raw save")
    }
    mc.saves[0].to_raw();
    Ok(())
}
