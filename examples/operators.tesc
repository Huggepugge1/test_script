addition("calculator") {
	const one: int = 1;
	const two: int = 2;

	const one_plus_two: string = one as string + "+" + two as string;

	input(one_plus_two);
	output((one + two) as string);
}

subtraction("calculator") {
	const one: int = 1;
	const two: int = 2;

	const one_minus_two: string = one as string + "-" + two as string;

	input(one_minus_two);
	output((one - two) as string);
}

multiplication("calculator") {
	const one: int = 1;
	const two: int = 2;

	const one_times_two: string = one as string + "*" + two as string;

	input(one_times_two);
	output((one * two) as string);
}

division("calculator") {
	const one: int = 1;
	const two: int = 2;

	const one_divided_two: string = one as string + "/" + two as string;

	input(one_times_two);
	output((one / two) as string);
}
