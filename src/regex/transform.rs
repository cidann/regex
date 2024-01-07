
#[derive(Debug,PartialEq)]
pub enum ThompsonOp  {
    LeftParen,
    RightParen,
    Asterisk,
    Or,
    Concat,
    Primary(char)
}



fn to_thompson_op(c:&char)->ThompsonOp{
    match c {
        '('=> ThompsonOp::LeftParen,
        ')'=> ThompsonOp::RightParen,
        '*'=> ThompsonOp::Asterisk,
        '|'=> ThompsonOp::Or,
        _=> ThompsonOp::Primary(*c)
    }
}

pub fn to_thompson_postfix(s:&str)->Result<Vec<ThompsonOp>,&str>{
    Ok(to_postfix_concat(s)?.0)
}

fn to_postfix_concat(s:&str)->Result<(Vec<ThompsonOp>,&str),&str>{
    let (mut stack, mut remain)=to_postfix_or(s)?;
    
    while let Some(next_char)=remain.chars().next(){
        match to_thompson_op(&next_char) {
            ThompsonOp::Primary(_)|ThompsonOp::LeftParen=>{
                let (right_concat,right_remain)=to_postfix_or(remain)?;
                stack.extend(right_concat);
                stack.push(ThompsonOp::Concat);
                remain=right_remain;
            },
            _=>{
                break;
            }
        }
    }

    Ok((stack,remain))
}

fn to_postfix_or(s:&str)->Result<(Vec<ThompsonOp>,&str),&str>{
    let (mut stack, mut remain)=to_postfix_asterisk(s)?;
    
    if let Some(next_char)=remain.chars().next(){
        if let ThompsonOp::Or= to_thompson_op(&next_char) {
            let (right_concat,right_remain)=to_postfix_or(&remain[1..])?;
            stack.extend(right_concat);
            stack.push(ThompsonOp::Or);
            remain=right_remain;
        }
    }
    
    Ok((stack,remain))
}

fn to_postfix_asterisk(s:&str)->Result<(Vec<ThompsonOp>,&str),&str>{
    let (mut stack, mut remain)=to_postfix_primary(s)?;
    
    if let Some(next_char)=remain.chars().next(){
        if let ThompsonOp::Asterisk= to_thompson_op(&next_char) {
            stack.push(ThompsonOp::Asterisk);
            remain=&remain[1..];
        }
    }
    
    Ok((stack,remain))
}

fn to_postfix_primary(s:&str)->Result<(Vec<ThompsonOp>,&str),&str>{
    let mut stack:Vec<ThompsonOp>=vec![];
    let mut remain=s;

    if let Some(next_char)=remain.chars().next(){
        match to_thompson_op(&next_char) {
            ThompsonOp::LeftParen=>{
                (stack,remain)=to_postfix_concat(&remain[1..])?;
                remain=&remain[1..];
            }
            ThompsonOp::Primary(_)=>{
                stack.push(ThompsonOp::Primary(next_char));
                remain=&remain[1..];
            }
            _=>{
                panic!("Unexpected operation at primary parse");
            }
        }  
    }
    else{
        return Err("Expected primary");
    }
    
    Ok((stack,remain))
}


#[cfg(test)]
mod thompson_postfix_tests{
    use super::*;

    #[test]
    fn concat_test()->Result<(),&'static str>{
        let input="abcdefg";
        let result=to_thompson_postfix(input)?;
        let mut expect=vec![ThompsonOp::Primary('a')];
        for c in input.chars().skip(1){
            expect.push(ThompsonOp::Primary(c));
            expect.push(ThompsonOp::Concat);
        }
        
        assert_eq!(result,expect);

        Ok(())
    }

    
    #[test]
    fn or_test_1()->Result<(),&'static str>{
        let input="ab|c";
        let result=to_thompson_postfix(input)?;
        let expect=vec![
            ThompsonOp::Primary('a'),
            ThompsonOp::Primary('b'),
            ThompsonOp::Primary('c'),
            ThompsonOp::Or,
            ThompsonOp::Concat
        ];

        assert_eq!(result,expect);

        Ok(())
    }

    #[test]
    fn or_test_2()->Result<(),&'static str>{
        let input="a|bc";
        let result=to_thompson_postfix(input)?;
        let expect=vec![
            ThompsonOp::Primary('a'),
            ThompsonOp::Primary('b'),
            ThompsonOp::Or,
            ThompsonOp::Primary('c'),
            ThompsonOp::Concat
        ];

        assert_eq!(result,expect);

        Ok(())
    }

    #[test]
    fn asterisk_test_1()->Result<(),&'static str>{
        let input="ab*|c";
        let result=to_thompson_postfix(input)?;
        let expect=vec![
            ThompsonOp::Primary('a'),
            ThompsonOp::Primary('b'),
            ThompsonOp::Asterisk,
            ThompsonOp::Primary('c'),
            ThompsonOp::Or,
            ThompsonOp::Concat
        ];

        assert_eq!(result,expect);

        Ok(())
    }

    #[test]
    fn asterisk_test_2()->Result<(),&'static str>{
        let input="a|b*c";
        let result=to_thompson_postfix(input)?;
        let expect=vec![
            ThompsonOp::Primary('a'),
            ThompsonOp::Primary('b'),
            ThompsonOp::Asterisk,
            ThompsonOp::Or,
            ThompsonOp::Primary('c'),
            ThompsonOp::Concat
        ];

        assert_eq!(result,expect);

        Ok(())
    }

    #[test]
    fn paren_test_1()->Result<(),&'static str>{
        let input="a|(b*c)";
        let result=to_thompson_postfix(input)?;
        let expect=vec![
            ThompsonOp::Primary('a'),
            ThompsonOp::Primary('b'),
            ThompsonOp::Asterisk,
            ThompsonOp::Primary('c'),
            ThompsonOp::Concat,
            ThompsonOp::Or,
        ];

        assert_eq!(result,expect);

        Ok(())
    }

    #[test]
    fn paren_test_2()->Result<(),&'static str>{
        let input="(ab)*|c";
        let result=to_thompson_postfix(input)?;
        let expect=vec![
            ThompsonOp::Primary('a'),
            ThompsonOp::Primary('b'),
            ThompsonOp::Concat,
            ThompsonOp::Asterisk,
            ThompsonOp::Primary('c'),
            ThompsonOp::Or,
        ];

        assert_eq!(result,expect);

        Ok(())
    }

    #[test]
    fn paren_test_3()->Result<(),&'static str>{
        let input="12(ab)*|c.*";
        let result=to_thompson_postfix(input)?;
        let expect=vec![
            ThompsonOp::Primary('1'),
            ThompsonOp::Primary('2'),
            ThompsonOp::Concat,
            ThompsonOp::Primary('a'),
            ThompsonOp::Primary('b'),
            ThompsonOp::Concat,
            ThompsonOp::Asterisk,
            ThompsonOp::Primary('c'),
            ThompsonOp::Or,
            ThompsonOp::Concat,
            ThompsonOp::Primary('.'),
            ThompsonOp::Asterisk,
            ThompsonOp::Concat,
        ];

        assert_eq!(result,expect);

        Ok(())
    }

    #[test]
    fn paren_test_4()->Result<(),&'static str>{
        let input="12(ab)*|(c.)*";
        let result=to_thompson_postfix(input)?;
        let expect=vec![
            ThompsonOp::Primary('1'),
            ThompsonOp::Primary('2'),
            ThompsonOp::Concat,
            ThompsonOp::Primary('a'),
            ThompsonOp::Primary('b'),
            ThompsonOp::Concat,
            ThompsonOp::Asterisk,
            ThompsonOp::Primary('c'),
            ThompsonOp::Primary('.'),
            ThompsonOp::Concat,
            ThompsonOp::Asterisk,
            ThompsonOp::Or,
            ThompsonOp::Concat,
        ];

        assert_eq!(result,expect);

        Ok(())
    }


}