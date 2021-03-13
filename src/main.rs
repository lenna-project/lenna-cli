use structopt::StructOpt;
use image::io::Reader as ImageReader;
use lenna_core::{Config, Pool, Pipeline};

mod plugins;

#[derive(StructOpt)]
#[structopt(name = "lenna-cli", about = "Command Line Interface for Lenna")]
struct Cli {
    #[structopt(parse(from_os_str), short = "c", long = "config", default_value = "lenna.yml")]
    config: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
    #[structopt(parse(from_os_str), short = "o", long = "output")]
    out_path: std::path::PathBuf,
    #[structopt(parse(from_os_str), short = "p", long = "plugins", default_value = "plugins/")]
    plugins: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();
    let config_file = std::fs::File::open(&args.config).unwrap();
    let config: Config = serde_yaml::from_reader(config_file).unwrap();
    let mut img = ImageReader::open(&args.path).unwrap().decode().unwrap();
    let mut pool = Pool::default();
    let mut plugins = plugins::Plugins::new();
    plugins.load_plugins(&mut pool, &args.plugins);
    let pipeline = Pipeline::new(config, pool);
    img = pipeline.run(img);
    img.save(&args.out_path).unwrap();
}
