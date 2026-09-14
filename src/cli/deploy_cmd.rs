use crate::cli::version::VERSION;
use crate::parser::AstNode;
use crate::{deploy, find_cronus_file, parser};
use std::fs;

fn write_file(path: &str, contents: &str) {
    if let Err(e) = fs::write(path, contents) {
        eprintln!("  \x1b[31m✗\x1b[0m Failed to write {}: {}", path, e);
        std::process::exit(1);
    }
    println!("  \x1b[32m✓\x1b[0m Generated {}", path);
}

fn print_image_notes(db_path: Option<&str>) {
    println!(
        "  \x1b[90mImage pins cronus {} from github.com/{} (override: --build-arg CRONUS_VERSION=x.y.z)\x1b[0m",
        VERSION,
        deploy::RELEASE_REPO
    );
    if let Some(warning) = deploy::database_outside_volume(db_path) {
        println!("  \x1b[33m⚠\x1b[0m {}", warning);
    }
}

pub fn cmd_deploy(args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found");
        std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Cannot read {}: {}", file, e);
        std::process::exit(1);
    });
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
        std::process::exit(1);
    });
    let mut app_name = "cronus-app".to_string();
    let mut port: u16 = 5175;
    let mut db_path: Option<String> = None;
    for node in &nodes {
        if let AstNode::App(a) = node {
            app_name = a.name.clone();
            port = a.port;
            db_path = a.database.as_ref().and_then(|d| d.path.clone());
        }
    }

    let target = args.get(2).map(|s| s.as_str()).unwrap_or("");

    match target {
        "--fly" => {
            write_file("Dockerfile", &deploy::generate_dockerfile(&app_name, port));
            write_file(".dockerignore", deploy::generate_dockerignore());
            write_file("fly.toml", &deploy::generate_fly_toml(&app_name, port));
            print_image_notes(db_path.as_deref());
            println!("\n  \x1b[1mDeploy to Fly.io:\x1b[0m");
            println!("  \x1b[32m1.\x1b[0m fly auth login");
            println!("  \x1b[32m2.\x1b[0m fly launch --copy-config --no-deploy");
            println!("  \x1b[32m3.\x1b[0m fly volumes create cronus_data --size 1");
            println!(
                "  \x1b[32m4.\x1b[0m fly secrets set JWT_SECRET=\"$(openssl rand -base64 48)\""
            );
            println!("  \x1b[32m5.\x1b[0m fly deploy");
        }
        "--railway" => {
            write_file("Dockerfile", &deploy::generate_dockerfile(&app_name, port));
            write_file(".dockerignore", deploy::generate_dockerignore());
            write_file(
                "railway.json",
                &deploy::generate_railway_config(&app_name, port),
            );
            print_image_notes(db_path.as_deref());
            println!("\n  \x1b[1mDeploy to Railway:\x1b[0m");
            println!("  \x1b[32m1.\x1b[0m railway login");
            println!(
                "  \x1b[32m2.\x1b[0m attach a volume mounted at {} and set JWT_SECRET",
                deploy::DATA_DIR
            );
            println!("  \x1b[32m3.\x1b[0m railway up");
        }
        "--static" => {
            println!("  \x1b[36m⚡\x1b[0m Static export requires running server first.");
            println!("  \x1b[90mStart with:\x1b[0m cronus run {}", port);
            println!(
                "  \x1b[90mThen use:\x1b[0m  wget -r -np http://localhost:{}/",
                port
            );
            println!("  \x1b[90mOr:\x1b[0m       curl http://localhost:{}/showcase -o dist/showcase.html", port);
        }
        _ => {
            write_file("Dockerfile", &deploy::generate_dockerfile(&app_name, port));
            write_file(
                "docker-compose.yml",
                &deploy::generate_compose(&app_name, port),
            );
            write_file(".dockerignore", deploy::generate_dockerignore());
            print_image_notes(db_path.as_deref());
            println!("\n  \x1b[1mReady for deployment!\x1b[0m\n");
            println!("  \x1b[32mDocker:\x1b[0m      docker compose up --build");
            println!("  \x1b[32mFly.io:\x1b[0m      cronus deploy --fly");
            println!("  \x1b[32mRailway:\x1b[0m     cronus deploy --railway");
            println!("  \x1b[32mStatic:\x1b[0m      cronus deploy --static");
        }
    }
    println!();
}
