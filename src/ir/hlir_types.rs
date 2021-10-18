use crate::SmlStr;

enum NixPrimitive {
    Bool(bool),
    Int(i64),
    Str(SmlStr),
}

trait NixValue {}

enum NixType {
    Integer,
    String,
    Function,
    List(Box<NixType>),
    AttrSet(Vec<Box<NixType>>),
}

//enum NixStructure {
//List(Vec<Box<dyn >>),
//}
