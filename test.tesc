const QUIT: string = "q";

test1("java Main.java") {
    let inp: string = "";
    for i: string in `\d` {
        println("Position: " + i);
        inp = inp + i;
        println(inp + "\n");
    }
    input(inp + QUIT);
    output(inp + QUIT);
}
