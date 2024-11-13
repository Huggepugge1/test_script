test1("./input_test") {
    let i: int = 1 + 2 * 3 + 4;
    let j: string = i as string;
    println(j);

    let j: int = "1" as int;
    let k: string = j as string;

    input("q");
    output("q");
}

test2("./input_test") {
    const i: int = 1;
    println(i as string);
    input("q");
    output("q");
    for i: string in `\d{3}` {
        
    }
}