const QUIT: string = "q";

test1("java Main.java") {
    let inp: string = "";
    for i: string in `\d` {
        inp = inp + i + "\n";
    }
    input(inp + QUIT);
    output(inp + QUIT);
}
