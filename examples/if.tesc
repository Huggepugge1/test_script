test("myapp") {
    println("string == string");
    if "string" == "string" {
	println("true");
    } else {
	println("false");
    }

    println("string != string");
    if "string" != "string" {
	println("true");
    } else {
	println("false");
    }

    println("1 < 2");
    if 1 < 2 {
	println("true");
    } else {
	println("false");
    }
    if 2 < 1 {
	println("true");
    } else {
	println("false");
    }

    println("1 <= 2");
    if 1 <= 2 {
	println("true");
    } else {
	println("false");
    }
    if 2 <= 1 {
	println("true");
    } else {
	println("false");
    }

    println("2 <= 2");
    if 2 <= 2 {
	println("true");
    } else {
	println("false");
    }
    
    println("1 > 2");
    if 1 > 2 {
	println("true");
    } else {
	println("false");
    }
    if 2 > 1 {
	println("true");
    } else {
	println("false");
    }

    println("1 >= 2");
    if 1 >= 2 {
	println("true");
    } else {
	println("false");
    }
    if 2 >= 1 {
	println("true");
    } else {
	println("false");
    }

    println("2 >= 2");
    if 2 >= 2 {
	println("true");
    } else {
	println("false");
    }

    println("true && true");
    if true && true {
	println("true");
    } else {
	println("false");
    }
    println("true && false");
    if true && false {
	println("true");
    } else {
	println("false");
    }
    println("false && true");
    if false && true {
	println("true");
    } else {
	println("false");
    }
    println("false && false");
    if false && false {
	println("true");
    } else {
	println("false");
    }
    
    println("true || true");
    if true || true {
	println("true");
    } else {
	println("false");
    }
    println("true || false");
    if true || false {
	println("true");
    } else {
	println("false");
    }
    println("false || true");
    if false || true {
	println("true");
    } else {
	println("false");
    }
    println("false || false");
    if false || false {
	println("true");
    } else {
	println("false");
    }

    println("!true");
    if !true {
	println("true");
    } else {
	println("false");
    }
    
    input("quit");
}
