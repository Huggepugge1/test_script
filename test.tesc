test1("./input_test") {
    for i: string in `\d{1,3}` {
        input(i);
        output(i);
    }

    const a: int = 1;
    let a: string = "1";
    a = a + "1";
    println(a);

    const b: int = 1;
    println(b as string);

    let s: string = "123";
    s = s + "456";
    println(s);

    input("q");
    output("q");
}
