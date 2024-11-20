const INPUT: string = "input";

fn same(a: string) {
    input(a);
    output(a);
}

test1("./input_test") {
    same(INPUT);

    const QUIT_THIS_BITCH: string = "q";
    input(QUIT_THIS_BITCH);
    output(QUIT_THIS_BITCH);
}
