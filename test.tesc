test1("./input_test") {
    let i: int = 1 + 2 * 3 + 4;
    let j: string = i as string;
    println(j);

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

test2("./input_test") {
    const i: int = 1;
    println(i as string);
    input("q");
    output("q");
}
