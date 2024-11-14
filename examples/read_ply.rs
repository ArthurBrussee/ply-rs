extern crate ply_rs;
use ply_rs as ply;

/// Demonstrates simplest use case for reading from a file.
fn main() {
    async_std::task::block_on(async {
        // set up a reader, in this a file.
        let path = "example_plys/greg_turk_example1_ok_ascii.ply";
        let mut f = async_std::fs::File::open(path).await.unwrap();

        // create a parser
        let p = ply::parser::Parser::<ply::ply::DefaultElement>::new();

        // use the parser: read the entire file
        let ply = p.read_ply(&mut f).await;

        // make sure it did work
        assert!(ply.is_ok());

        // proof that data has been read
        println!("Read ply data: {:#?}", ply.unwrap());
    });
}
