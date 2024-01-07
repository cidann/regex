use std::{collections::{HashSet, VecDeque, HashMap}, rc::Rc};
use std::collections::BTreeSet;
use crate::regex::transform::*;

mod state;

use state::{State,StateRef};

use self::state::Symbol;


#[derive(Debug,Clone)]
struct Automaton{
    start_state:StateRef,
    end_state:StateRef
}


#[derive(Debug)]
struct NFA{
    automaton:Automaton
}

impl PartialEq for NFA {
    fn eq(&self, other: &Self) -> bool {
        self.automaton == other.automaton
    }
}

impl NFA {
    fn construct_nfa(re :&String)->Result<NFA,String>{
        let thompson=to_thompson_postfix(re)?;
        let mut stack:Vec<Automaton>=Vec::new();
        //println!("{:?}",thompson);
    
        for op in &thompson{
            match op {
                ThompsonOp::Primary(c)=>{
                    stack.push(Automaton::automaton_transition(c));
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
                _=>{panic!("ThompsonOp should not have parenthese")}
            }
        }
    
        if stack.len()!=1{
            panic!("Expect there to be only 1 automaton for each regex");
        }
    
        Ok(NFA{automaton:stack.pop().unwrap()})
    }
}

#[derive(Debug)]
struct DFA{
    automaton:Automaton,
    success_ids:HashSet<usize>
}

impl PartialEq for DFA {
    fn eq(&self, other: &Self) -> bool {
        self.automaton == other.automaton
    }
}

impl DFA {
    fn construct_dfa(re :&String)->Result<DFA,String>{
        let nfa=NFA::construct_nfa(re)?;
        let nfa_success_id=nfa.automaton.end_state.borrow().get_id();
        let alphabet=nfa.automaton.get_alphabet();

        let start_state_set=nfa.automaton.start_state.borrow().follow_epsilon();
        let start=State::new_accept_ref();
        let start_ids= start_state_set.iter().map(|state| state.get_id()).collect();
        let mut success_dfa_states=HashSet::new();
        let mut visited_dfa:HashSet<BTreeSet<usize>>=HashSet::from([start_ids]);

        let mut queue=VecDeque::from([(start.clone(),start_state_set)]);

        while !queue.is_empty() {
            let (node_ref,nfa_states)=queue.pop_front().expect("checked for empty");
            if nfa_states.iter().any(|state| state.get_id()==nfa_success_id){
                success_dfa_states.insert(node_ref.borrow_mut().get_id());
            }
            queue.append(&mut DFA::generate_transitions(node_ref.clone(), &nfa_states, &alphabet, &mut visited_dfa));
        }

        Ok(DFA{
            automaton:Automaton{
                start_state:start,
                end_state:State::new_accept_ref()//useless maybe improve interface in future
            },
            success_ids:success_dfa_states
        })
    }

    fn match_input(&self,input:&str)->bool{
        let mut cur_state=self.automaton.start_state.borrow().clone();
        for c in input.chars(){
            cur_state=
            cur_state
            .delta(Symbol::Alphabet(c))
            .into_iter()
            .next()
            .expect("dfa should only have single transitions");
        }

        self.success_ids.contains(&cur_state.get_id())
    }

    fn generate_transitions(dfa_state:StateRef,nfa_states:&Vec<State>,alphabet:&Vec<char>,visited_dfa:&mut HashSet<BTreeSet<usize>>)
    ->VecDeque<(StateRef,Vec<State>)>{
        let mut dfa_nfa_pairs=VecDeque::new();
        for c in alphabet{
            let delta_states=DFA::delta_states(*c, nfa_states);
            let new_ids=delta_states.iter().map(|state|state.get_id()).collect();
            println!("==================={c} {:?} {:?}",delta_states,new_ids);
            if !delta_states.is_empty()&&!visited_dfa.contains(&new_ids){
                println!("===================!!{c} {:?}",delta_states);
                let new_node=State::new_accept_ref();
                *dfa_state.borrow_mut()=State::new_transition(*c, Some(new_node.clone()));
                visited_dfa.insert(new_ids);
                dfa_nfa_pairs.push_back((new_node,delta_states))
            }
        }
        dfa_nfa_pairs
    }

    fn delta_states(c:char,nfa_states:&Vec<State>)->Vec<State>{
        let mut visited_nfa:HashMap<usize,State>=HashMap::new();
        for nfa_state in nfa_states{
            let delta_states:Vec<State>=
            nfa_state
            .delta(Symbol::Alphabet(c))
            .iter()
            .filter(|state|!visited_nfa.contains_key(&state.get_id()))
            .flat_map(|state| state.follow_epsilon())
            .filter(|state|!visited_nfa.contains_key(&state.get_id()))
            .collect();
            
            delta_states.into_iter().for_each(|state| {visited_nfa.insert(state.get_id(), state);});
        }

        visited_nfa.into_values().collect()
    }
}


impl Automaton {
    fn concat_automaton(from:&Automaton,to:&Automaton)->Automaton{
        (*from.end_state).borrow_mut().connect(&to.start_state);
        
        Automaton{
            start_state:from.start_state.clone(),
            end_state:to.end_state.clone()
        }
    }

    fn automaton_transition(transition:&char)->Automaton{
        let end=State::new_accept_ref();
        let transition=State::new_transition_ref(*transition,Some(end.clone()));
        Automaton{
            start_state:transition.clone(),
            end_state:end
        }
    }
    
    fn automaton_concat(automaton1:&Automaton,automaton2:&Automaton)->Automaton{
        Automaton::concat_automaton(automaton1, automaton2)
    }
    
    fn automaton_alternate(automaton1:&Automaton,automaton2:&Automaton)->Automaton{
        let new_start_state=State::new_split_ref(Some(automaton1.start_state.clone()), Some(automaton2.start_state.clone()));
        let new_end=State::new_accept_ref();
        (*automaton1.end_state).borrow_mut().connect(&new_end);
        (*automaton2.end_state).borrow_mut().connect(&new_end);
    
        Automaton{
            start_state:new_start_state,
            end_state:new_end
        }
    }
    
    fn automaton_zero_or_one(automaton:&Automaton)->Automaton{
        let new_end=State::new_accept_ref();
        let new_start=State::new_split_ref(Some(automaton.start_state.clone()), Some(new_end.clone()));
        *(*automaton.end_state).borrow_mut()=State::new_split(Some(automaton.start_state.clone()), Some(new_end.clone()));
        Automaton{
            start_state:new_start,
            end_state:new_end
        }
    }

    pub fn get_alphabet(&self)->Vec<char>{
        let flatten_graph=Vec::from(self.clone());

        flatten_graph
        .iter()
        .filter_map(|state| match state {
            State::Transition(symbol, _,_ )=>Some(symbol),
            _=>None
        })
        .filter_map(|symbol| match symbol {
            Symbol::Alphabet(c) => Some(*c),
            Symbol::Epsilon => None,
        })
        .collect()
    }

}

impl From<Automaton> for Vec<State> {
    fn from(value: Automaton) -> Self {
        let mut res=Vec::new();
        value.start_state.borrow().dfs(&mut HashSet::new(), &mut res);
        res
    }
}

impl PartialEq for Automaton {
    fn eq(&self, other: &Self) -> bool {
        let flatten_graph1=Vec::from(self.clone());
        let flatten_graph2=Vec::from(other.clone());
        flatten_graph1==flatten_graph2&&self.start_state==other.start_state&&self.end_state==other.end_state
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    mod nfa{
        use super::*;
        #[test]
        fn build_concat1() {
            let re="abc";
            let accept1=State::new_accept_ref();
            let accept2=State::new_accept_ref();
            let accept3=State::new_accept_ref();
            let a1=Automaton{start_state:State::new_transition_ref('a', Some(accept1.clone())),end_state:accept1};
            let a2=Automaton{start_state:State::new_transition_ref('b', Some(accept2.clone())),end_state:accept2};
            let a3=Automaton{start_state:State::new_transition_ref('c', Some(accept3.clone())),end_state:accept3};
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
            let a1=Automaton{start_state:State::new_transition_ref('a', Some(accept1.clone())),end_state:accept1};
            let a2=Automaton{start_state:State::new_transition_ref('b', Some(accept2.clone())),end_state:accept2};
            let a3=Automaton{start_state:State::new_transition_ref('c', Some(accept3.clone())),end_state:accept3};
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
            let a1=Automaton{start_state:State::new_transition_ref('a', Some(accept1.clone())),end_state:accept1};
            let a2=Automaton{start_state:State::new_transition_ref('b', Some(accept2.clone())),end_state:accept2};
            let a3=Automaton{start_state:State::new_transition_ref('c', Some(accept3.clone())),end_state:accept3};
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
            let a1=Automaton{start_state:State::new_transition_ref('a', Some(accept1.clone())),end_state:accept1};
            let a2=Automaton{start_state:State::new_transition_ref('b', Some(accept2.clone())),end_state:accept2};
            let a3=Automaton{start_state:State::new_transition_ref('c', Some(accept3.clone())),end_state:accept3};
            let ab=Automaton::concat_automaton(&a1, &a2);
            let ab=Automaton::automaton_zero_or_one(&ab);
            let expect=NFA{automaton:Automaton::automaton_alternate(&ab, &a3)};
            let result=NFA::construct_nfa(&re.to_string()).unwrap();
            
            assert_eq!(result,expect)
        }
    }

    mod dfa{
        use super::*;

        #[test]
        fn concat_regex(){
            let re="abc";
            let dfa=DFA::construct_dfa(&re.to_string()).expect("Expect successful dfa construction");
            assert!(dfa.match_input("abc"));
        }

        #[test]
        fn or_regex(){
            let re="ab|c";
            let dfa=DFA::construct_dfa(&re.to_string()).expect("Expect successful dfa construction");
            assert!(dfa.match_input("ab"));
            assert!(dfa.match_input("ac"));
        }
    }

}
