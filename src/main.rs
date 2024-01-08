use parse::regex::automaton::DFA;

fn main(){
    let dfa=DFA::construct_dfa(&"(*)*".to_string());
}