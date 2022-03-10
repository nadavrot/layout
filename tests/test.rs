use layout::core::geometry::Point;

#[cfg(test)]
mod tests {

    use layout::core::geometry::weighted_median;
    use layout::gv::record::parse_record_string;
    use layout::gv::record::print_record;
    use layout::gv::DotParser;
    use layout::gv::Lexer;
    use layout::gv::Token;
    use layout::std_shapes::shapes::RecordDef;

    fn is_identifier(t: Token, target: &str) -> bool {
        match t {
            Token::Identifier(name) => target == name,
            _ => false,
        }
    }
    fn get_sample_program2() -> String {
        r##"/* ancestor graph from Caroline Bouvier Kennedy */
        graph G {
            I5 [shape=ellipse,color=red,style=bold,label="Caroline Bouvier Kennedy\nb. 27.11.1957 New York",image="images/165px-Caroline_Kennedy.jpg",labelloc=b];
            I1 [shape=box,color=blue,style=bold,label="John Fitzgerald Kennedy\nb. 29.5.1917 Brookline\nd. 22.11.1963 Dallas",image="images/kennedyface.jpg",labelloc=b];
            I6 [shape=box,color=blue,style=bold,label="John Fitzgerald Kennedy\nb. 25.11.1960 Washington\nd. 16.7.1999 over the Atlantic Ocean, near Aquinnah, MA, USA",image="images/180px-JFKJr2.jpg",labelloc=b];
            I7 [shape=box,color=blue,style=bold,label="Patrick Bouvier Kennedy\nb. 7.8.1963\nd. 9.8.1963"];
            I2 [shape=ellipse,color=red,style=bold,label="Jaqueline Lee Bouvier\nb. 28.7.1929 Southampton\nd. 19.5.1994 New York City",image="images/jacqueline-kennedy-onassis.jpg",labelloc=b];
            I8 [shape=box,color=blue,style=bold,label="Joseph Patrick Kennedy\nb. 6.9.1888 East Boston\nd. 16.11.1969 Hyannis Port",image="images/1025901671.jpg",labelloc=b];
            I10 [shape=box,color=blue,style=bold,label="Joseph Patrick Kennedy Jr\nb. 1915\nd. 1944"];
            I11 [shape=ellipse,color=red,style=bold,label="Rosemary Kennedy\nb. 13.9.1918\nd. 7.1.2005",image="images/rosemary.jpg",labelloc=b];
            I12 [shape=ellipse,color=red,style=bold,label="Kathleen Kennedy\nb. 1920\nd. 1948"];
            I13 [shape=ellipse,color=red,style=bold,label="Eunice Mary Kennedy\nb. 10.7.1921 Brookline"];
            I9 [shape=ellipse,color=red,style=bold,label="Rose Elizabeth Fitzgerald\nb. 22.7.1890 Boston\nd. 22.1.1995 Hyannis Port",image="images/Rose_kennedy.JPG",labelloc=b];
            I15 [shape=box,color=blue,style=bold,label="Aristotle Onassis"];
            I3 [shape=box,color=blue,style=bold,label="John Vernou Bouvier III\nb. 1891\nd. 1957",image="images/BE037819.jpg",labelloc=b];
            I4 [shape=ellipse,color=red,style=bold,label="Janet Norton Lee\nb. 2.10.1877\nd. 3.1.1968",image="images/n48862003257_1275276_1366.jpg",labelloc=b];
             I1 -- I5  [style=bold,color=blue]; 
             I1 -- I6  [style=bold,color=orange]; 
             I2 -- I6  [style=bold,color=orange]; 
             I1 -- I7  [style=bold,color=orange]; 
             I2 -- I7  [style=bold,color=orange]; 
             I1 -- I2  [style=bold,color=violet]; 
             I8 -- I1  [style=bold,color=blue]; 
             I8 -- I10  [style=bold,color=orange]; 
             I9 -- I10  [style=bold,color=orange]; 
             I8 -- I11  [style=bold,color=orange]; 
             I9 -- I11  [style=bold,color=orange]; 
             I8 -- I12  [style=bold,color=orange]; 
             I9 -- I12  [style=bold,color=orange]; 
             I8 -- I13  [style=bold,color=orange]; 
             I9 -- I13  [style=bold,color=orange]; 
             I8 -- I9  [style=bold,color=violet]; 
             I9 -- I1  [style=bold,color=red]; 
             I2 -- I5  [style=bold,color=red]; 
             I2 -- I15  [style=bold,color=violet]; 
             I3 -- I2  [style=bold,color=blue]; 
             I3 -- I4  [style=bold,color=violet]; 
             I4 -- I2  [style=bold,color=red]; 
            }
        "##
        .to_string()
    }

    #[test]
    fn simple() {
        let mut lexer = Lexer::from_string("a -> b");
        let t0 = lexer.next_token();
        let t1 = lexer.next_token();
        let t2 = lexer.next_token();
        println!("{:?}", t0);
        println!("{:?}", t1);
        println!("{:?}", t2);
        assert!(is_identifier(t0, "a"));
        assert!(matches!(t1, Token::ArrowRight));
        assert!(is_identifier(t2, "b"));
    }
    #[test]
    fn simple_number() {
        let mut lexer = Lexer::from_string("-12345");
        let t0 = lexer.next_token();
        let t1 = lexer.next_token();
        println!("{:?}", t0);
        println!("{:?}", t1);
        assert!(is_identifier(t0, "-12345"));
        assert!(matches!(t1, Token::EOF));
    }
    #[test]
    fn simple_float_number() {
        let mut lexer = Lexer::from_string("1.12");
        let t0 = lexer.next_token();
        let t1 = lexer.next_token();
        println!("{:?}", t0);
        println!("{:?}", t1);
        assert!(is_identifier(t0, "1.12"));
        assert!(matches!(t1, Token::EOF));
    }

    #[test]
    fn simple_program() {
        let mut lexer = Lexer::from_string("digraph { a -> b; } ");
        assert!(matches!(lexer.next_token(), Token::DigraphKW));
        assert!(matches!(lexer.next_token(), Token::OpenBrace));
        assert!(matches!(lexer.next_token(), Token::Identifier(_)));
        assert!(matches!(lexer.next_token(), Token::ArrowRight));
        assert!(matches!(lexer.next_token(), Token::Identifier(_)));
        assert!(matches!(lexer.next_token(), Token::Semicolon));
        assert!(matches!(lexer.next_token(), Token::CloseBrace));
        assert!(matches!(lexer.next_token(), Token::EOF));
    }

    #[test]
    fn lex_program() {
        let program = get_sample_program2();
        let mut lexer = Lexer::from_string(&program[..]);
        let mut tok = lexer.next_token();
        let mut counter = 1;
        while !matches!(tok, Token::EOF) {
            println!("{:?}", tok);
            if let Token::Error(_) = tok {
                lexer.print_error();
                panic!();
            }

            tok = lexer.next_token();
            counter += 1;
        }
        assert_eq!(counter, 629);
    }

    #[test]
    fn parse_program0() {
        let mut parser = DotParser::new("graph { a -> b; b -> c;}");
        if let Result::Err(err) = parser.process() {
            parser.print_error();
            println!("Error: {}", err);
            panic!();
        }
    }

    #[test]
    fn parse_program1() {
        let mut parser = DotParser::new("graph { a -> b -> c; }");
        if let Result::Err(err) = parser.process() {
            parser.print_error();
            println!("Error: {}", err);
            panic!();
        }
    }

    #[test]
    fn parse_program2() {
        let program = get_sample_program2();
        let mut parser = DotParser::new(&program[..]);
        if let Result::Err(err) = parser.process() {
            parser.print_error();
            println!("Error: {}", err);
            panic!();
        }
    }

    #[test]
    fn parse_program_fail() {
        let mut parser = DotParser::new("graph { } s");
        if parser.process().is_err() {
            return;
        }
        panic!();
    }

    #[test]
    fn parse_record0() {
        let desc = "hello&#92;nworld |{ b |{c|<here> d|e}| f}| g | h";
        let res = parse_record_string(desc);
        print_record(&res, 0);
    }
    #[test]
    fn parse_record1() {
        let desc = "{InputLayer\n|{input:|output:}|{{[(?, ?)]}|{[(?, ?)]}}}";
        let res = parse_record_string(desc);
        print_record(&res, 0);
    }

    #[test]
    fn parse_record2() {
        let desc = "department: Dense\n|{input:|output:}|{{(?, 172)}|{(?, 4)}}";
        let res = parse_record_string(desc);
        print_record(&res, 0);
    }

    #[test]
    fn parse_record_port0() {
        let desc = "<f0> foo";
        let res = parse_record_string(desc);
        print_record(&res, 0);
        if let RecordDef::Array(arr) = res {
            assert_eq!(arr.len(), 1, "expecting one element");
            if let RecordDef::Text(label, port) = &arr[0] {
                assert_eq!(label, "foo");
                if let Option::Some(port) = port {
                    assert_eq!(port, "f0");
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_median() {
        let k = weighted_median(&[1.]);
        assert_eq!(k, 1.);
        let k = weighted_median(&[2., 1.]);
        assert_eq!(k, 1.5);
        let k = weighted_median(&[99., 2., 1.]);
        assert_eq!(k, 2.);
        let k = weighted_median(&[90., 23., 0., 1., 3.]);
        assert_eq!(k, 3.);
        let k = weighted_median(&[0., 99., 30., 40.]);
        assert_eq!(k, 35.);
    }

    #[test]
    fn test_median_range() {
        for i in 2..10 {
            let data: Vec<f64> = (1..i).map(|x: usize| x as f64).collect();
            println!("{:?}", data);
            let _ = weighted_median(&data);
        }
    }
}

#[test]
fn test_rotate() {
    fn almost(a: f64, b: f64) {
        let abs_difference = (b - a).abs();
        assert!(abs_difference < 1e-10);
    }
    // 180'
    let p = Point::new(1.0, 0.0);
    let r = p.rotate(180_f64.to_radians());
    almost(r.x, -1.);
    almost(r.y, 0.);

    //90'
    let p = Point::new(1.0, 0.0);
    let r = p.rotate(90_f64.to_radians());
    almost(r.x, 0.);
    almost(r.y, 1.);

    //45'
    let p = Point::new(1.0, 0.0);
    let r = p.rotate(45_f64.to_radians());
    almost(r.x, 1. / 2_f64.sqrt());
    almost(r.y, 1. / 2_f64.sqrt());

    //Rotate around a point.
    let p = Point::new(101.0, 100.0);
    let c = Point::new(100., 100.);
    let r = p.rotate_around(c, 45_f64.to_radians());
    almost(r.x, 100. + 1. / 2_f64.sqrt());
    almost(r.y, 100. + 1. / 2_f64.sqrt());
}
