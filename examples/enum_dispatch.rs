use enum_dispatch::enum_dispatch;

#[enum_dispatch]
trait DoSomething {
    fn do_something(&self);
}

#[enum_dispatch(DoSomething)]
enum Types {
    Apple(A),
    Banana(B),
}

struct A;
struct B;

impl DoSomething for A {
    fn do_something(&self) {
        println!("A");
    }
}

impl DoSomething for B {
    fn do_something(&self) {
        println!("B");
    }
}
fn main() {
    // test enum_dispatch
    let apple = Types::Apple(A);
    let banana = Types::Banana(B);

    let type_apple = apple;
    let type_banana = banana;

    // 都是 types 类型的, 但是结果不同, enum_dispatch 相当于是主动帮我 match 了
    type_apple.do_something();
    type_banana.do_something();
}
