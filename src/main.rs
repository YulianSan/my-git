use sangit;
fn main() {
    sangit::main()
        .map_err(|e| {
            eprintln!("Error: {}", e);
        })
        .unwrap();
}
