test_for("./app") {
	input("start");
	for i in /\d{3}/  { // Loop over all 3 digit numbers
		input(i);
		output(i);
	}
	input("exit");
}

test_for_2("./app") {
	input("start");
	for i in /\w{3}/  { // Loop over all 3 letter words
		input(i);
		output(i);
	}
	input("exit");
}
