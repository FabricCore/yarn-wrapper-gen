use std::{collections::HashMap, env};

use yarn_wrapper_gen::{FileTree, Index};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();

    if args.len() < 3 || (args.len() - 3) % 2 != 0 {
        println!(
            "Expect 3 + 2n arguments: [source] [output] [package] ([frompackage] [topackage])*"
        );
        return;
    }

    let remap = HashMap::from_iter(
        args.iter()
            .skip(3)
            .step_by(2)
            .map(String::clone)
            .zip(args.iter().skip(4).step_by(2).map(String::clone)),
    );

    let ftree = FileTree::new(&args[0]);
    let index = Index::new(&ftree);
    index.write(&args[1], &index, &args[2], &remap);
}
