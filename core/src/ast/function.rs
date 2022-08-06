struct TypeDescriptions {
    pub typeIdentifier: String, // TODO: use enum
    pub typeString: String,
}

struct Expression {
    pub argumentTypes: Option<String>, // TODO: probably not String
    pub id: u64,
    pub isConstant: bool,
    pub isLValue: bool,
    pub isPure: bool,
    pub lValueRequested: bool,
    // pub leftHandSide: Option<Expression>,
    pub nodeType: String, // TODO: use an enum
    pub operator: String, // TODO: use an enum
    // pub rightHandSide: Option<Expression>,
    pub src: String, // TODO: use Source,
    pub typeDescriptions: TypeDescriptions,
}
