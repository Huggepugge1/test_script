test1("./input_test") {
    let digits: regex = /\d/;
    for i: string in digits {
        input(i);
        output(i);
        print(i + ": ");
        println(i + i);
    }
    input("q");
    output("q");
}
