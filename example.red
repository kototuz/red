fn add(a, b) int {
    return a + b
}

fn fizzbuzz() {
    fizz := 12341234
    for i := 1; i <= 20; i = i+1 {
        fizz := i % 3 == 0
        buzz := i % 5 == 0
        if fizz && buzz {
            @cmd "say FizzBuzz"
        } else if fizz {
            @cmd "say Fizz"
        } else if buzz {
            @cmd "say Buzz"
        } else {
            @log "i"
        }
    }
}

extern setblock(int, int, int)

fn main() {
    for x := 31; x < 41; x = x+1 {
        for z := 55; z < 65; z = z+1 {
            setblock(x, 151, z)
        }
    }
}
