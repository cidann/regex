
#[derive(Clone,Debug,PartialEq, Eq, PartialOrd, Ord)]
//#[repr(u8)] this adds u8 to distinguish enum variants. This increase size from alignment to 2*alignment
//
pub enum Symbol {
    Alphabet(char),
    CharClass(Class),
    Epsilon,
}

#[derive( Clone,Debug,PartialEq, Eq, PartialOrd, Ord)]
pub enum Class{
    All
}

impl Symbol{
    pub fn new_alphabet(c:char)->Symbol{
        Self::Alphabet(c)
    }
    pub fn new_epsilon()->Symbol{
        Self::Epsilon
    }
    pub fn new_all_char()->Symbol{
        Self::CharClass(Class::All)
    }

    pub fn contains(&self,other:&Self)->bool{
        match self {
            Symbol::Alphabet(c0) => {
                match other {
                    Symbol::Alphabet(c1) => c0==c1,
                    Symbol::CharClass(char_class) => char_class.contains(c0),
                    Symbol::Epsilon => false,
                }
            },
            Symbol::CharClass(char_class) => {
                match other {
                    Symbol::Alphabet(c1) => char_class.contains(c1),
                    Symbol::CharClass(char_class) => false,
                    Symbol::Epsilon => false,
                }
            },
            Symbol::Epsilon => matches!(other,Symbol::Epsilon),
        }
    }
}

impl Class {
    fn contains(&self,c:&char)->bool{
        match self {
            Class::All => {
                true
            },
        }
    }
}