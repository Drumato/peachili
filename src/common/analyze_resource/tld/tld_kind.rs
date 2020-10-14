use std::collections::BTreeMap;

type ArgType = String;
type ArgName = String;
type MemberType = String;
type MemberName = String;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
pub enum TLDKind {
    CONST {
        type_name: String,
        expr: String,
    },
    FN {
        return_type: String,
        args: Vec<(ArgName, ArgType)>,
    },
    ALIAS {
        src_type: String,
    },

    STRUCT {
        members: BTreeMap<MemberName, MemberType>,
    },
}
