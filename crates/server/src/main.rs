mod db;
mod handler;

pub use db::Database;
pub use handler::handle_client;

fn main() {
    println!("Hello, world!");
}
