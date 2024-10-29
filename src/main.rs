use std::env;
use anyhow::{Result, bail};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        bail!("incorrect argument count");
    }

    let filename = &args[1];

    let result = bgst::bgst_processing::extract_bgst(&filename);

    result
}
