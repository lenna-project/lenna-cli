use lenna_cli::{plugins, zip_images};
use lenna_core::{Config, Pipeline};
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
    let mut plugins = plugins::Plugins::new();
    plugins.load_plugins(&args.plugins);

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
    } else {
        let path = &args.path.unwrap();
        let mut img = Box::new(
            lenna_core::io::read::read_from_file(path.to_str().unwrap().to_string()).unwrap(),
        );

        let pipeline = Pipeline::new(config, plugins.pool);
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
                img.path = args.out_path.parent().unwrap().to_str().unwrap().into();
                match ext {
                    "zip" => {
                        img.name = format!("{}.jpg", img.name);
                        let images = vec![&mut img];
                        let file = std::fs::File::create(&args.out_path).unwrap();
                        zip_images(
                            images,
                            image::ImageOutputFormat::Jpeg(80),
                            file,
                            zip::CompressionMethod::DEFLATE,
                        )
                        .unwrap();
                    }
                    "png" | "PNG" => {
                        lenna_core::io::write::write_to_file(&img, image::ImageOutputFormat::Png)
                            .unwrap();
                    }
                    _ => {
                        lenna_core::io::write::write_to_file(
                            &img,
                            image::ImageOutputFormat::Jpeg(80),
                        )
                        .unwrap();
                    }
                }
            }
        };
    }
}
