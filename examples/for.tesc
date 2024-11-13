test_for("./app") {
	input("start");
	for i: string in `\d{3}`  { // Loop over all 3 digit numbers
		input(i);
		output(i);
	}
	input("exit");
}

test_for_2("./app") {
	input("start");
	for i: string in `\w{3}`  { // Loop over all 3 letter words
		input(i);
		output(i);
	}
	input("exit");
}

test_for_3("./app") {
	input("start");
	let numbers: regex = `\d{3}`;
	for i: string in numbers  {
		input(i);
		output(i);
	}
	input("exit");
}
