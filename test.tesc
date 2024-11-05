test1("input_test") {
    for reg in /[^\d]*/ {
        input(reg);
        output(reg);
    }
    input("a");
    output("a");
}

test2("input_test") {
    input("hello");
    output("hello");
    input("hello2");
    output("hello2");
}
