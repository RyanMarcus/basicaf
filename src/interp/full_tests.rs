// < begin copyright >
// Copyright Ryan Marcus 2017
//
// This file is part of basicaf.
//
// basicaf is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// basicaf is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with basicaf.  If not, see <http://www.gnu.org/licenses/>.
//
// < end copyright >

#[cfg(test)]
mod test {
    use compile;
    use interp::BFEnv;

    #[test]
    fn simple_print_test() {
        let program = r#"
10 PRINT "hello world"
20 END
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);

        assert_eq!(result, "hello world");
    }

    #[test]
    fn simple_expr_test() {
        let program = r#"
10 LET X = ((6 * 8) / 2) + 1
20 PRINT X
30 END
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, "25");
    }

    #[test]
    fn simple_loop_test() {
        let program = r#"
10 LET Y = 5
15 FOR X = 0 TO 10
20 LET Y = Y * 2
25 NEXT X
30 PRINT Y
35 END
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, "5120");
    }

    #[test]
    fn simple_subroutine_test() {
        let program = r#"
20 LET X = 5
30 GOSUB 100
40 PRINT "X is now: ", X, "\n"
50 GOTO 103
100 LET X = 6
101 PRINT "\n\nLook ma, a subroutine!\n\n"
102 RETURN
103 END
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, "\n\nLook ma, a subroutine!\n\nX is now: 6\n");
    }

    #[test]
    fn nested_loop_test() {
        let program = r#"
10 FOR X = 5 TO 8
15 FOR Y = 3 TO 7
20 PRINT X, " times ", Y, " is ", X * Y, "\n"
25 NEXT Y
27 PRINT "\n"
30 NEXT X
40 END
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, "5 times 3 is 15\n5 times 4 is 20\n5 times 5 is 25\n5 times 6 is 30\n\n6 times 3 is 18\n6 times 4 is 24\n6 times 5 is 30\n6 times 6 is 36\n\n7 times 3 is 21\n7 times 4 is 28\n7 times 5 is 35\n7 times 6 is 42\n\n");
    }

    #[test]
    fn goto_test() {
        let program = r#"
5  GOTO 10
6  PRINT "2"
7  GOTO 40
10 LET X = 500
20 IF X < 1000 THEN 30
25 PRINT "does not print"
30 PRINT "1"
35 GOTO 6
40 END
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, "12");
    }

    #[test]
    fn collatz_test() {
        let program = r#"

10 REM compute the hailstone sequence of the number
15 FOR C = 1 TO 20
16 LET X = C
17 PRINT "\n", X, ": "
20 LET MODRESULT = 0
21 IF X = 1 THEN 70
25 GOSUB 1000
30 IF MODRESULT = 0 THEN 40
35 IF MODRESULT = 1 THEN 50
40 LET X = X / 2
45 GOTO 55
50 LET X = (3*X) + 1
55 PRINT X, " "
60 GOTO 20
70 NEXT C
80 GOTO 2000


1000 REM start of modulo subroutine.
1001 REM we should compute the modulo of X by 2
1002 REM and store the result into MODRESULT
1005 LET tmp = X
1010 IF tmp = 0 THEN 1202
1015 IF tmp = 1 THEN 1202
1020 LET tmp = tmp - 2
1025 GOTO 1010
1202 LET MODRESULT = tmp
1203 RETURN

2000 PRINT "\n"
2001 END
"#;

        let out = r#"
1: 
2: 1 
3: 10 5 16 8 4 2 1 
4: 2 1 
5: 16 8 4 2 1 
6: 3 10 5 16 8 4 2 1 
7: 22 11 34 17 52 26 13 40 20 10 5 16 8 4 2 1 
8: 4 2 1 
9: 28 14 7 22 11 34 17 52 26 13 40 20 10 5 16 8 4 2 1 
10: 5 16 8 4 2 1 
11: 34 17 52 26 13 40 20 10 5 16 8 4 2 1 
12: 6 3 10 5 16 8 4 2 1 
13: 40 20 10 5 16 8 4 2 1 
14: 7 22 11 34 17 52 26 13 40 20 10 5 16 8 4 2 1 
15: 46 23 70 35 106 53 160 80 40 20 10 5 16 8 4 2 1 
16: 8 4 2 1 
17: 52 26 13 40 20 10 5 16 8 4 2 1 
18: 9 28 14 7 22 11 34 17 52 26 13 40 20 10 5 16 8 4 2 1 
19: 58 29 88 44 22 11 34 17 52 26 13 40 20 10 5 16 8 4 2 1 
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, out);
    }

    #[test]
    fn simple_array_test() {
        let program = r#"
10  DIM X(5)
20  LET X(0) = 7
30  LET X(1) = 9
40  LET X(2) = 11
60  LET X(3) = 13
70  LET X(4) = 15
80  FOR I = 0 TO 5
90  PRINT X(I), "\n"
100 NEXT I
110 END
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, "7\n9\n11\n13\n15\n");
    }

    #[test]
    fn simple_read_test() {
        let program = r#"
10 DATA 1, 2, 3
20 READ X, Y, Z
30 PRINT X, " ", Y, " ", Z
40 END
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, "1 2 3");
    }

    #[test]
    fn array_read_test() {
        let program = r#"
5  DIM X(5)
10 DATA 1, 2, 3
20 READ X(1), X(3), X(4)
30 PRINT X(1), " ", X(3), " ", X(4)
40 END
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, "1 2 3");
    }

    #[test]
    fn twod_array_test() {
        let program = r#"
5  DIM X(5, 5)
10 LET X(0, 0) = 7
11 LET X(2, 3) = 5
12 LET X(1, 4) = 3
13 PRINT X(1, 4), " ", X(2, 3), " ", X(0, 0)
14 END
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, "3 5 7");
    }

    #[test]
    fn multi_call_test() {
        let program = r#"
10 GOSUB 100
20 PRINT "Again!"
30 GOSUB 100
40 PRINT "Done."
50 END

100 PRINT " in sub "
110 RETURN
"#;

        let bf = compile::compile(String::from(program), false, false, true);
        let mut interp = BFEnv::new();
        let result = interp.execute(bf);
        assert_eq!(result, " in sub Again! in sub Done.");
    }

}
