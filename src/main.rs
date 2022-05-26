use std::env;

mod ui;

fn main() -> anyhow::Result<()> {
    let filename = parse_args();

    let app = ui::App::new(filename)?;
    let res = app.run();
    println!("res = {:?}", res);

    Ok(())
}

fn parse_args() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("no filename specified");
    }
    let filename = &args[1];
    filename.to_string()
}
