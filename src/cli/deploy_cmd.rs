use crate::parser::AstNode;
use crate::{deploy, find_cronus_file, parser};
use std::fs;

pub fn cmd_deploy(args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found");
        std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
        std::process::exit(1);
    });
    let mut app_name = "cronus-app".to_string();
    let mut port: u16 = 5175;
    for node in &nodes {
        if let AstNode::App(a) = node {
            app_name = a.name.clone();
            port = a.port;
        }
    }

    let target = args.get(2).map(|s| s.as_str()).unwrap_or("");

    match target {
        "--fly" => {
            fs::write("Dockerfile", deploy::generate_dockerfile(&app_name, port)).unwrap();
            fs::write(".dockerignore", deploy::generate_dockerignore()).unwrap();
            fs::write("fly.toml", deploy::generate_fly_toml(&app_name, port)).unwrap();
            println!("  \x1b[32m✓\x1b[0m Generated Dockerfile + fly.toml");
            println!("\n  \x1b[1mDeploy to Fly.io:\x1b[0m");
            println!("  \x1b[32m1.\x1b[0m fly auth login");
            println!("  \x1b[32m2.\x1b[0m fly launch --copy-config --yes");
            println!("  \x1b[32m3.\x1b[0m fly deploy");
        }
        "--railway" => {
            fs::write("Dockerfile", deploy::generate_dockerfile(&app_name, port)).unwrap();
            fs::write(".dockerignore", deploy::generate_dockerignore()).unwrap();
            fs::write(
                "railway.json",
                deploy::generate_railway_config(&app_name, port),
            )
            .unwrap();
            println!("  \x1b[32m✓\x1b[0m Generated Dockerfile + railway.json");
            println!("\n  \x1b[1mDeploy to Railway:\x1b[0m");
            println!("  \x1b[32m1.\x1b[0m railway login");
            println!("  \x1b[32m2.\x1b[0m railway up");
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
            fs::write("Dockerfile", deploy::generate_dockerfile(&app_name, port)).unwrap();
            println!("  \x1b[32m✓\x1b[0m Generated Dockerfile");
            fs::write(
                "docker-compose.yml",
                deploy::generate_compose(&app_name, port),
            )
            .unwrap();
            println!("  \x1b[32m✓\x1b[0m Generated docker-compose.yml");
            fs::write(".dockerignore", deploy::generate_dockerignore()).unwrap();
            println!("  \x1b[32m✓\x1b[0m Generated .dockerignore");
            println!("\n  \x1b[1mReady for deployment!\x1b[0m\n");
            println!("  \x1b[32mDocker:\x1b[0m      docker compose up --build");
            println!("  \x1b[32mFly.io:\x1b[0m      cronus deploy --fly");
            println!("  \x1b[32mRailway:\x1b[0m     cronus deploy --railway");
            println!("  \x1b[32mStatic:\x1b[0m      cronus deploy --static");
        }
    }
    println!();
}
