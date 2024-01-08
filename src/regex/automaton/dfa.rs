use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use super::Automaton;
use super::State;
use super::NFA;
use super::state::StateRef;
use super::state::Symbol;


#[derive(Debug)]
pub struct DFA{
    automaton:Automaton
}

impl PartialEq for DFA {
    fn eq(&self, other: &Self) -> bool {
        self.automaton == other.automaton
    }
}

impl DFA {
    pub fn construct_dfa(re :&String)->Result<DFA,String>{
        let nfa=NFA::construct_nfa(re)?;
        let nfa_success_id=nfa.automaton.end_state.borrow().get_id();
        let alphabet=nfa.automaton.get_alphabet();

        let start_state_set=nfa.automaton.start_state.borrow().follow_epsilon();
        let start=State::new_accept_ref();
        let end=State::new_accept_ref();
        let mut success_dfa_states=Vec::new();
        let mut visited_dfa:HashMap<BTreeSet<usize>,StateRef>=
        [(start_state_set.iter().map(|state| state.get_id()).collect(), start.clone())].into_iter().collect();

        let mut queue=VecDeque::from([(start.clone(),start_state_set)]);

        while !queue.is_empty() {
            let (node_ref,nfa_states)=queue.pop_front().expect("checked for empty");
            if nfa_states.iter().any(|state| state.get_id()==nfa_success_id){
                success_dfa_states.push(node_ref.clone());
            }
            queue.append(&mut DFA::generate_transitions(node_ref.clone(), &nfa_states, &alphabet, &mut visited_dfa));
        }

        for success_state in success_dfa_states{
            success_state.borrow_mut().insert_transition_ord(Symbol::Epsilon, &end)
        }

        Ok(DFA{
            automaton:Automaton{
                start_state:start,
                end_state:end
            }
        })
    }

    pub fn match_input(&self,input:&str)->bool{
        let mut cur_state=self.automaton.start_state.borrow().clone();
        for c in input.chars(){
            cur_state=
            cur_state
            .delta(Symbol::Alphabet(c))
            .into_iter()
            .next()
            .expect("dfa should only have single transitions");
        }
        cur_state.follow_epsilon().iter().any(|final_state| matches!(final_state,State::Accept(_)))
    }

    fn generate_transitions(dfa_state:StateRef,nfa_states:&Vec<State>,alphabet:&HashSet<char>,visited_dfa:&mut HashMap<BTreeSet<usize>,StateRef>)
    ->VecDeque<(StateRef,Vec<State>)>{
        let mut dfa_nfa_pairs=VecDeque::new();
        for c in alphabet{
            let delta_states=DFA::delta_states(*c, nfa_states);
            let new_ids=delta_states.iter().map(|state|state.get_id()).collect();
            if !delta_states.is_empty(){
                let target_dfa_state=visited_dfa.entry(new_ids).or_insert_with(||{
                    let new_node=State::new_accept_ref();
                    dfa_nfa_pairs.push_back((new_node.clone(),delta_states));
                    new_node
                });
                
                dfa_state.borrow_mut().insert_transition_ord(Symbol::Alphabet(*c), &target_dfa_state);
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


#[cfg(test)]
mod dfa{
    use super::*;

    #[test]
    fn concat_1(){
        let re="abc";
        let dfa=DFA::construct_dfa(&re.to_string()).expect("Expect successful dfa construction");
        let accept=State::new_accept_ref();
        let to_accept=State::new_transition_ref(Symbol::Epsilon, Some(accept.clone()));
        let c=State::new_transition_ref(Symbol::Alphabet('c'), Some(to_accept));
        let b=State::new_transition_ref(Symbol::Alphabet('b'), Some(c));
        let a=State::new_transition_ref(Symbol::Alphabet('a'), Some(b));
        let expect=DFA{
            automaton:Automaton{
                start_state:a,
                end_state:accept
            }
        };
        
        assert_eq!(dfa,expect)
    }

    #[test]
    fn or_1(){
        let re="ab|c";
        let dfa=DFA::construct_dfa(&re.to_string()).expect("Expect successful dfa construction");
        let accept=State::new_accept_ref();
        let to_accept=State::new_transition_ref(Symbol::Epsilon, Some(accept.clone()));
        let bc=State::new_transition_ref(Symbol::Alphabet('c'), Some(to_accept.clone()));
        bc.borrow_mut().insert_transition_ord(Symbol::Alphabet('b'), &to_accept);
        let a=State::new_transition_ref(Symbol::Alphabet('a'), Some(bc));
        let expect=DFA{
            automaton:Automaton{
                start_state:a,
                end_state:accept
            }
        };
        
        assert_eq!(dfa,expect)
    }

    #[test]
    fn asterisk_1(){
        let re="ab*c";
        let dfa=DFA::construct_dfa(&re.to_string()).expect("Expect successful dfa construction");
        let accept=State::new_accept_ref();
        let to_accept=State::new_transition_ref(Symbol::Epsilon, Some(accept.clone()));
        let bs_c=State::new_transition_ref(Symbol::Alphabet('c'), Some(to_accept.clone()));
        bs_c.borrow_mut().insert_transition_ord(Symbol::Alphabet('b'), &bs_c);
        let a=State::new_transition_ref(Symbol::Alphabet('a'), Some(bs_c));
        let expect=DFA{
            automaton:Automaton{
                start_state:a,
                end_state:accept
            }
        };
        
        assert_eq!(dfa,expect)
    }
}