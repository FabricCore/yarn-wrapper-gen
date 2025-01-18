use std::env;

use yarn_wrapper_gen::{FileTree, Index};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();

    if args.len() != 3 {
        println!("Expect 3 arguments: [source] [output] [package]");
        return;
    }

    let ftree = FileTree::new(&args[0]);
    let index = Index::new(&ftree);
    index.write(&args[1], &index, &args[2]);
}
