const QUIT: string = "q";

test("test-script --help") {
    const VEC: [string] = ["a", "b", "c"];
    for s: string in VEC {
        println(s);
    }
    if "a" in VEC {
        println("a is in vec");
    }
    if !("d" in VEC) {
        println("d is not in vec");
    }
}
