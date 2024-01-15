use crate::regex::transform::{ThompsonOp,to_thompson_postfix};
use super::Automaton;
use super::Symbol;


#[derive(Debug)]
pub struct NFA{
    pub automaton:Automaton
}

impl PartialEq for NFA {
    fn eq(&self, other: &Self) -> bool {
        self.automaton == other.automaton
    }
}

impl NFA {
    pub fn construct_nfa(re :&String)->Result<NFA,String>{
        let thompson=to_thompson_postfix(re)?;
        let mut stack:Vec<Automaton>=Vec::new();
        println!("Thompson construction stack: {:?}",thompson);
    
        for op in &thompson{
            match op {
                ThompsonOp::Primary(c)=>{
                    stack.push(Automaton::automaton_transition(&Symbol::new_alphabet(*c)));
                },
                ThompsonOp::Asterisk => {
                    let automaton=stack.pop().unwrap();
                    stack.push(Automaton::automaton_zero_or_one(&automaton));
                },
                ThompsonOp::Or => {
                    let automaton2=stack.pop().unwrap();
                    let automaton1=stack.pop().unwrap();
                    stack.push(Automaton::automaton_alternate(&automaton1, &automaton2));
                },
                ThompsonOp::Concat => {
                    let automaton2=stack.pop().unwrap();
                    let automaton1=stack.pop().unwrap();
                    stack.push(Automaton::automaton_concat(&automaton1, &automaton2));
                },
                ThompsonOp::All=>{
                    stack.push(Automaton::automaton_transition(&Symbol::new_all_char()));
                }
                _=>{panic!("Unhandled thomson op")}
            }
        }
    
        if stack.len()!=1{
            panic!("Expect there to be only 1 automaton for each regex");
        }
    
        Ok(NFA{automaton:stack.pop().unwrap()})
    }
}



#[cfg(test)]
mod tests{
    use crate::regex::automaton::state::{State,Symbol::Alphabet};
    use super::*;
    

    #[test]
    fn build_concat1() {
        let re="abc";
        let accept1=State::new_accept_ref();
        let accept2=State::new_accept_ref();
        let accept3=State::new_accept_ref();
        let a1=Automaton{start_state:State::new_transition_ref(Alphabet('a'), Some(accept1.clone())),end_state:accept1};
        let a2=Automaton{start_state:State::new_transition_ref(Alphabet('b'), Some(accept2.clone())),end_state:accept2};
        let a3=Automaton{start_state:State::new_transition_ref(Alphabet('c'), Some(accept3.clone())),end_state:accept3};
        let a=Automaton::concat_automaton(&a1, &a2);
        let expect=NFA{automaton:Automaton::concat_automaton(&a, &a3)};
        let result=NFA::construct_nfa(&re.to_string()).unwrap();
        
        assert_eq!(result,expect)
    }

    #[test]
    fn build_or1() {
        let re="ab|c";
        let accept1=State::new_accept_ref();
        let accept2=State::new_accept_ref();
        let accept3=State::new_accept_ref();
        let a1=Automaton{start_state:State::new_transition_ref(Alphabet('a'), Some(accept1.clone())),end_state:accept1};
        let a2=Automaton{start_state:State::new_transition_ref(Alphabet('b'), Some(accept2.clone())),end_state:accept2};
        let a3=Automaton{start_state:State::new_transition_ref(Alphabet('c'), Some(accept3.clone())),end_state:accept3};
        let a=Automaton::automaton_alternate(&a2, &a3);
        let expect=NFA{automaton:Automaton::concat_automaton(&a1, &a)};
        let result=NFA::construct_nfa(&re.to_string()).unwrap();
        
        assert_eq!(result,expect)
    }

    #[test]
    fn build_asterisk() {
        let re="ab*|c";
        let accept1=State::new_accept_ref();
        let accept2=State::new_accept_ref();
        let accept3=State::new_accept_ref();
        let a1=Automaton{start_state:State::new_transition_ref(Alphabet('a'), Some(accept1.clone())),end_state:accept1};
        let a2=Automaton{start_state:State::new_transition_ref(Alphabet('b'), Some(accept2.clone())),end_state:accept2};
        let a3=Automaton{start_state:State::new_transition_ref(Alphabet('c'), Some(accept3.clone())),end_state:accept3};
        let a2=Automaton::automaton_zero_or_one(&a2);
        let a=Automaton::automaton_alternate(&a2, &a3);
        let expect=NFA{automaton:Automaton::concat_automaton(&a1, &a)};
        let result=NFA::construct_nfa(&re.to_string()).unwrap();
        
        assert_eq!(result,expect)
    }

    #[test]
    fn build_paren() {
        let re="(ab)*|c";
        let accept1=State::new_accept_ref();
        let accept2=State::new_accept_ref();
        let accept3=State::new_accept_ref();
        let a1=Automaton{start_state:State::new_transition_ref(Alphabet('a'), Some(accept1.clone())),end_state:accept1};
        let a2=Automaton{start_state:State::new_transition_ref(Alphabet('b'), Some(accept2.clone())),end_state:accept2};
        let a3=Automaton{start_state:State::new_transition_ref(Alphabet('c'), Some(accept3.clone())),end_state:accept3};
        let ab=Automaton::concat_automaton(&a1, &a2);
        let ab=Automaton::automaton_zero_or_one(&ab);
        let expect=NFA{automaton:Automaton::automaton_alternate(&ab, &a3)};
        let result=NFA::construct_nfa(&re.to_string()).unwrap();
        
        assert_eq!(result,expect)
    }

    #[test]
    fn build_char_class_1() {
        let re="(a.)*|c";
        let accept1=State::new_accept_ref();
        let accept2=State::new_accept_ref();
        let accept3=State::new_accept_ref();
        let a1=Automaton{start_state:State::new_transition_ref(Alphabet('a'), Some(accept1.clone())),end_state:accept1};
        let a2=Automaton{start_state:State::new_transition_ref(Symbol::new_all_char(), Some(accept2.clone())),end_state:accept2};
        let a3=Automaton{start_state:State::new_transition_ref(Alphabet('c'), Some(accept3.clone())),end_state:accept3};
        let ab=Automaton::concat_automaton(&a1, &a2);
        let ab=Automaton::automaton_zero_or_one(&ab);
        let expect=NFA{automaton:Automaton::automaton_alternate(&ab, &a3)};
        let result=NFA::construct_nfa(&re.to_string()).unwrap();
        
        assert_eq!(result,expect)
    }
}