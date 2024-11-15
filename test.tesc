test1("./input_test") {
<<<<<<< HEAD
    let i: int = 1 + 2 * 3 + 4;
    let j: string = i as string;
    println(j);

    let j: int = "1" as int;
    let k: string = j as string;

    input("q");
    output("q");
=======
    let quit: int = "q";
    quit = "send";

    input(quit);
    output(quit);
>>>>>>> refs/remotes/origin/master
}

test2("./input_test") {
    const i: int = 1;
    println(i as string);
    input("q");
    output("q");
}