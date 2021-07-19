use super::ShipController;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use log::{error, info};
use rhai::plugin::*;
use rhai::{Dynamic, Engine, Scope, AST, FLOAT, INT};
use smartstring::alias::CompactString;

type Vec2 = nalgebra::Vector2<f64>;

#[export_module]
mod globals_module {
    #[derive(Copy, Clone)]
    pub struct Globals {
        pub map: *mut std::collections::HashMap<CompactString, Dynamic>,
    }

    #[rhai_fn(index_get, return_raw)]
    pub fn get(obj: Globals, key: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        unsafe {
            match (*obj.map).get(key).cloned() {
                Some(value) => Ok(value),
                None => Err(format!("unknown global variable {:?}", key).into()),
            }
        }
    }

    #[rhai_fn(index_set)]
    pub fn set(obj: Globals, key: &str, value: Dynamic) {
        unsafe {
            (*obj.map).insert(key.into(), value);
        }
    }
}

#[export_module]
mod vec2_module {
    #[rhai_fn(name = "vec2")]
    pub fn vec2ff(x: f64, y: f64) -> Vec2 {
        Vec2::new(x, y)
    }

    #[rhai_fn(name = "vec2")]
    pub fn vec2ii(x: i64, y: i64) -> Vec2 {
        Vec2::new(x as f64, y as f64)
    }

    #[rhai_fn(name = "vec2")]
    pub fn vec2if(x: i64, y: f64) -> Vec2 {
        Vec2::new(x as f64, y)
    }

    #[rhai_fn(name = "vec2")]
    pub fn vec2fi(x: f64, y: i64) -> Vec2 {
        Vec2::new(x, y as f64)
    }

    #[rhai_fn(name = "+")]
    pub fn add(obj: &mut Vec2, other: Vec2) -> Vec2 {
        *obj + other
    }

    #[rhai_fn(name = "-")]
    pub fn sub(obj: &mut Vec2, other: Vec2) -> Vec2 {
        *obj - other
    }

    #[rhai_fn(name = "-")]
    pub fn negate(obj: &mut Vec2) -> Vec2 {
        -*obj
    }

    #[rhai_fn(name = "*")]
    pub fn scalef(obj: &mut Vec2, other: f64) -> Vec2 {
        *obj * other
    }

    #[rhai_fn(name = "*")]
    pub fn scalei(obj: &mut Vec2, other: i64) -> Vec2 {
        *obj * other as f64
    }

    #[rhai_fn(name = "*")]
    pub fn scale2f(obj: &mut f64, other: Vec2) -> Vec2 {
        *obj * other
    }

    #[rhai_fn(name = "*")]
    pub fn scale2i(obj: &mut i64, other: Vec2) -> Vec2 {
        *obj as f64 * other
    }

    #[rhai_fn(name = "/")]
    pub fn divf(obj: &mut Vec2, other: f64) -> Vec2 {
        *obj / other
    }

    #[rhai_fn(name = "/")]
    pub fn divi(obj: &mut Vec2, other: i64) -> Vec2 {
        *obj / other as f64
    }

    #[rhai_fn(get = "x", pure)]
    pub fn get_x(obj: &mut Vec2) -> f64 {
        obj.x
    }

    #[rhai_fn(set = "x")]
    pub fn set_xf(obj: &mut Vec2, value: f64) {
        obj.x = value;
    }

    #[rhai_fn(set = "x")]
    pub fn set_xi(obj: &mut Vec2, value: i64) {
        obj.x = value as f64;
    }

    #[rhai_fn(get = "y", pure)]
    pub fn get_y(obj: &mut Vec2) -> f64 {
        obj.y
    }

    #[rhai_fn(set = "y")]
    pub fn set_yf(obj: &mut Vec2, value: f64) {
        obj.y = value;
    }

    #[rhai_fn(set = "y")]
    pub fn set_yi(obj: &mut Vec2, value: i64) {
        obj.y = value as f64;
    }

    #[rhai_fn(name = "magnitude")]
    pub fn magnitude(obj: &mut Vec2) -> f64 {
        obj.magnitude()
    }

    #[rhai_fn(name = "dot")]
    pub fn dot(obj: &mut Vec2, other: Vec2) -> f64 {
        obj.dot(&other)
    }

    #[rhai_fn(name = "distance")]
    pub fn distance(obj: &mut Vec2, other: Vec2) -> f64 {
        obj.metric_distance(&other)
    }

    #[rhai_fn(name = "angle")]
    pub fn angle(obj: &mut Vec2) -> f64 {
        obj.y.atan2(obj.x)
    }

    #[rhai_fn(name = "normalize")]
    pub fn normalize(obj: &mut Vec2) -> Vec2 {
        obj.normalize()
    }

    fn assert_internal<T: PartialEq + std::fmt::Debug>(
        a: &mut T,
        b: T,
    ) -> Result<(), Box<EvalAltResult>> {
        if *a != b {
            Err(format!("assertion failed: {:?} != {:?}", *a, b).into())
        } else {
            Ok(())
        }
    }

    #[rhai_fn(name = "assert_eq", return_raw)]
    pub fn assert_eq_i64(a: &mut i64, b: i64) -> Result<(), Box<EvalAltResult>> {
        assert_internal(a, b)
    }

    #[rhai_fn(name = "assert_eq", return_raw)]
    pub fn assert_eq_f64(a: &mut f64, b: f64) -> Result<(), Box<EvalAltResult>> {
        assert_internal(a, b)
    }

    #[rhai_fn(name = "assert_eq", return_raw)]
    pub fn assert_eq_vec2(a: &mut Vec2, b: Vec2) -> Result<(), Box<EvalAltResult>> {
        assert_internal(a, b)
    }

    #[rhai_fn(name = "to_string")]
    pub fn to_string(obj: &mut Vec2) -> String {
        format!("({:.2}, {:.2})", obj.x, obj.y)
    }

    #[rhai_fn(name = "to_debug")]
    pub fn to_debug(obj: &mut Vec2) -> String {
        format!("({}, {})", obj.x, obj.y)
    }
}

#[derive(Copy, Clone)]
struct Api {
    handle: ShipHandle,
    sim: *mut Simulation,
}

impl Api {
    fn new(handle: ShipHandle, sim: *mut Simulation) -> Self {
        Self { handle, sim }
    }

    #[allow(clippy::mut_from_ref)]
    fn sim(&self) -> &mut Simulation {
        unsafe { &mut *self.sim }
    }

    fn position(&mut self) -> Vec2 {
        self.sim().ship(self.handle).position().vector
    }

    fn velocity(&mut self) -> Vec2 {
        self.sim().ship(self.handle).velocity()
    }

    fn heading(&mut self) -> f64 {
        self.sim().ship(self.handle).heading()
    }

    fn angular_velocity(&mut self) -> f64 {
        self.sim().ship(self.handle).angular_velocity()
    }

    fn accelerate(&mut self, acceleration: Vec2) {
        self.sim().ship_mut(self.handle).accelerate(acceleration);
    }

    fn torque(&mut self, acceleration: FLOAT) {
        self.sim().ship_mut(self.handle).torque(acceleration);
    }

    fn fire_weapon(&mut self) {
        self.sim().ship_mut(self.handle).fire_weapon(0);
    }

    fn fire_weapon_with_index(&mut self, index: INT) {
        self.sim().ship_mut(self.handle).fire_weapon(index);
    }

    fn explode(&mut self) {
        self.sim().ship_mut(self.handle).explode();
    }
}

pub struct RhaiShipController {
    engine: Engine,
    scope: Scope<'static>,
    #[allow(unused)]
    globals_map: Box<std::collections::HashMap<CompactString, Dynamic>>,
    // TODO share AST across ships
    ast: Option<AST>,
}

impl RhaiShipController {
    pub fn new(handle: ShipHandle, sim: *mut Simulation) -> Self {
        let mut engine = Engine::new();
        engine.set_max_expr_depths(64, 32);

        engine.on_print(|x| info!("Script: {}", x));
        engine.on_debug(|x, src, pos| info!("Script ({}:{:?}): {}", src.unwrap_or(""), pos, x));

        engine
            .register_type::<Api>()
            .register_fn("position", Api::position)
            .register_fn("velocity", Api::velocity)
            .register_fn("heading", Api::heading)
            .register_fn("angular_velocity", Api::angular_velocity)
            .register_fn("accelerate", Api::accelerate)
            .register_fn("torque", Api::torque)
            .register_fn("fire_weapon", Api::fire_weapon)
            .register_fn("fire_weapon", Api::fire_weapon_with_index)
            .register_fn("explode", Api::explode);

        engine.register_global_module(exported_module!(vec2_module).into());
        engine.register_global_module(exported_module!(globals_module).into());

        let api = Api::new(handle, sim);
        let mut globals_map = Box::new(std::collections::HashMap::new());
        let globals = globals_module::Globals {
            map: &mut *globals_map,
        };
        engine.on_var(move |name, _index, _context| match name {
            "api" => Ok(Some(Dynamic::from(api))),
            "globals" => Ok(Some(Dynamic::from(globals))),
            _ => Ok(None),
        });

        Self {
            engine,
            scope: Scope::new(),
            ast: None,
            globals_map,
        }
    }

    pub fn test(&mut self, code: &str) {
        self.upload_code(code);
        if let Some(v) = self
            .engine
            .consume_ast_with_scope(&mut self.scope, self.ast.as_ref().unwrap())
            .err()
        {
            panic!("Test failed: {:?}", v);
        }
    }
}

impl ShipController for RhaiShipController {
    fn upload_code(&mut self, code: &str) {
        match self.engine.compile(code) {
            Ok(ast) => {
                self.ast = Some(ast_rewrite::rewrite_ast(ast));
            }
            Err(msg) => {
                error!("Compilation failed: {}", msg);
            }
        }
    }

    fn start(&mut self) {
        if let Some(ast) = &self.ast {
            if let Err(msg) = self.engine.consume_ast_with_scope(&mut self.scope, &ast) {
                error!("Script error: {}", msg);
                self.ast = None;
            }
        }
        if self.ast.is_some() {
            self.ast.as_mut().unwrap().clear_statements();
        }
    }

    fn tick(&mut self) {
        if let Some(ast) = &self.ast {
            let result: Result<(), _> = self.engine.call_fn(&mut self.scope, &ast, "tick", ());
            if let Err(msg) = result {
                error!("Script error: {}", msg);
                self.ast = None;
            }
        }
    }

    fn write_target(&mut self, target: Vec2) {
        self.scope.push("target", target);
    }
}

#[allow(deprecated)]
mod ast_rewrite {
    use log::debug;
    use rhai::plugin::Module;
    use rhai::{
        BinaryExpr, Engine, Expr, FnCallExpr, Identifier, ScriptFnDef, StaticVec, Stmt, StmtBlock,
        AST,
    };

    pub fn find_globals(ast: &AST) -> std::collections::HashSet<Identifier> {
        let mut globals = std::collections::HashSet::new();
        for stmt in ast.statements() {
            if let Stmt::Let(_, ident, _, _) = stmt {
                globals.insert(ident.name.clone());
            }
        }
        globals
    }

    pub fn parse_expr(code: &str) -> Expr {
        let ast = Engine::new_raw().compile(code).unwrap();
        if let Stmt::Expr(expr) = &ast.statements()[0] {
            expr.clone()
        } else {
            panic!("Failed to parse expression")
        }
    }

    fn rewrite_expr_vec(
        exprs: &StaticVec<Expr>,
        globals: &std::collections::HashSet<Identifier>,
    ) -> StaticVec<Expr> {
        exprs
            .iter()
            .map(|expr| rewrite_expr(expr, globals))
            .collect()
    }

    fn rewrite_binary_expr(
        binary_expr: &BinaryExpr,
        globals: &std::collections::HashSet<Identifier>,
    ) -> BinaryExpr {
        BinaryExpr {
            lhs: rewrite_expr(&binary_expr.lhs, globals),
            rhs: rewrite_expr(&binary_expr.rhs, globals),
        }
    }

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

    pub fn rewrite_expr(expr: &Expr, globals: &std::collections::HashSet<Identifier>) -> Expr {
        match expr {
            Expr::Variable(_, _, bx) => {
                let name = &bx.2;
                if globals.contains(name) {
                    parse_expr(&format!("globals.{}", name))
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
            Expr::FnCall(bx, pos) => {
                Expr::FnCall(Box::new(rewrite_fn_call_expr(&*bx, globals)), *pos)
            }
            Expr::Dot(bx, pos) => Expr::Dot(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
            Expr::Index(bx, pos) => Expr::Index(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
            Expr::And(bx, pos) => Expr::And(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
            Expr::Or(bx, pos) => Expr::Or(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
            _ => expr.clone(),
        }
    }

    pub fn rewrite_stmt_block(
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

    pub fn rewrite_stmt(stmt: &Stmt, globals: &std::collections::HashSet<Identifier>) -> Stmt {
        debug!("stmt={:?}", stmt);
        match stmt {
            Stmt::Let(expr, ident, _, pos) => {
                if globals.contains(&ident.name) {
                    let dot_expr = parse_expr(&format!("globals.{}", ident.name));
                    Stmt::Assignment(
                        Box::new((dot_expr, None, rewrite_expr(expr, globals))),
                        *pos,
                    )
                } else {
                    stmt.clone()
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
            Stmt::FnCall(bx, pos) => {
                Stmt::FnCall(Box::new(rewrite_fn_call_expr(&*bx, globals)), *pos)
            }
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

    pub fn rewrite_ast(ast: AST) -> AST {
        let globals = find_globals(&ast);
        debug!("globals={:?}", globals);
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
}

#[cfg(test)]
mod test {
    use crate::simulation::ship;
    use crate::simulation::Simulation;
    use test_env_log::test;

    #[test]
    fn test_vec2() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, -100.0, 0.0, 100.0, 0.0, 0.0, ship::fighter());
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
        let v1 = vec2(1.0, 2.0);
        let v2 = vec2(3.0, 4.0);
        assert_eq((v1 + v2).x, 4.0);
        assert_eq(v1 + v2, vec2(4.0, 6.0));
        assert_eq(v2.magnitude(), 5.0);
        assert_eq(v1.distance(v2), 2.8284271247461903);
        assert_eq(v1.dot(v2), 11.0);
        assert_eq(-v1, vec2(-1, -2));
        ",
        );
    }

    #[test]
    fn test_pos_vel_hd() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(
            &mut sim,
            1.0,
            2.0,
            3.0,
            4.0,
            std::f64::consts::PI,
            ship::fighter(),
        );
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
        assert_eq(api.position(), vec2(1.0, 2.0));
        assert_eq(api.velocity(), vec2(3.0, 4.0));
        assert_eq(api.heading(), PI());
        ",
        );
    }

    #[test]
    fn test_angle() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter());
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
        assert_eq(vec2(1.0, 0.0).angle(), 0.0);
        assert_eq(vec2(0.0, 1.0).angle(), PI() / 2.0);
        assert_eq(vec2(-1.0, 0.0).angle(), PI());
        assert_eq(vec2(0.0, -1.0).angle(), -PI() / 2.0);
        ",
        );
    }

    #[test]
    fn test_function() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter());
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
            fn foo() {
                assert_eq(api.position(), vec2(0.0, 0.0));
            }

            assert_eq(api.velocity(), vec2(0.0, 0.0));
            foo();
        ",
        );
    }

    #[test]
    fn test_globals() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter());
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            r#"
           let a = 1;
           let b = 2.0;
           let c = b;
           fn foo() {
               assert_eq(a, 1);
               assert_eq(b, 2.0);
               a += 1;
               b += 1.0;
               bar();
           }
           fn bar() {
               assert_eq(a, 2);
               a += 1;
           }
           foo();
           assert_eq(a, 3);
           assert_eq(b, 3.0);
           assert_eq(c, 2.0);
           if 1 == 1 {
               assert_eq(a, 3);
           }
           while a < 4 {
               a += 1;
           }
           assert_eq(a, 4);
       "#,
        );
    }

    #[test]
    fn test_mixed_integer_float() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter());
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            r#"
assert_eq(vec2(1, 2), vec2(1.0, 2.0));
assert_eq(vec2(1.0, 2), vec2(1, 2.0));
assert_eq(vec2(1, 1) * 2.0, vec2(1, 1) * 2);
assert_eq(2.0 * vec2(1, 1), 2 * vec2(1, 1));
assert_eq(vec2(1, 1) / 2.0, vec2(1, 1) / 2);
       "#,
        );
    }
}
