use std::env;

mod ui;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("no filename specified");
    }
    let filename = &args[1];

    let app = ui::App::new(filename.clone())?;
    app.run()?;
    Ok(())
}
