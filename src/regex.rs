pub mod automaton;
mod transform;


#[cfg(test)]
mod tests{
    use super::{*, automaton::DFA};

    #[test]
    fn regex_concat(){
        let dfa=DFA::construct_dfa(&"abc".to_string()).unwrap();   
        assert!(dfa.match_input("abc"));
    }

    #[test]
    fn regex_or1(){
        let dfa=DFA::construct_dfa(&"a|bc".to_string()).unwrap();   
        assert!(dfa.match_input("bc"));
        assert!(dfa.match_input("ac"));
    }

    #[test]
    fn regex_or2(){
        let dfa=DFA::construct_dfa(&"(ab)|c".to_string()).unwrap();   
        assert!(dfa.match_input("ab"));
        assert!(dfa.match_input("c"));
    }

    #[test]
    fn regex_or3(){
        let dfa=DFA::construct_dfa(&"(ab)|(cd)".to_string()).unwrap();   
        assert!(dfa.match_input("ab"));
        assert!(dfa.match_input("cd"));
    }


    #[test]
    fn regex_asterisk1(){
        let dfa=DFA::construct_dfa(&"a(bc)*|d".to_string()).unwrap();   
        assert!(dfa.match_input("abcbcbc"));
        assert!(dfa.match_input("ad"));
    }
    
    #[test]
    fn regex_asterisk2(){
        let dfa=DFA::construct_dfa(&"0a(bc)*|d(ef)d*".to_string()).unwrap();   
        assert!(dfa.match_input("0adefdddd"));
        assert!(dfa.match_input("0abcbcbcef"));
    }
        
    #[test]
    fn regex_asterisk3(){
        let dfa=DFA::construct_dfa(&"(0a(bc)*|d(ef)d*)*(abc)|(123)".to_string()).unwrap();   
        assert!(dfa.match_input("0adefdddd0adefddddabc"));
        assert!(dfa.match_input("0abcbcbcef0abcbcbcef123"));
    }
        
    #[test]
    fn regex_incomplete1(){
        let dfa=DFA::construct_dfa(&"(0a(bc)*|d(ef)d*)*(abc)|(123)".to_string()).unwrap();   
        assert!(dfa.match_input("0adefdddd0adefddddabcaaa"));
        assert!(dfa.match_input("bbb0abcbcbcef0abcbcbcef123"));
    }
}