use chipper::ChipperUI;

fn main() {
    match ChipperUI::run() {
        Ok(_) => {}
        Err(e) => println!("Error: {}", e)
    };
}
