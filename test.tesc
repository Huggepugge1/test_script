const QUIT: string = "q";

test1("java Main.java") {
    println((1 as float + 2.5 * 3 as float - 7 as float * 4.7) as string);
    let inp: string = "";
    for i: string in `\d` {
        inp = inp + i + "\n";
    }

    input(inp + QUIT);
    output(inp + QUIT);
}
