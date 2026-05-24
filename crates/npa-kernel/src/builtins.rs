use crate::{
    decl::{Binder, ConstructorDecl, InductiveDecl, RecursorDecl},
    expr::Expr,
    level::Level,
};

pub fn prop() -> Level {
    Level::zero()
}

pub fn type0() -> Level {
    Level::succ(prop())
}

pub fn nat() -> Expr {
    Expr::konst("Nat", vec![])
}

pub fn nat_zero() -> Expr {
    Expr::konst("Nat.zero", vec![])
}

pub fn nat_succ(arg: Expr) -> Expr {
    Expr::app(Expr::konst("Nat.succ", vec![]), arg)
}

pub fn eq(level: Level, ty: Expr, lhs: Expr, rhs: Expr) -> Expr {
    Expr::apps(Expr::konst("Eq", vec![level]), vec![ty, lhs, rhs])
}

pub fn eq_refl(level: Level, ty: Expr, value: Expr) -> Expr {
    Expr::apps(Expr::konst("Eq.refl", vec![level]), vec![ty, value])
}

pub fn eq_type(level: Level) -> Expr {
    Expr::pi(
        "A",
        Expr::sort(level),
        Expr::pi(
            "lhs",
            Expr::bvar(0),
            Expr::pi("rhs", Expr::bvar(1), Expr::sort(prop())),
        ),
    )
}

pub fn eq_refl_type(level: Level) -> Expr {
    Expr::pi(
        "A",
        Expr::sort(level.clone()),
        Expr::pi(
            "x",
            Expr::bvar(0),
            eq(level, Expr::bvar(1), Expr::bvar(0), Expr::bvar(0)),
        ),
    )
}

pub fn eq_rec_type(value_level: Level, motive_level: Level) -> Expr {
    let a_sort_level = value_level.clone();
    let motive_ty = Expr::pi(
        "b",
        Expr::bvar(1),
        Expr::pi(
            "h",
            eq(
                value_level.clone(),
                Expr::bvar(2),
                Expr::bvar(1),
                Expr::bvar(0),
            ),
            Expr::sort(motive_level),
        ),
    );
    let refl_proof = eq_refl(value_level.clone(), Expr::bvar(2), Expr::bvar(1));
    let minor_ty = Expr::apps(Expr::bvar(0), vec![Expr::bvar(1), refl_proof]);
    let major_ty = eq(value_level, Expr::bvar(4), Expr::bvar(3), Expr::bvar(0));
    let result_ty = Expr::apps(Expr::bvar(3), vec![Expr::bvar(1), Expr::bvar(0)]);

    Expr::pi(
        "A",
        Expr::sort(a_sort_level),
        Expr::pi(
            "a",
            Expr::bvar(0),
            Expr::pi(
                "motive",
                motive_ty,
                Expr::pi(
                    "minor",
                    minor_ty,
                    Expr::pi("b", Expr::bvar(3), Expr::pi("h", major_ty, result_ty)),
                ),
            ),
        ),
    )
}

pub fn setoid(level: Level, carrier: Expr) -> Expr {
    Expr::app(Expr::konst("Setoid", vec![level]), carrier)
}

pub fn rel_equiv(level: Level, carrier: Expr, relation: Expr) -> Expr {
    Expr::apps(
        Expr::konst("RelEquiv", vec![level]),
        vec![carrier, relation],
    )
}

pub fn setoid_relation(level: Level, carrier: Expr, setoid: Expr, lhs: Expr, rhs: Expr) -> Expr {
    Expr::apps(
        Expr::konst("Setoid.r", vec![level]),
        vec![carrier, setoid, lhs, rhs],
    )
}

pub fn quotient(level: Level, carrier: Expr, setoid: Expr) -> Expr {
    Expr::apps(Expr::konst("Quotient", vec![level]), vec![carrier, setoid])
}

pub fn quotient_mk(level: Level, carrier: Expr, setoid: Expr, value: Expr) -> Expr {
    Expr::apps(
        Expr::konst("Quotient.mk", vec![level]),
        vec![carrier, setoid, value],
    )
}

pub fn setoid_type(level: Level) -> Expr {
    Expr::pi(
        "A",
        Expr::sort(Level::succ(level.clone())),
        Expr::sort(Level::succ(level)),
    )
}

pub fn rel_equiv_type(level: Level) -> Expr {
    let relation_ty = Expr::pi(
        "_",
        Expr::bvar(0),
        Expr::pi("_", Expr::bvar(1), Expr::sort(prop())),
    );
    Expr::pi(
        "A",
        Expr::sort(Level::succ(level)),
        Expr::pi("r", relation_ty, Expr::sort(prop())),
    )
}

pub fn setoid_mk_type(level: Level) -> Expr {
    let relation_ty = Expr::pi(
        "_",
        Expr::bvar(0),
        Expr::pi("_", Expr::bvar(1), Expr::sort(prop())),
    );
    let equivalence_ty = rel_equiv(level.clone(), Expr::bvar(1), Expr::bvar(0));
    let setoid_ty = setoid(level.clone(), Expr::bvar(2));
    Expr::pi(
        "A",
        Expr::sort(Level::succ(level)),
        Expr::pi(
            "r",
            relation_ty,
            Expr::pi("equiv", equivalence_ty, setoid_ty),
        ),
    )
}

pub fn setoid_relation_type(level: Level) -> Expr {
    Expr::pi(
        "A",
        Expr::sort(Level::succ(level.clone())),
        Expr::pi(
            "s",
            setoid(level.clone(), Expr::bvar(0)),
            Expr::pi(
                "a",
                Expr::bvar(1),
                Expr::pi("b", Expr::bvar(2), Expr::sort(prop())),
            ),
        ),
    )
}

pub fn quotient_type(level: Level) -> Expr {
    Expr::pi(
        "A",
        Expr::sort(Level::succ(level.clone())),
        Expr::pi(
            "s",
            setoid(level.clone(), Expr::bvar(0)),
            Expr::sort(Level::succ(level)),
        ),
    )
}

pub fn quotient_mk_type(level: Level) -> Expr {
    Expr::pi(
        "A",
        Expr::sort(Level::succ(level.clone())),
        Expr::pi(
            "s",
            setoid(level.clone(), Expr::bvar(0)),
            Expr::pi(
                "a",
                Expr::bvar(1),
                quotient(level, Expr::bvar(2), Expr::bvar(1)),
            ),
        ),
    )
}

pub fn quotient_sound_type(level: Level) -> Expr {
    let relation_premise = setoid_relation(
        level.clone(),
        Expr::bvar(3),
        Expr::bvar(2),
        Expr::bvar(1),
        Expr::bvar(0),
    );
    let quotient_for_s = quotient(level.clone(), Expr::bvar(4), Expr::bvar(3));
    let lhs = quotient_mk(level.clone(), Expr::bvar(4), Expr::bvar(3), Expr::bvar(2));
    let rhs = quotient_mk(level.clone(), Expr::bvar(4), Expr::bvar(3), Expr::bvar(1));
    let equality = eq(Level::succ(level.clone()), quotient_for_s, lhs, rhs);
    Expr::pi(
        "A",
        Expr::sort(Level::succ(level.clone())),
        Expr::pi(
            "s",
            setoid(level.clone(), Expr::bvar(0)),
            Expr::pi(
                "a",
                Expr::bvar(1),
                Expr::pi(
                    "b",
                    Expr::bvar(2),
                    Expr::pi("p", relation_premise, equality),
                ),
            ),
        ),
    )
}

pub fn quotient_lift_type(carrier_level: Level, result_level: Level) -> Expr {
    let relation_premise = setoid_relation(
        carrier_level.clone(),
        Expr::bvar(5),
        Expr::bvar(3),
        Expr::bvar(1),
        Expr::bvar(0),
    );
    let lhs = Expr::app(Expr::bvar(3), Expr::bvar(2));
    let rhs = Expr::app(Expr::bvar(3), Expr::bvar(1));
    let compatibility_result = eq(Level::succ(result_level.clone()), Expr::bvar(5), lhs, rhs);
    let compatibility_ty = Expr::pi(
        "a",
        Expr::bvar(3),
        Expr::pi(
            "b",
            Expr::bvar(4),
            Expr::pi("p", relation_premise, compatibility_result),
        ),
    );
    Expr::pi(
        "A",
        Expr::sort(Level::succ(carrier_level.clone())),
        Expr::pi(
            "B",
            Expr::sort(Level::succ(result_level)),
            Expr::pi(
                "s",
                setoid(carrier_level.clone(), Expr::bvar(1)),
                Expr::pi(
                    "f",
                    Expr::pi("_", Expr::bvar(2), Expr::bvar(2)),
                    Expr::pi(
                        "h",
                        compatibility_ty,
                        Expr::pi(
                            "q",
                            quotient(carrier_level, Expr::bvar(4), Expr::bvar(2)),
                            Expr::bvar(4),
                        ),
                    ),
                ),
            ),
        ),
    )
}

pub fn nat_rec_type(level: Level) -> Expr {
    let motive_ty = Expr::pi("_", nat(), Expr::sort(level.clone()));
    let z_ty = Expr::app(Expr::bvar(0), nat_zero());

    let s_ty = Expr::pi(
        "n",
        nat(),
        Expr::pi(
            "ih",
            Expr::app(Expr::bvar(2), Expr::bvar(0)),
            Expr::app(Expr::bvar(3), nat_succ(Expr::bvar(1))),
        ),
    );

    Expr::pi(
        "motive",
        motive_ty,
        Expr::pi(
            "z",
            z_ty,
            Expr::pi(
                "s",
                s_ty,
                Expr::pi("n", nat(), Expr::app(Expr::bvar(3), Expr::bvar(0))),
            ),
        ),
    )
}

pub fn nat_inductive() -> InductiveDecl {
    InductiveDecl::new(
        "Nat",
        vec![],
        vec![],
        vec![],
        type0(),
        vec![
            ConstructorDecl::new("Nat.zero", nat()),
            ConstructorDecl::new("Nat.succ", Expr::pi("_", nat(), nat())),
        ],
        Some(RecursorDecl::new(
            "Nat.rec",
            vec!["u".to_owned()],
            nat_rec_type(Level::param("u")),
        )),
    )
}

pub fn eq_inductive() -> InductiveDecl {
    InductiveDecl::new(
        "Eq",
        vec!["u".to_owned()],
        vec![
            Binder::new("A", Expr::sort(Level::param("u"))),
            Binder::new("lhs", Expr::bvar(0)),
        ],
        vec![Binder::new("rhs", Expr::bvar(1))],
        prop(),
        vec![ConstructorDecl::new(
            "Eq.refl",
            eq_refl_type(Level::param("u")),
        )],
        None,
    )
}
