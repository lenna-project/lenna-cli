use image::io::Reader as ImageReader;
use lenna_core::{Config, Pipeline, Pool};
use structopt::StructOpt;

mod plugins;

#[derive(StructOpt)]
#[structopt(name = "lenna-cli", about = "Command Line Interface for Lenna")]
struct Cli {
    #[structopt(
        parse(from_os_str),
        short = "c",
        long = "config",
        default_value = "lenna.yml"
    )]
    config: std::path::PathBuf,
    #[structopt(parse(from_os_str), required_unless = "list-plugins")]
    path: Option<std::path::PathBuf>,
    #[structopt(
        parse(from_os_str),
        short = "o",
        long = "output",
        default_value = "output"
    )]
    out_path: std::path::PathBuf,
    #[structopt(
        parse(from_os_str),
        short = "p",
        long = "plugins",
        default_value = "plugins/"
    )]
    plugins: std::path::PathBuf,
    #[structopt(long = "list-plugins")]
    list_plugins: bool,
    #[structopt(short, long)]
    verbose: bool,
}

fn main() {
    let args = Cli::from_args();
    let config_file = std::fs::File::open(&args.config).unwrap();
    let config: Config = serde_yaml::from_reader(config_file).unwrap();
    let mut pool = Pool::default();
    let mut plugins = plugins::Plugins::new();
    plugins.load_plugins(&mut pool, &args.plugins);

    if args.list_plugins {
        for plugin_id in pool.ids() {
            println!("{}", plugin_id);
            if args.verbose {
                match pool.get(&plugin_id) {
                    Some(plugin) => println!("\t{}\n", plugin.description()),
                    _ => (),
                }
            }
        }
    } else {
        let mut img = ImageReader::open(&args.path.unwrap())
            .unwrap()
            .decode()
            .unwrap();

        let pipeline = Pipeline::new(config, pool);
        img = pipeline.run(img);
        img.save(&args.out_path).unwrap();
    }
}
