test1("./input_test") {
    if true {
        println("This is true");
    } else {
        println("This is false");
    }

    if true println("This is true"); else println("This is false");

    const quitThisBitch: string = "q";
    input(quitThisBitch);
    output(quitThisBitch);
}
