use std::{env, io::{stdin, stdout, Write, Read}, fmt::{Binary, Arguments}};
use parse::regex::automaton::DFA;

fn main(){
    let mut args = env::args();
    args.next();
    let re=args.next().expect("Expect regex");
    let dfa=DFA::construct_dfa(&re).expect("Failed to construct dfa");

    let input=stdin();
    let mut output=stdout();
    
    for line in input.lines(){
        let line=line.expect("Failed to read line");
        if dfa.match_input(&line){
            //output.write_fmt(format_args!("{line}\n")).expect("Failed to write to output");
            write!(output,"{line}\n").expect("Failed to write to output");
        }
    }
}