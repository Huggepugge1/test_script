test1("./input_test") {
    const quit: string = "q";

    for str: string in `helo?` {
        println(str);
    }

    input(quit);
    output(quit);
}
