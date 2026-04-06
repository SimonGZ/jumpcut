use std::env;
use std::path::Path;
use std::process::ExitCode;

use jumpcut::diagnostics::{line_breaks, page_breaks};

struct DiagnosticCommand {
    name: &'static str,
    action: DiagnosticAction,
}

#[derive(Copy, Clone)]
enum DiagnosticAction {
    Direct(fn(&Path)),
}

const DIAGNOSTIC_COMMANDS: &[DiagnosticCommand] = &[
    DiagnosticCommand {
        name: "big-fish-review",
        action: DiagnosticAction::Direct(write_big_fish_review),
    },
    DiagnosticCommand {
        name: "big-fish-full-script",
        action: DiagnosticAction::Direct(write_big_fish_full_script_review),
    },
    DiagnosticCommand {
        name: "big-fish-linebreak",
        action: DiagnosticAction::Direct(write_big_fish_linebreak),
    },
    DiagnosticCommand {
        name: "little-women-review",
        action: DiagnosticAction::Direct(write_little_women_review),
    },
    DiagnosticCommand {
        name: "little-women-full-script",
        action: DiagnosticAction::Direct(write_little_women_full_script_review),
    },
    DiagnosticCommand {
        name: "little-women-linebreak",
        action: DiagnosticAction::Direct(write_little_women_linebreak),
    },
    DiagnosticCommand {
        name: "mostly-genius-linebreak",
        action: DiagnosticAction::Direct(write_mostly_genius_linebreak),
    },
    DiagnosticCommand {
        name: "mostly-genius-full-script",
        action: DiagnosticAction::Direct(write_mostly_genius_full_script_review),
    },
    DiagnosticCommand {
        name: "vikings-full-script",
        action: DiagnosticAction::Direct(write_vikings_full_script_review),
    },
    DiagnosticCommand {
        name: "gumshoe-full-script",
        action: DiagnosticAction::Direct(write_gumshoe_full_script_review),
    },
    DiagnosticCommand {
        name: "fd-probes",
        action: DiagnosticAction::Direct(write_fd_probes),
    },
    DiagnosticCommand {
        name: "big-fish-json",
        action: DiagnosticAction::Direct(write_big_fish_json),
    },
    DiagnosticCommand {
        name: "public-window-json",
        action: DiagnosticAction::Direct(write_public_window_json),
    },
    DiagnosticCommand {
        name: "visual-export",
        action: DiagnosticAction::Direct(write_visual_export),
    },
];

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "all".to_string());

    if command == "help" || command == "--help" || command == "-h" {
        print_help();
        return ExitCode::SUCCESS;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let commands: Vec<&DiagnosticCommand> = if command == "all" {
        DIAGNOSTIC_COMMANDS.iter().collect()
    } else {
        match DIAGNOSTIC_COMMANDS.iter().find(|item| item.name == command) {
            Some(item) => vec![item],
            None => {
                eprintln!("Unknown diagnostics command: {command}");
                print_help();
                return ExitCode::from(2);
            }
        }
    };

    for diagnostic in commands {
        match diagnostic.action {
            DiagnosticAction::Direct(writer) => writer(repo_root),
        }
    }

    ExitCode::SUCCESS
}

fn print_help() {
    eprintln!("Usage: cargo run --bin pagination-diagnostics -- <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  all");
    for diagnostic in DIAGNOSTIC_COMMANDS {
        eprintln!("  {}", diagnostic.name);
    }
}

fn write_big_fish_linebreak(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/big-fish-linebreak-parity");
    line_breaks::write_big_fish_packet(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
    println!("wrote {}", debug_dir.join("parity.json").display());
}

fn write_little_women_linebreak(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/little-women-linebreak-parity");
    line_breaks::write_little_women_packet(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
    println!("wrote {}", debug_dir.join("parity.json").display());
}

fn write_mostly_genius_linebreak(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/mostly-genius-linebreak-parity");
    line_breaks::write_mostly_genius_packet(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
    println!("wrote {}", debug_dir.join("parity.json").display());
}

fn write_big_fish_json(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug");
    page_breaks::write_big_fish_public_slice_json(&debug_dir);
    println!(
        "wrote {}",
        debug_dir
            .join("big-fish.p18-19.actual.page-breaks.json")
            .display()
    );
    println!(
        "wrote {}",
        debug_dir
            .join("big-fish.p18-19.comparison-report.json")
            .display()
    );
    println!(
        "wrote {}",
        debug_dir
            .join("big-fish.p18-19.page-endings.json")
            .display()
    );
}

fn write_public_window_json(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug");
    page_breaks::write_selected_public_windows_json(&debug_dir);
    println!(
        "wrote {}",
        debug_dir
            .join("brick-n-steel.p2-4.actual.page-breaks.json")
            .display()
    );
    println!(
        "wrote {}",
        debug_dir
            .join("brick-n-steel.p2-4.comparison-report.json")
            .display()
    );
    println!(
        "wrote {}",
        debug_dir
            .join("brick-n-steel.p2-4.pdf-line-counts.json")
            .display()
    );
    println!(
        "wrote {}",
        debug_dir
            .join("little-women.p4-6.actual.page-breaks.json")
            .display()
    );
    println!(
        "wrote {}",
        debug_dir
            .join("little-women.p4-6.comparison-report.json")
            .display()
    );
    println!(
        "wrote {}",
        debug_dir
            .join("little-women.p4-6.pdf-line-counts.json")
            .display()
    );
}

fn write_big_fish_review(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/big-fish-review");
    page_breaks::write_big_fish_review_packet(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
}

fn write_big_fish_full_script_review(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/big-fish-full-script");
    page_breaks::write_big_fish_full_script_page_break_packet(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
}

fn write_little_women_review(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/little-women-review");
    page_breaks::write_little_women_review_packet(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
}

fn write_little_women_full_script_review(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/little-women-full-script");
    page_breaks::write_little_women_full_script_page_break_packet(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
}

fn write_mostly_genius_full_script_review(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/mostly-genius-full-script");
    page_breaks::write_mostly_genius_full_script_page_break_packet(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
}

fn write_vikings_full_script_review(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/vikings-full-script");
    page_breaks::write_vikings_full_script_page_break_packet(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
}

fn write_gumshoe_full_script_review(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/gumshoe-full-script");
    page_breaks::write_gumshoe_full_script_page_break_packet(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
}

fn write_fd_probes(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/fd-probes");
    page_breaks::write_fd_probe_packets(&debug_dir);
    println!("wrote {}", debug_dir.join("REVIEW.md").display());
}

fn write_visual_export(repo_root: &Path) {
    let debug_dir = repo_root.join("target/pagination-debug/visual");
    page_breaks::write_visual_comparison_data(&debug_dir);
    println!(
        "wrote {}",
        debug_dir.join("big-fish.comparison.json").display()
    );
    println!(
        "wrote {}",
        debug_dir
            .join("little-women-p4-6.comparison.json")
            .display()
    );
}
