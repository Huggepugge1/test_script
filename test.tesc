const QUIT: string = "q";
const BIG_NUMBER: int = 1000000000;

test1("ls") {
    let inp: int = BIG_NUMBER;

    println((1 >= 1) as string);

    for i: string in `[1-9]` {
        println("Position: " + i);
        inp = inp + i as int;
        println(inp as string + "\n");
    }
    input(inp as string + QUIT);
    output(inp as string + QUIT);
}
