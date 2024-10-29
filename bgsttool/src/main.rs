use std::env;
use anyhow::{Result, bail};
use bgst;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("incorrect argument count");
    }

    let filename = &args[1];
    
    let mut should_mask = false;

    if args.len() == 3 {
        should_mask = &args[2] == "mask";
    }

    let result = bgst::extract_bgst(&filename, should_mask);

    result
}
