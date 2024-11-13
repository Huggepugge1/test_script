test("./input_test") {
    let x: bool = true;
    let y: string =  if x "This should cause an error"; else "String";
    

    println(y);

    const quit: string = "q";
    input(quit);
    output(quit);
}
