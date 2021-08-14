use lenna_cli::{images_in_path, plugins, write_to_path};
use lenna_core::{Config, Pipeline};
use std::env;
use structopt::StructOpt;

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
    #[structopt(
        parse(from_os_str),
        required_unless = "list-plugins",
        required_unless = "example-config"
    )]
    path: Option<std::path::PathBuf>,
    #[structopt(
        parse(from_os_str),
        short = "o",
        long = "output",
        default_value = "output"
    )]
    out_path: std::path::PathBuf,
    #[structopt(parse(from_os_str), short = "p", long = "plugins")]
    plugins: Option<std::path::PathBuf>,
    #[structopt(long = "list-plugins")]
    list_plugins: bool,
    #[structopt(short, long)]
    verbose: bool,
    #[structopt(long = "example-config")]
    example_config: bool,
}

fn main() {
    let args = Cli::from_args();
    let config_file = std::fs::File::open(&args.config).unwrap();
    let config: Config = serde_yaml::from_reader(config_file).unwrap();
    let mut plugins = plugins::Plugins::new();
    let plugins_path = match args.plugins {
        Some(path) => path,
        None => match env::var("LENNA_PLUGINS") {
            Ok(val) => std::path::PathBuf::from(val),
            _ => std::path::PathBuf::from("plugins/"),
        },
    };

    plugins.load_plugins(&plugins_path);

    if args.list_plugins {
        for plugin_id in plugins.pool.ids() {
            println!("{}", plugin_id);
            if args.verbose {
                match plugins.pool.get(&plugin_id) {
                    Some(plugin) => println!("\t{}\n", plugin.description()),
                    _ => (),
                }
            }
        }
    } else if args.example_config {
        let lenna_yml = include_str!("../../lenna.yml");
        print!("{}", lenna_yml);
    } else {
        let pipeline = Pipeline::new(config, plugins.pool);
        let path = &args.path.unwrap();
        for path in images_in_path(path) {
            if args.verbose {
                println!("{}", path.to_str().unwrap());
            }
            let mut img = Box::new(
                lenna_core::io::read::read_from_file(path.to_str().unwrap().to_string()).unwrap(),
            );
            pipeline.run(&mut img).unwrap();

            let out_path = args.out_path.to_str().unwrap().to_string();
            match args.out_path.is_dir() {
                true => {
                    img.path = out_path;
                    lenna_core::io::write::write_to_file(&img, image::ImageOutputFormat::Jpeg(80))
                        .unwrap();
                }
                false => {
                    let ext = args.out_path.extension().unwrap().to_str().unwrap();
                    img.name = args.out_path.file_stem().unwrap().to_str().unwrap().into();
                    write_to_path(img, out_path, ext.to_string());
                }
            };
        }
    }
}
