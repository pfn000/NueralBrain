//! Headless server example (placeholder — would use a web framework in production)

fn main() {
    println!("Headless server mode");
    println!("In production, this would use actix-web or axum to expose:");
    println!("  POST /input   — send text input");
    println!("  GET  /status  — get brain state");
    println!("  POST /reward  — reward the brain");
    println!("  POST /sleep   — enter sleep mode");
    println!("  POST /wake    — wake up");
    println!();
    println!("The brain would run continuously in a background thread,");
    println!("dreaming when idle, always learning.");
}
