use std::{env, process};

mod ui;

fn main() -> anyhow::Result<()> {

let filename = parse_args()
    .unwrap_or_else(|err| {
        println!("Wrong input arguments: {}", err);
        process::exit(1);
    });

    let app = ui::App::new(filename)?;
    let res = app.run();
    println!("res = {:?}", res);

    Ok(())
}

fn parse_args() -> Result<String, &'static str> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("no filename specified");
    }
    let filename = &args[1];
    Ok(filename.to_string())
}
