test1("./input_test") {
    if true {
        println("Hello, World");
    }

    let _str: string = if 1 != 1 {
        "string";
    } else {
        1;
    }

    const quit: string = "q";
    input(quit);
    output(quit);
}
