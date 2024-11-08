test1("valgrind ./input_test") {
	for i in /[a-z]\d{3}/ {
		input(i);
		output(i);
	}
	input("q");
	output("q");
}
