use std::collections::HashSet;
use std::fmt::Debug;

mod state;
pub mod dfa;
pub mod nfa;

pub use dfa::DFA;
pub use nfa::NFA;

use state::{State,StateRef};

use self::state::Symbol;


#[derive(Clone)]
pub struct Automaton{
    start_state:StateRef,
    end_state:StateRef
}

impl Automaton {
    fn concat_automaton(from:&Automaton,to:&Automaton)->Automaton{
        (*from.end_state).borrow_mut().connect(&to.start_state);
        
        Automaton{
            start_state:from.start_state.clone(),
            end_state:to.end_state.clone()
        }
    }

    fn automaton_transition(symbol:&Symbol)->Automaton{
        let end=State::new_accept_ref();
        let transition=State::new_transition_ref(symbol.clone(),Some(end.clone()));
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

    pub fn get_alphabet(&self)->HashSet<char>{
        let flatten_graph=Vec::from(self.clone());

        flatten_graph
        .iter()
        .flat_map(|state| state.adjacent())
        .map(|(symbol,_)|symbol)
        .filter_map(|symbol| match symbol {
            Symbol::Alphabet(c) => Some(c),
            Symbol::CharClass(_)=>None,
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

impl Debug for Automaton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
        .debug_struct("Automaton")
        .field("flat_map:", &Vec::from(self.clone()))
        .field("end_state", &self.end_state)
        .finish()
    }
}

