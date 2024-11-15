test_print("echo test") {
	for i: string in `\d` {
		print(i);
	}
	println("");
	println("Test complete");
	output("test");
}
