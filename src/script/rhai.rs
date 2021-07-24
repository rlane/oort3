use super::rhai_random::random_module;
use super::ShipController;
use crate::simulation::ship::ShipHandle;
use crate::simulation::Simulation;
use lazy_static::lazy_static;
use log::{error, info};
use nalgebra::{vector, Point2, Rotation2};
use regex::Regex;
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
        let mut a = obj.y.atan2(obj.x);
        if a < 0.0 {
            a += std::f64::consts::TAU;
        }
        a
    }

    #[rhai_fn(name = "normalize")]
    pub fn normalize(obj: &mut Vec2) -> Vec2 {
        obj.normalize()
    }

    #[rhai_fn(name = "rotate")]
    pub fn rotatef(obj: &mut Vec2, angle: f64) -> Vec2 {
        Rotation2::new(angle).transform_vector(obj)
    }

    #[rhai_fn(name = "rotate")]
    pub fn rotatei(obj: &mut Vec2, angle: i64) -> Vec2 {
        rotatef(obj, angle as f64)
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

#[export_module]
mod util_module {
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
    pub fn assert_eq_bool(a: &mut bool, b: bool) -> Result<(), Box<EvalAltResult>> {
        assert_internal(a, b)
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
}

#[export_module]
mod api_module {
    #[derive(Copy, Clone)]
    pub struct Api {
        pub handle: ShipHandle,
        pub sim: *mut Simulation,
    }

    impl Api {
        #[allow(clippy::mut_from_ref)]
        fn sim(&self) -> &mut Simulation {
            unsafe { &mut *self.sim }
        }
    }

    pub fn position(api: Api) -> Vec2 {
        api.sim().ship(api.handle).position().vector
    }

    pub fn velocity(api: Api) -> Vec2 {
        api.sim().ship(api.handle).velocity()
    }

    pub fn heading(api: Api) -> f64 {
        api.sim().ship(api.handle).heading()
    }

    pub fn angular_velocity(api: Api) -> f64 {
        api.sim().ship(api.handle).angular_velocity()
    }

    pub fn accelerate(api: Api, acceleration: Vec2) {
        api.sim().ship_mut(api.handle).accelerate(acceleration);
    }

    pub fn torque(api: Api, acceleration: FLOAT) {
        api.sim().ship_mut(api.handle).torque(acceleration);
    }

    pub fn fire_weapon(api: Api) {
        api.sim().ship_mut(api.handle).fire_weapon(0);
    }

    pub fn fire_weapon_with_index(api: Api, index: INT) {
        api.sim().ship_mut(api.handle).fire_weapon(index);
    }

    pub fn explode(api: Api) {
        api.sim().ship_mut(api.handle).explode();
    }

    #[derive(Copy, Clone)]
    pub struct ScanResult {
        pub found: bool,
        pub position: Vec2,
        pub velocity: Vec2,
    }

    #[rhai_fn(get = "found", pure)]
    pub fn get_found(obj: &mut ScanResult) -> bool {
        obj.found
    }

    #[rhai_fn(get = "position", pure)]
    pub fn get_position(obj: &mut ScanResult) -> Vec2 {
        obj.position
    }

    #[rhai_fn(get = "velocity", pure)]
    pub fn get_velocity(obj: &mut ScanResult) -> Vec2 {
        obj.velocity
    }

    pub fn scan(api: Api) -> ScanResult {
        let sim = api.sim();
        let own_team = sim.ship(api.handle).data().team;
        let own_position: Point2<f64> = sim.ship(api.handle).position().vector.into();
        let mut result = ScanResult {
            found: false,
            position: vector![0.0, 0.0],
            velocity: vector![0.0, 0.0],
        };
        let mut best_distance = 0.0;
        for &other in sim.ships.iter() {
            if sim.ship(other).data().team == own_team {
                continue;
            }
            let other_position: Point2<f64> = sim.ship(other).position().vector.into();
            let distance = nalgebra::distance(&own_position, &other_position);
            if !result.found || distance < best_distance {
                result = ScanResult {
                    found: true,
                    position: other_position.coords,
                    velocity: sim.ship(other).velocity(),
                };
                best_distance = distance;
            }
        }
        result
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
        engine.set_max_operations(1000);

        engine.on_print(|x| info!("Script: {}", x));
        engine.on_debug(|x, src, pos| info!("Script ({}:{:?}): {}", src.unwrap_or(""), pos, x));

        engine.register_global_module(exported_module!(api_module).into());
        engine.register_global_module(exported_module!(vec2_module).into());
        engine.register_global_module(exported_module!(globals_module).into());
        engine.register_global_module(exported_module!(random_module).into());
        engine.register_global_module(exported_module!(util_module).into());

        let (i, j) = handle.0.into_raw_parts();
        let seed = ((i as i64) << 32) | j as i64;
        let rng = random_module::new_rng(seed);

        let api = api_module::Api { handle, sim };
        let mut globals_map = Box::new(std::collections::HashMap::new());
        let globals = globals_module::Globals {
            map: &mut *globals_map,
        };
        globals_map.insert("rng".into(), Dynamic::from(rng));
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

    fn start(&mut self) -> Result<(), super::Error> {
        if let Some(ast) = &self.ast {
            let result = self.engine.consume_ast_with_scope(&mut self.scope, &ast);
            if let Err(e) = result {
                error!("Script error: {}", e);
                self.ast = None;
                return Err(super::Error {
                    line: extract_line(&e.to_string()),
                    msg: e.to_string(),
                });
            }
        }
        if self.ast.is_some() {
            self.ast.as_mut().unwrap().clear_statements();
        }
        Ok(())
    }

    fn tick(&mut self) -> Result<(), super::Error> {
        if let Some(ast) = &self.ast {
            let result: Result<(), _> = self.engine.call_fn(&mut self.scope, &ast, "tick", ());
            if let Err(e) = result {
                error!("Script error: {}", e);
                self.ast = None;
                return Err(super::Error {
                    line: extract_line(&e.to_string()),
                    msg: e.to_string(),
                });
            }
        }
        Ok(())
    }

    fn write_target(&mut self, target: Vec2) {
        self.scope.push("target", target);
    }
}

fn extract_line(msg: &str) -> usize {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"line (?P<line>\d+)").unwrap();
    }
    RE.captures(msg)
        .and_then(|cap| cap.name("line").map(|line| line.as_str().parse()))
        .unwrap_or(Ok(0))
        .unwrap_or(0)
}

#[allow(deprecated)]
mod ast_rewrite {
    use rhai::plugin::Module;
    use rhai::{
        BinaryExpr, Engine, Expr, FnCallExpr, Identifier, Position, ScriptFnDef, StaticVec, Stmt,
        StmtBlock, AST,
    };

    pub fn find_globals(ast: &AST) -> std::collections::HashSet<Identifier> {
        let mut globals = std::collections::HashSet::new();
        globals.insert("rng".into());
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

    pub fn global_variable(name: &str, pos: Position) -> Expr {
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
            Expr::FnCall(bx, pos) => {
                Expr::FnCall(Box::new(rewrite_fn_call_expr(&*bx, globals)), *pos)
            }
            Expr::Dot(bx, pos) => Expr::Dot(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
            Expr::Index(bx, pos) => Expr::Index(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
            Expr::And(bx, pos) => Expr::And(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
            Expr::Or(bx, pos) => Expr::Or(Box::new(rewrite_binary_expr(&*bx, globals)), *pos),
            Expr::Stmt(bx) => Expr::Stmt(Box::new(rewrite_stmt_block(bx, globals))),
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
        let ship0 = ship::create(&mut sim, -100.0, 0.0, 100.0, 0.0, 0.0, ship::fighter(0));
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
        assert_eq(vec2(1, 2).rotate(PI() / 2), vec2(-2, 1.0000000000000002));
        assert_eq(vec2(1, 2).rotate(PI()), vec2(-1.0000000000000002, -1.9999999999999998));
        assert_eq(vec2(1, 2).rotate(3 * PI() / 2), vec2(1.9999999999999998, -1.0000000000000004));
        assert_eq(vec2(3, 4).normalize(), vec2(0.6, 0.8));
        ",
        );
    }

    #[test]
    fn test_vec2_angle() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
        assert_eq(vec2(1.0, 0.0).angle(), 0.0);
        assert_eq(vec2(0.0, 1.0).angle(), PI() / 2.0);
        assert_eq(vec2(-1.0, 0.0).angle(), PI());
        assert_eq(vec2(0.0, -1.0).angle(), 3 * PI() / 2.0);
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
            ship::fighter(0),
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
    fn test_function() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
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
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
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
           print(`a=${a}`);
       "#,
        );
    }

    #[test]
    fn test_mixed_integer_float() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
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

    #[test]
    #[should_panic(expected = "ErrorTooManyOperations")]
    fn test_infinite_loop() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            r#"
let i = 0;
while true {
    print(`i=${i}`);
    i += 1;
}
       "#,
        );
    }

    #[test]
    fn test_random() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            r#"
let rng = new_rng(1);
assert_eq(rng.next(-10.0, 10.0), -5.130375501385842);
assert_eq(rng.next(-10.0, 10.0), -3.0351627041509293);
assert_eq(rng.next(-10.0, 10.0), -4.8407819174603075);
assert_eq(rng.next(-10.0, 10.0), 4.134284076597936);
       "#,
        );
    }

    #[test]
    fn test_scan() {
        let mut sim = Simulation::new();
        let ship0 = ship::create(
            &mut sim,
            1.0,
            2.0,
            3.0,
            4.0,
            std::f64::consts::PI,
            ship::fighter(0),
        );
        let _ship1 = ship::create(
            &mut sim,
            100.0,
            2.0,
            3.0,
            4.0,
            std::f64::consts::PI,
            ship::fighter(1),
        );
        let mut ctrl = super::RhaiShipController::new(ship0, &mut sim);
        ctrl.test(
            "
let contact = api.scan();
assert_eq(contact.found, true);
assert_eq(contact.position, vec2(100, 2));
assert_eq(contact.velocity, vec2(3, 4));
        ",
        );
    }
}
