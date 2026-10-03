
fn main() {
    println!("{}", hello(false));
}

fn hello(x:bool) -> &'static str {
	if x == true {
		return "Hello world";
	}

	"Hello world!!!"
}

