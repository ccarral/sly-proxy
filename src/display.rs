use crate::config::AppConfig;

pub fn display_app(config: &AppConfig) {
    println!("    App : {}\n", config.name);
    print!("    listening on : [");
    for port in config.ports() {
        print!("  {}  ", port);
    }
    print!("]\n\n");
    println!("    forwarding to :");
    for t in &config.target {
        println!("              {}", t.sock_addr());
    }
}
