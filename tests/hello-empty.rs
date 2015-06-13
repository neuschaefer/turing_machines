extern crate turing_machines;

use turing_machines::{TMDesc, TM};

// This turing machine should write the string "HELLO.WORLD!", walk back to
// its start and terminate.
static HELLO_TM: &'static str = "
	H	E	L	O	.	W	R	D	!	B
q0	-	-	-	-	-	-	-	-	-	q1,H,R
q1	-	-	-	-	-	-	-	-	-	q2,E,R
q2	-	-	-	-	-	-	-	-	-	q3,L,R
q3	-	-	-	-	-	-	-	-	-	q4,L,R
q4	-	-	-	-	-	-	-	-	-	q5,O,R
q5	-	-	-	-	-	-	-	-	-	q6,.,R
q6	-	-	-	-	-	-	-	-	-	q7,W,R
q7	-	-	-	-	-	-	-	-	-	q8,O,R
q8	-	-	-	-	-	-	-	-	-	q9,R,R
q9	-	-	-	-	-	-	-	-	-	qA,L,R
qA	-	-	-	-	-	-	-	-	-	qB,D,R
qB	-	-	-	-	-	-	-	-	-	q←,!,L
q←	q←,H,L	q←,E,L	q←,L,L	q←,O,L	q←,.,L	q←,W,L	q←,R,L	q←,D,L	q←,!,L	STOPP,B,R
STOPP
";

#[test]
fn test_hello() {
    let desc = TMDesc::from_string(HELLO_TM);
    println!("{:?}", desc);

    let mut tm = TM::new(&desc, "");

    for _ in 0..40 {
        tm.run_step();
    }

    assert!(tm.has_finished());
    assert_eq!(tm.get_tape_output(), "HELLO.WORLD!");
}
