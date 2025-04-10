fn fib(n: int): int {
    if n < 2 {
        n;
    } else {
        fib(n - 1) + fib(n - 2);
    }
}

test("./my_app") {
    const N: int = 10;
    input(N as string);
    output(fib(N) as string);
}
