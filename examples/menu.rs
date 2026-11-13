/*══════════════════════════════════════════════════════════════════════════════
 FILE: examples/menu.rs - RUSTMT4 EXAMPLES LAUNCHER
 PURPOSE:
   Interactive launcher: pick a demo by number and it runs it via
   `cargo run --example <name>`. Enter 0 / q to quit.

 🎯 WHO SHOULD USE THIS:
   • First-time users browsing the examples without memorising names
   • A single entry point that dispatches to every other example

 📚 WHAT THIS DEMO COVERS:
   A numbered catalog that dispatches to each example:
      • quickstart           - connect, read account, (optional) trade
      • lowlevel_general     - low-level MT4Account read operations
      • lowlevel_trading     - low-level order send / modify / close
      • lowlevel_streaming   - low-level tick / event streams
      • service_demo         - typed MT4Service facade
      • sugar_basics         - account, prices, math, risk
      • sugar_trading        - risk-sized order with SL/TP
      • sugar_positions      - positions & exposure
      • strategies_demo      - orchestrators demo
      • protobuf_inspector   - explore proto types (offline)

 ℹ️  SAFE: launcher - it only spawns other examples; by itself it opens no
          connection and sends no orders.

 USAGE:
   cargo run --example menu
   (connection settings come from examples/0_common/settings.json and/or the
    MT4_LOGIN / MT4_PASSWORD / MT4_SERVER / MRPC_API_KEY env vars)
══════════════════════════════════════════════════════════════════════════════*/

use std::io::{self, Write};
use std::process::Command;

// region: EXAMPLE CATALOG
// ══════════════════════════════════════════════════════════════
// EXAMPLE CATALOG
// ══════════════════════════════════════════════════════════════

// ──────────────────────────────────────────────────────────────
// ITEMS
//   (menu label, cargo example name) for every runnable example,
//   in menu order. The index + 1 is the number shown to the user.
// ──────────────────────────────────────────────────────────────
const ITEMS: &[(&str, &str)] = &[
    ("Quickstart — connect, read account, (optional) trade", "quickstart"),
    ("Low-level · general operations (MT4Account)", "lowlevel_general"),
    ("Low-level · trading (MT4Account)", "lowlevel_trading"),
    ("Low-level · streaming (MT4Account)", "lowlevel_streaming"),
    ("Service · typed facade demo (MT4Service)", "service_demo"),
    ("Sugar · basics — account, prices, math, risk", "sugar_basics"),
    ("Sugar · trading — risk-sized order with SL/TP", "sugar_trading"),
    ("Sugar · positions & exposure", "sugar_positions"),
    ("Strategies · orchestrators demo", "strategies_demo"),
    ("Tools · Protobuf Inspector (explore proto types — offline)", "protobuf_inspector"),
];
// endregion: EXAMPLE CATALOG

// region: MENU LOOP
// ══════════════════════════════════════════════════════════════
// MENU LOOP
// ══════════════════════════════════════════════════════════════

// ──────────────────────────────────────────────────────────────
// main()
//   Print the catalog, read a choice, and shell out to the chosen
//   example via `cargo run --example <name>`; loop until 0 / q / EOF.
// ──────────────────────────────────────────────────────────────
fn main() {
    loop {
        println!("\n╔══════════════════════════════════════════════╗");
        println!("║             RustMT4 — examples menu          ║");
        println!("╚══════════════════════════════════════════════╝");
        for (i, (label, _)) in ITEMS.iter().enumerate() {
            println!("  {:>2}. {label}", i + 1);
        }
        println!("   0. quit");
        print!("\nselect [0-{}]: ", ITEMS.len());
        let _ = io::stdout().flush();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let choice = line.trim();
        if choice.is_empty() || choice == "0" || choice.eq_ignore_ascii_case("q") {
            println!("bye");
            break;
        }
        let n: usize = match choice.parse() {
            Ok(n) if (1..=ITEMS.len()).contains(&n) => n,
            _ => {
                println!("→ enter a number between 0 and {}", ITEMS.len());
                continue;
            }
        };
        let name = ITEMS[n - 1].1;
        println!("\n▶ cargo run --example {name}\n");
        match Command::new("cargo").args(["run", "--quiet", "--example", name]).status() {
            Ok(s) if s.success() => println!("\n✓ {name} finished"),
            Ok(s) => println!("\n✗ {name} exited with {s}"),
            Err(e) => println!("\n✗ failed to launch cargo: {e}"),
        }
    }
}
// endregion: MENU LOOP
