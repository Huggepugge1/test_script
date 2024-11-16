test1("./input_test") {
    const REG: regex = `-?\d \+ \d`;
    for i: string in REG {
        input(i);
        output(i);
    }

    const QUIT_THIS_BITCH: string = "q";
    input(QUIT_THIS_BITCH);
    output(QUIT_THIS_BITCH);
}
