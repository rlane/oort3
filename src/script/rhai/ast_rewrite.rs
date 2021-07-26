use rhai::plugin::Module;

#[allow(deprecated)]
use rhai::{
    BinaryExpr, Engine, Expr, FnCallExpr, Identifier, Position, ScriptFnDef, StaticVec, Stmt,
    StmtBlock, AST,
};

#[allow(deprecated)]
fn find_globals(ast: &AST) -> std::collections::HashSet<Identifier> {
    let mut globals = std::collections::HashSet::new();
    globals.insert("rng".into());
    for stmt in ast.statements() {
        if let Stmt::Let(_, ident, _, _) = stmt {
            globals.insert(ident.name.clone());
        }
    }
    globals
}

#[allow(deprecated)]
fn parse_expr(code: &str) -> Expr {
    let ast = Engine::new_raw().compile(code).unwrap();
    if let Stmt::Expr(expr) = &ast.statements()[0] {
        expr.clone()
    } else {
        panic!("Failed to parse expression")
    }
}

fn global_variable(name: &str, pos: Position) -> Expr {
    if let Expr::Dot(bx, _) = parse_expr(&format!("globals.{}", name)) {
        let BinaryExpr { lhs, rhs } = &*bx;
        if let Expr::Variable(_, _, bx) = lhs {
            Expr::Dot(
                Box::new(BinaryExpr {
                    lhs: Expr::Variable(None, pos, Box::new((None, None, bx.2.clone()))),
                    rhs: rhs.clone(),
                }),
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
        constants: fn_call_expr.constants.clone(),
        name: fn_call_expr.name.clone(),
        capture: fn_call_expr.capture,
    }
}

#[allow(deprecated)]
fn rewrite_expr(expr: &Expr, globals: &std::collections::HashSet<Identifier>) -> Expr {
    match expr {
        Expr::Variable(_, pos, bx) => {
            let name = &bx.2;
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
        Expr::Dot(bx, pos) => Expr::Dot(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
        Expr::Index(bx, pos) => Expr::Index(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
        Expr::And(bx, pos) => Expr::And(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
        Expr::Or(bx, pos) => Expr::Or(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
        Expr::Stmt(bx) => Expr::Stmt(Box::new(rewrite_stmt_block(bx, globals))),
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
            .map(|stmt| rewrite_stmt(&stmt, globals))
            .collect(),
        block.position(),
    )
}

#[allow(deprecated)]
fn rewrite_stmt(stmt: &Stmt, globals: &std::collections::HashSet<Identifier>) -> Stmt {
    match stmt {
        Stmt::Let(expr, ident, b, pos) => {
            if globals.contains(&ident.name) {
                Stmt::Assignment(
                    Box::new((
                        global_variable(&ident.name, *pos),
                        None,
                        rewrite_expr(expr, globals),
                    )),
                    *pos,
                )
            } else {
                Stmt::Let(rewrite_expr(&expr, globals), ident.clone(), *b, *pos)
            }
        }
        Stmt::If(expr, bx, pos) => Stmt::If(
            rewrite_expr(&expr, globals),
            Box::new((
                rewrite_stmt_block(&bx.0, globals),
                rewrite_stmt_block(&bx.1, globals),
            )),
            *pos,
        ),
        Stmt::Switch(expr, bx, pos) => Stmt::Switch(
            rewrite_expr(&expr, globals),
            Box::new((
                bx.0.iter()
                    .map(|(k, v)| {
                        (
                            *k,
                            Box::new((
                                v.0.as_ref().map(|expr| rewrite_expr(expr, globals)),
                                rewrite_stmt_block(&v.1, globals),
                            )),
                        )
                    })
                    .collect(),
                rewrite_stmt_block(&bx.1, globals),
            )),
            *pos,
        ),
        Stmt::While(expr, bx, pos) => Stmt::While(
            rewrite_expr(&expr, globals),
            Box::new(rewrite_stmt_block(&*bx, globals)),
            *pos,
        ),
        Stmt::Do(bx, expr, b, pos) => Stmt::Do(
            Box::new(rewrite_stmt_block(&*bx, globals)),
            rewrite_expr(&expr, globals),
            *b,
            *pos,
        ),
        Stmt::For(expr, bx, pos) => Stmt::For(
            rewrite_expr(&expr, globals),
            Box::new((
                bx.0.clone(),
                bx.1.clone(),
                rewrite_stmt_block(&bx.2, globals),
            )),
            *pos,
        ),
        Stmt::Assignment(bx, pos) => Stmt::Assignment(
            Box::new((
                rewrite_expr(&bx.0, globals),
                bx.1,
                rewrite_expr(&bx.2, globals),
            )),
            *pos,
        ),
        Stmt::FnCall(bx, pos) => Stmt::FnCall(Box::new(rewrite_fn_call_expr(&*bx, globals)), *pos),
        Stmt::Expr(expr) => Stmt::Expr(rewrite_expr(expr, globals)),
        Stmt::Block(bx, pos) => Stmt::Block(
            bx.iter().map(|stmt| rewrite_stmt(stmt, globals)).collect(),
            *pos,
        ),
        Stmt::TryCatch(bx, pos) => Stmt::TryCatch(
            Box::new((
                rewrite_stmt_block(&bx.0, globals),
                bx.1.clone(),
                rewrite_stmt_block(&bx.2, globals),
            )),
            *pos,
        ),
        Stmt::Return(t, expr_opt, pos) => Stmt::Return(
            *t,
            expr_opt.as_ref().map(|expr| rewrite_expr(&expr, globals)),
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
