fn run_in_env_frame(statement: &AstStatement, env_frame: &mut EnvFrame) {
    match statement {
        AstStatement::FuncCall(call) => {
            run_func_call_in_env_frame(call, env_frame);
        }
        AstStatement::MethodCall(call) => run_method_call_in_env_frame(
            &call.get_base_expr().compute_value_in_env(env_frame),
            call.get_call(),
            env_frame,
        ),
        AstStatement::Assignment(assignment) => {}
    };
}

fn run_func_call_in_env_frame(call: &AstFuncCall, env_frame: &mut EnvFrame) {
    eval_call(call, env_frame, &get_global_functions(), None);
}

fn run_method_call_in_env_frame(
    base_value: &Value<Box<dyn ValueTypeMarker>>,
    call: &AstFuncCall,
    env_frame: &mut EnvFrame,
) {
    eval_call(call, env_frame, &get_global_functions(), Some(base_value));
}

/// Evaluate a method or a function call, depending on base_value
fn eval_call(
    call: &AstFuncCall,
    env_frame: &mut EnvFrame,
    func_call_poll: &FuncCallPool,
    base_value: Option<&Value<Box<dyn ValueTypeMarker>>>,
) -> Value<Box<dyn ValueTypeMarker>> {
    (func_call_poll
        .executors
        .iter()
        .find(|executor| executor.name == *call.get_name())
        .unwrap()
        .func)(call.get_args(), env_frame, base_value)
}

include!("global_functions.rs");
