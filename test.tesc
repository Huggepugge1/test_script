const INPUT: string = "input";

fn fib(n: int): int {
    if n < 2 {
        n;
    } else {
        const RESULT: int = fib(n - 1) + fib(n - 2);
        RESULT;
    }
}

const COLON: string = ":";

test1("./input_test") {
    for i: string in `\d{2}` {
        print(i + COLON);
        println(fib(i as int) as string);
    }

    const QUIT_THIS_BITCH: string = "q";
    input(QUIT_THIS_BITCH);
    output(QUIT_THIS_BITCH);
}
