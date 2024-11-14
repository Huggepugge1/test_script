test("./input_test") {
    const quit: string = "q";
    input(quit);
    output(quit);
}

test2("./input_test") {
    const quit: string = "q";
    quit = "quit";
    input(quit);
    output(quit);
}
