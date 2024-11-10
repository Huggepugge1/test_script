test1("./input_test") {
    for i in /[a-z]\d{2}/ {
        input(i + i);
        output(i + i);
    }
    input("q");
    output("q");
}
