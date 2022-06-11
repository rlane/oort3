use rhai::plugin::Module;

#[allow(deprecated)]
use rhai::{
    BinaryExpr, ConditionalStmtBlock, Engine, Expr, FnCallExpr, Identifier, Position, ScriptFnDef,
    StaticVec, Stmt, StmtBlock, SwitchCases, TryCatchBlock, AST,
};

#[allow(deprecated)]
fn find_globals(ast: &AST) -> std::collections::HashSet<Identifier> {
    let mut globals = std::collections::HashSet::new();
    globals.insert("rng".into());
    globals.insert("orders".into());
    for stmt in ast.statements() {
        if let Stmt::Var(bx, _, _) = stmt {
            globals.insert(bx.0.name.clone());
        }
    }
    globals
}

#[allow(deprecated)]
fn parse_expr(code: &str) -> Expr {
    let ast = Engine::new_raw().compile(code).unwrap();
    if let Stmt::Expr(expr) = &ast.statements()[0] {
        (**expr).clone()
    } else {
        panic!("Failed to parse expression")
    }
}

fn global_variable(name: &str, pos: Position) -> Expr {
    if let Expr::Dot(bx, dummy, _) = parse_expr(&format!("globals.{}", name)) {
        let BinaryExpr { lhs, rhs } = &*bx;
        if let Expr::Variable(bx, _, _) = lhs {
            Expr::Dot(
                Box::new(BinaryExpr {
                    lhs: Expr::Variable(
                        Box::new((None, bx.1.clone(), bx.2, bx.3.clone())),
                        None,
                        pos,
                    ),
                    rhs: rhs.clone(),
                }),
                dummy,
                pos,
            )
        } else {
            panic!("Unexpected Expr")
        }
    } else {
        panic!("Unexpected Expr")
    }
}

#[allow(deprecated)]
fn rewrite_expr_vec(
    exprs: &StaticVec<Expr>,
    globals: &std::collections::HashSet<Identifier>,
) -> StaticVec<Expr> {
    exprs
        .iter()
        .map(|expr| rewrite_expr(expr, globals))
        .collect()
}

#[allow(deprecated)]
fn rewrite_binary_expr(
    binary_expr: &BinaryExpr,
    globals: &std::collections::HashSet<Identifier>,
) -> BinaryExpr {
    BinaryExpr {
        lhs: rewrite_expr(&binary_expr.lhs, globals),
        rhs: rewrite_expr(&binary_expr.rhs, globals),
    }
}

#[allow(deprecated)]
fn rewrite_fn_call_expr(
    fn_call_expr: &FnCallExpr,
    globals: &std::collections::HashSet<Identifier>,
) -> FnCallExpr {
    FnCallExpr {
        namespace: fn_call_expr.namespace.clone(),
        hashes: fn_call_expr.hashes,
        args: rewrite_expr_vec(&fn_call_expr.args, globals),
        name: fn_call_expr.name.clone(),
        capture_parent_scope: fn_call_expr.capture_parent_scope,
        pos: fn_call_expr.pos,
    }
}

#[allow(deprecated)]
fn rewrite_expr(expr: &Expr, globals: &std::collections::HashSet<Identifier>) -> Expr {
    match expr {
        Expr::Variable(bx, _, pos) => {
            let name = &bx.3;
            if globals.contains(name) {
                global_variable(name, *pos)
            } else {
                expr.clone()
            }
        }
        Expr::InterpolatedString(bx, pos) => {
            Expr::InterpolatedString(Box::new(rewrite_expr_vec(&*bx, globals)), *pos)
        }
        Expr::Array(bx, pos) => Expr::Array(Box::new(rewrite_expr_vec(&*bx, globals)), *pos),
        Expr::Map(bx, pos) => Expr::Map(
            Box::new((
                bx.0.iter()
                    .map(|(ident, expr)| (ident.clone(), rewrite_expr(expr, globals)))
                    .collect(),
                bx.1.clone(),
            )),
            *pos,
        ),
        Expr::FnCall(bx, pos) => Expr::FnCall(Box::new(rewrite_fn_call_expr(&*bx, globals)), *pos),
        Expr::Dot(bx, dummy, pos) => {
            Expr::Dot(Box::new(rewrite_binary_expr(&*bx, globals)), *dummy, *pos)
        }
        Expr::Index(bx, stop, pos) => {
            Expr::Index(Box::new(rewrite_binary_expr(&*bx, globals)), *stop, *pos)
        }
        Expr::And(bx, pos) => Expr::And(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
        Expr::Or(bx, pos) => Expr::Or(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
        Expr::Stmt(bx) => Expr::Stmt(Box::new(rewrite_stmt_block(bx, globals))),
        Expr::MethodCall(bx, pos) => {
            Expr::MethodCall(Box::new(rewrite_fn_call_expr(&*bx, globals)), *pos)
        }
        _ => expr.clone(),
    }
}

#[allow(deprecated)]
fn rewrite_stmt_block(
    block: &StmtBlock,
    globals: &std::collections::HashSet<Identifier>,
) -> StmtBlock {
    StmtBlock::new(
        block
            .iter()
            .map(|stmt| rewrite_stmt(stmt, globals))
            .collect::<Vec<Stmt>>(),
        block.position(),
        block.end_position(),
    )
}

#[allow(deprecated)]
fn rewrite_conditional_stmt_block(
    block: &ConditionalStmtBlock,
    globals: &std::collections::HashSet<Identifier>,
) -> ConditionalStmtBlock {
    ConditionalStmtBlock {
        condition: rewrite_expr(&block.condition, globals),
        statements: rewrite_stmt_block(&block.statements, globals),
    }
}

#[allow(deprecated)]
fn rewrite_stmt(stmt: &Stmt, globals: &std::collections::HashSet<Identifier>) -> Stmt {
    match stmt {
        Stmt::Var(bx, option_flags, pos) => {
            let (ident, expr, _) = &**bx;
            if globals.contains(&ident.name) {
                Stmt::Assignment(Box::new((
                    rhai::OpAssignment::new_assignment(*pos),
                    BinaryExpr {
                        lhs: global_variable(&ident.name, *pos),
                        rhs: rewrite_expr(expr, globals),
                    },
                )))
            } else {
                Stmt::Var(
                    Box::new((ident.clone(), rewrite_expr(expr, globals), None)),
                    *option_flags,
                    *pos,
                )
            }
        }
        Stmt::If(bx, pos) => Stmt::If(
            Box::new((
                rewrite_expr(&bx.0, globals),
                rewrite_stmt_block(&bx.1, globals),
                rewrite_stmt_block(&bx.2, globals),
            )),
            *pos,
        ),
        Stmt::Switch(bx, pos) => Stmt::Switch(
            Box::new((
                rewrite_expr(&bx.0, globals),
                SwitchCases {
                    cases: bx
                        .1
                        .cases
                        .iter()
                        .map(|(k, v)| (*k, Box::new(rewrite_conditional_stmt_block(&*v, globals))))
                        .collect(),
                    def_case: Box::new(rewrite_stmt_block(&bx.1.def_case, globals)),
                    ranges: bx.1.ranges.clone(),
                },
            )),
            *pos,
        ),
        Stmt::While(bx, pos) => Stmt::While(
            Box::new((
                rewrite_expr(&bx.0, globals),
                rewrite_stmt_block(&bx.1, globals),
            )),
            *pos,
        ),
        Stmt::Do(bx, b, pos) => Stmt::Do(
            Box::new((
                rewrite_expr(&bx.0, globals),
                rewrite_stmt_block(&bx.1, globals),
            )),
            *b,
            *pos,
        ),
        Stmt::For(bx, pos) => Stmt::For(
            Box::new((
                bx.0.clone(),
                bx.1.clone(),
                rewrite_expr(&bx.2, globals),
                rewrite_stmt_block(&bx.3, globals),
            )),
            *pos,
        ),
        Stmt::Assignment(bx) => {
            Stmt::Assignment(Box::new((bx.0, rewrite_binary_expr(&bx.1, globals))))
        }
        Stmt::FnCall(bx, pos) => Stmt::FnCall(Box::new(rewrite_fn_call_expr(&*bx, globals)), *pos),
        Stmt::Expr(expr) => Stmt::Expr(Box::new(rewrite_expr(expr, globals))),
        Stmt::Block(bx) => Stmt::Block(Box::new(rewrite_stmt_block(&*bx, globals))),
        Stmt::TryCatch(bx, pos) => Stmt::TryCatch(
            Box::new(TryCatchBlock {
                try_block: rewrite_stmt_block(&bx.try_block, globals),
                catch_var: bx.catch_var.clone(),
                catch_block: rewrite_stmt_block(&bx.catch_block, globals),
            }),
            *pos,
        ),
        Stmt::Return(expr_opt, flags, pos) => Stmt::Return(
            expr_opt
                .as_ref()
                .map(|expr| Box::new(rewrite_expr(expr, globals))),
            *flags,
            *pos,
        ),
        _ => stmt.clone(),
    }
}

#[allow(deprecated)]
pub fn rewrite_ast(ast: AST) -> AST {
    let globals = find_globals(&ast);
    let stmts: Vec<Stmt> = ast
        .statements()
        .iter()
        .map(|stmt| rewrite_stmt(stmt, &globals))
        .collect();
    let mut module = Module::new();
    for (_, _, _, _, def) in ast.lib().iter_script_fn_info() {
        module.set_script_fn(ScriptFnDef {
            body: rewrite_stmt_block(&def.body, &globals),
            ..(**def).clone()
        });
    }
    AST::new(stmts, module)
}
