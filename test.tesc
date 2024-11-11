test1("./input_test") {
    let i: int = 55;
    let j: int = i + i;

    println(i as string + ": " + j as string);
    input(j as string);
    output(110 as string);

    input("q");
    output("q");
}
