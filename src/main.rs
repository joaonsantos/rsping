use rsping::scanner::Pinger;

fn print_usage(cmd: String) {
    eprint!("Usage\n  {} target\n", cmd)
}

fn main() {
    let mut args = std::env::args();
    let cmd = args.next().unwrap();
    if args.len() != 1 {
        print_usage(cmd);
        return;
    }

    let target = args.next().unwrap_or("".to_string());
    if target == "" || target == "--help" {
        print_usage(cmd);
        return;
    }

    let mut scanner = Pinger::new(target, String::from("0"));
    if let Err(e) = scanner.ping() {
        eprintln!("{cmd}: {}", e);
    }
}
