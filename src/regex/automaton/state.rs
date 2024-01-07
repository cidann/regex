use std::{cell::RefCell, rc::Rc, sync::atomic::{AtomicUsize, Ordering}, collections::{HashSet, VecDeque}, fmt::Debug};
use std::hash::Hash;

#[derive(Eq, Hash, PartialEq,Clone,Debug)]
pub enum Symbol {
    Alphabet(char),
    Epsilon
}



pub type StateRef=Rc<RefCell<State>>;

#[derive(Clone)]
pub enum State {
    Transition(Symbol,Option<StateRef>,usize),
    Split(Option<StateRef>,Option<StateRef>,usize),
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
            State::Transition(_, _, id) => *id,
            State::Split(_, _, id) => *id,
            State::Accept(id) => *id,
        }
    }

    pub fn new_transition(c:char,to:Option<StateRef>)->State{
        State::Transition(Symbol::Alphabet(c),to,State::get_new_id())
    }

    pub fn new_split(to1:Option<StateRef>,to2:Option<StateRef>)->State{
        State::Split(to1,to2,State::get_new_id())
    }

    pub fn new_accept()->State{
        State::Accept(State::get_new_id())
    }

    pub fn new_transition_ref(c:char,to:Option<StateRef>)->StateRef{
        Rc::new(RefCell::new(State::new_transition(c, to)))
    }

    pub fn new_split_ref(to1:Option<StateRef>,to2:Option<StateRef>)->StateRef{
        Rc::new(RefCell::new(State::new_split(to1, to2)))
    }

    pub fn new_accept_ref()->StateRef{
        Rc::new(RefCell::new(State::new_accept()))
    }

    pub fn connect(&mut self,other:&StateRef){
        match self {
            State::Transition(_, to,_) => {
                *to=Some(other.clone())
            },
            State::Split(to1, to2,_) => {
                *to1=Some(other.clone());
                *to2=Some(other.clone());
            },
            State::Accept(_) => {
                *self=State::Transition(Symbol::Epsilon, Some(other.clone()),State::get_new_id());
            }
        }
    }

    pub fn dfs(&self,visited1:&mut HashSet<usize>,res:&mut Vec<State>){
        if !visited1.contains(&self.get_id()) {
            res.push(self.clone());
            visited1.insert(self.get_id());
            match self {
                State::Transition(_, next_state, _) => {
                    next_state.as_ref().map(
                        |state_rf|(*state_rf).borrow().dfs(visited1, res)
                    );
                },
                State::Split(next_state1, next_state2, _) =>{
                    next_state1.as_ref().map(
                        |state_rf|(*state_rf).borrow().dfs(visited1, res)
                    );
                    next_state2.as_ref().map(
                        |state_rf|(*state_rf).borrow().dfs(visited1, res)
                    );
                },
                State::Accept(_) => {},
            }
        }
    }
    
    pub fn delta(&self,symbol:Symbol)->Vec<State>{
        match self {
            State::Transition(s, r, _) => {
                if *s==symbol{
                    r.iter().map(|state_ref|state_ref.borrow().clone()).collect()
                }
                else {
                    Vec::new()
                }
            },
            State::Split(r0, r1, _) => {
                let mut valid=Vec::new();
                if matches!(symbol,Symbol::Epsilon){
                    r0.iter().for_each(|valid_ref|valid.push(valid_ref.borrow().clone()));
                    r1.iter().for_each(|valid_ref|valid.push(valid_ref.borrow().clone()));
                }
                valid
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
            (Self::Transition(l0, l1,_), Self::Transition(r0, r1,_)) => {
                l0 == r0 && core::mem::discriminant(l1)==core::mem::discriminant(r1)
            },
            (Self::Split(l0, l1,_), Self::Split(r0, r1,_)) =>{
                core::mem::discriminant(l0)==core::mem::discriminant(r0) && core::mem::discriminant(l1)==core::mem::discriminant(r1)
            },
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}


impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transition(arg0, arg1, _) => f.debug_tuple("Transition").field(arg0).field(&arg1.is_some()).finish(),
            Self::Split(arg0, arg1, _) => f.debug_tuple("Split").field(&arg0.is_some()).field(&arg1.is_some()).finish(),
            Self::Accept(_) => f.debug_tuple("Accept").finish(),
        }
    }
}