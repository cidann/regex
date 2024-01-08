use std::{cell::RefCell, rc::Rc, sync::atomic::{AtomicUsize, Ordering}, collections::{HashSet, VecDeque}, fmt::Debug};
use std::hash::Hash;

#[derive(Eq, Hash, PartialEq,Clone,Debug,PartialOrd,Ord)]
//#[repr(u8)] this adds u8 to distinguish enum variants. This increase size from alignment to 2*alignment
//
pub enum Symbol {
    Alphabet(char),
    Epsilon,
}


pub type StateRef=Rc<RefCell<State>>;

#[derive(Clone)]
pub enum State {
    Transition(Vec<(Symbol,Option<StateRef>)>,usize),
    Accept(usize)
}

/*
All State will have unique id from function static variable
*/
impl State {
    fn get_new_id()->usize{
        static STATE_ID_COUNTER:AtomicUsize =AtomicUsize::new(0);
        STATE_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    pub fn get_id(&self)->usize{
        match self {
            State::Transition(_,  id) => *id,
            State::Accept(id) => *id,
        }
    }

    pub fn new_transition(s:Symbol,to:Option<StateRef>)->State{
        State::Transition(vec![(s,to)],State::get_new_id())
    }

    pub fn new_split(to1:Option<StateRef>,to2:Option<StateRef>)->State{
        State::Transition(
            vec![(Symbol::Epsilon,to1),(Symbol::Epsilon,to2)],
            State::get_new_id()
        )
    }

    pub fn new_accept()->State{
        State::Accept(State::get_new_id())
    }

    pub fn new_transition_ref(s:Symbol,to:Option<StateRef>)->StateRef{
        Rc::new(RefCell::new(State::new_transition(s, to)))
    }

    pub fn new_split_ref(to1:Option<StateRef>,to2:Option<StateRef>)->StateRef{
        Rc::new(RefCell::new(State::new_split(to1, to2)))
    }

    pub fn new_accept_ref()->StateRef{
        Rc::new(RefCell::new(State::new_accept()))
    }

    pub fn connect(&mut self,other:&StateRef){
        match self {
            State::Transition(adj,_) => {
                for (_,to) in adj{
                    *to=Some(other.clone());
                }
            },
            State::Accept(_) => {
                *self=State::new_transition(Symbol::Epsilon, Some(other.clone()));
            }
        }
    }

    pub fn insert_transition(&mut self,symbol:Symbol,other:&StateRef){
        match self {
            State::Transition(adj,_) => {
                adj.push((symbol,Some(other.clone())))
            },
            State::Accept(_) => {
                *self=State::new_transition(symbol, Some(other.clone()));
            }
        }
    }

    pub fn insert_transition_ord(&mut self,symbol:Symbol,other:&StateRef){
        match self {
            State::Transition(adj,_) => {
                let index=match adj.binary_search_by(|transition|transition.0.cmp(&symbol)) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                adj.insert(index,(symbol,Some(other.clone())));
                
            },
            State::Accept(_) => {
                *self=State::new_transition(symbol, Some(other.clone()));
            }
        }
    }

    pub fn adjacent(&self)->Vec<(Symbol,Option<StateRef>)>{
        match self {
            State::Transition(adj, _) => adj.clone(),
            State::Accept(_) => Vec::new(),
        }
    }

    pub fn dfs(&self,visited1:&mut HashSet<usize>,res:&mut Vec<State>){
        if !visited1.contains(&self.get_id()) {
            res.push(self.clone());
            visited1.insert(self.get_id());
            match self {
                State::Transition(adj, _) => {
                    for (_,next_state) in adj{
                        next_state.iter().for_each(
                            |state_rf|(*state_rf).borrow().dfs(visited1, res)
                        );
                    }
                },
                State::Accept(_) => {},
            }
        }
    }
    
    pub fn delta(&self,symbol:Symbol)->Vec<State>{
        match self {
            State::Transition(adj, _) => {
                let mut next_states=Vec::new();
                for (s,to) in adj{
                    if *s==symbol{
                        //Can also do this
                        //let state:Vec<State>=to.iter().map(|state_ref|state_ref.borrow().clone()).collect();
                        //to.and_then(|state| Some(next_states.push(state)));
                        to.iter().for_each(|state_ref| next_states.push(state_ref.borrow().clone()));
                    }
                }
                next_states
            },
            State::Accept(_) => Vec::new(),
        }
    }

    pub fn follow_epsilon(&self)->Vec<State>{
        let mut visited:HashSet<usize>=HashSet::new();
        let mut epsilons=VecDeque::from([self.clone()]);
        let mut non_epsilons=Vec::new();
        
        while !epsilons.is_empty() {
            let state=epsilons.pop_front().expect("already checked not empty");
            let transitions=state.delta(Symbol::Epsilon);
            if transitions.is_empty(){
                non_epsilons.push(state);
            }
            else {
                for transition in transitions{
                    if !visited.contains(&transition.get_id()){
                        epsilons.push_back(transition.clone());
                        visited.insert(transition.get_id());
                    }
                }
            }
        }
        
        non_epsilons
    }

}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Transition(to0,_), Self::Transition(to1,_)) => {
                if to0.len()==to1.len(){
                    to0
                    .iter()
                    .zip(to1)
                    .all(
                        |(transition0,transition1)| 
                        {
                            transition0.0==transition1.0 &&
                            transition0.1.is_some()==transition1.1.is_some()
                        }
                    )
                }
                else {
                    false
                }
            },
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}


impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transition(adj, _) => {
                f
                .debug_tuple("Transition")
                .field(
                    &adj
                    .iter()
                    .map(|(sym,_)|sym.clone())
                    .collect::<Vec<Symbol>>()
                )
                .field(&adj.len())
                .finish()
            },
            Self::Accept(_) => f.debug_tuple("Accept").finish(),
        }
    }
}